//! Real backdrop blur for the UI (HUD frosted-glass chips + the pause overlay),
//! phase 2 of the HUD redesign (see `docs/hud-redesign.md`).
//!
//! Bevy UI has no backdrop-filter primitive, so the effect is built from two
//! pieces that never touch the existing (HDR / Solari / DLSS) camera:
//!
//! 1. A render-graph **blit node** ([`BackdropBlitNode`]) inserted between
//!    `Node3d::EndMainPassPostProcessing` and the UI pass. It copies the final
//!    *tonemapped* scene (the view target at that point) into a standalone
//!    [`BackdropImage`] using Bevy's own [`BlitPipeline`]. Running before the UI
//!    pass means the copy holds the scene only — never the HUD itself.
//! 2. A [`BackdropBlurMaterial`] (`UiMaterial`) that samples that image at its
//!    own screen position (`@builtin(position) / view.viewport`) with a
//!    golden-angle disk blur, tints it and clips to the node's rounded rect.
//!    The node's `BorderColor` is left untouched, so the existing HUD border
//!    recolor keeps working — only the *fill* becomes frosted glass.
//!
//! The HUD swaps between a tiny shared palette ([`BackdropMaterials`]) of
//! normal / focused / disabled fills rather than mutating per-chip materials,
//! so focus changes are a cheap `Handle` swap with no bind-group churn.
//!
//! The backdrop image is sized once to the window and never resized: both the
//! blit (full-screen triangle, uv 0..1) and the sampling (uv = screen / viewport)
//! are resolution-independent, so a later window resize only makes the frost a
//! touch softer — invisible under a blur.

use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::blit::{BlitPipeline, BlitPipelineKey};
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::ecs::query::QueryItem;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
    NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::{
    AsBindGroup, CachedRenderPipelineId, Operations, PipelineCache, RenderPassColorAttachment,
    RenderPassDescriptor, ShaderType, SpecializedRenderPipelines, TextureFormat,
};
use bevy::render::renderer::RenderContext;
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewTarget;
use bevy::shader::ShaderRef;
use bevy::ui_render::graph::NodeUi;
use bevy::ui_render::prelude::{UiMaterial, UiMaterialPlugin};
use bevy::window::PrimaryWindow;

/// HDR-friendly format for the backdrop copy. Matches the HDR view target, but
/// the blit samples the source as a plain float texture so the copy works for
/// any view-target format (the choice here only fixes the *destination*).
const BACKDROP_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

// Frosted-glass palette. The fills are translucent so the blurred scene reads
// through; borders stay on the nodes (`BorderColor`) and are recolored by the
// HUD systems as before.
const FROST_NORMAL: Color = Color::srgba(0.04, 0.05, 0.08, 0.72);
const FROST_FOCUSED: Color = Color::srgba(0.30, 0.33, 0.45, 0.80);
const FROST_DISABLED: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const PAUSE_TINT: Color = Color::srgba(0.03, 0.04, 0.07, 0.55);
const CHIP_BLUR: f32 = 6.0;
const PAUSE_BLUR: f32 = 16.0;

/// `UiMaterial` that frosts the scene behind the node: disk-blurred backdrop +
/// tint, clipped to the node's rounded rect.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BackdropBlurMaterial {
    #[uniform(0)]
    pub settings: BackdropSettings,
    #[texture(1)]
    #[sampler(2)]
    pub backdrop: Handle<Image>,
}

#[derive(Clone, Copy, ShaderType)]
pub struct BackdropSettings {
    /// Frost tint: `rgb` over the blur, `a` = how opaque the tint is.
    pub tint: Vec4,
    /// Blur disk radius in pixels.
    pub blur_radius: f32,
}

impl UiMaterial for BackdropBlurMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/backdrop_blur.wgsl".into()
    }
}

/// The standalone texture the blit node writes the scene into and every
/// [`BackdropBlurMaterial`] samples. Extracted so the render node can find it.
#[derive(Resource, Clone, ExtractResource)]
pub struct BackdropImage(pub Handle<Image>);

/// Shared frosted-glass materials. The HUD swaps a chip's `MaterialNode` handle
/// between these instead of editing a per-chip material.
#[derive(Resource)]
pub struct BackdropMaterials {
    pub normal: Handle<BackdropBlurMaterial>,
    pub focused: Handle<BackdropBlurMaterial>,
    pub disabled: Handle<BackdropBlurMaterial>,
    pub pause: Handle<BackdropBlurMaterial>,
}

fn tint(color: Color) -> Vec4 {
    let c = color.to_linear();
    Vec4::new(c.red, c.green, c.blue, c.alpha)
}

/// Startup: create the backdrop texture (sized to the window) and the shared
/// material palette. Must run before `setup_ui`, which reads `BackdropMaterials`.
pub fn init_backdrop_assets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<BackdropBlurMaterial>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let (width, height) = windows
        .single()
        .ok()
        .map(|win| (win.physical_width(), win.physical_height()))
        .filter(|(w, h)| *w > 1 && *h > 1)
        .unwrap_or((1920, 1080));

    let mut image = Image::new_target_texture(width, height, BACKDROP_FORMAT, None);
    // The image lives only on the GPU (rendered into every frame, sampled by UI).
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.sampler = ImageSampler::linear();
    let handle = images.add(image);

    let mut frost = |color: Color, blur: f32| {
        materials.add(BackdropBlurMaterial {
            settings: BackdropSettings {
                tint: tint(color),
                blur_radius: blur,
            },
            backdrop: handle.clone(),
        })
    };

    commands.insert_resource(BackdropMaterials {
        normal: frost(FROST_NORMAL, CHIP_BLUR),
        focused: frost(FROST_FOCUSED, CHIP_BLUR),
        disabled: frost(FROST_DISABLED, CHIP_BLUR),
        pause: frost(PAUSE_TINT, PAUSE_BLUR),
    });
    commands.insert_resource(BackdropImage(handle));
}

// ---------------------------------------------------------------------------
// Render-graph node: blit the post-processed scene into the backdrop image.
// ---------------------------------------------------------------------------

#[derive(RenderLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BackdropBlitLabel;

// `FromWorld` is auto-derived from `Default`, which `add_render_graph_node`
// (via `ViewNodeRunner`) requires.
#[derive(Default)]
struct BackdropBlitNode {
    pipeline_id: Option<CachedRenderPipelineId>,
}

impl ViewNode for BackdropBlitNode {
    type ViewQuery = &'static ViewTarget;

    fn update(&mut self, world: &mut World) {
        if self.pipeline_id.is_some() || !world.contains_resource::<BlitPipeline>() {
            return;
        }
        // The destination format never changes, so this specializes exactly once.
        let key = BlitPipelineKey {
            texture_format: BACKDROP_FORMAT,
            blend_state: None,
            samples: 1,
        };
        let id = world.resource_scope(
            |world, mut pipelines: Mut<SpecializedRenderPipelines<BlitPipeline>>| {
                let blit = world.resource::<BlitPipeline>();
                let cache = world.resource::<PipelineCache>();
                pipelines.specialize(cache, blit, key)
            },
        );
        self.pipeline_id = Some(id);
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        view_target: QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Some(pipeline_id) = self.pipeline_id else {
            return Ok(());
        };
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
            return Ok(());
        };
        let Some(backdrop) = world.get_resource::<BackdropImage>() else {
            return Ok(());
        };
        let Some(dst) = world.resource::<RenderAssets<GpuImage>>().get(&backdrop.0) else {
            return Ok(());
        };
        let blit = world.resource::<BlitPipeline>();

        // Source = the tonemapped scene (we run after EndMainPassPostProcessing).
        let bind_group = blit.create_bind_group(
            render_context.render_device(),
            view_target.main_texture_view(),
            pipeline_cache,
        );

        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("backdrop_blit"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &dst.texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}

pub struct BackdropBlurPlugin;

impl Plugin for BackdropBlurPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UiMaterialPlugin::<BackdropBlurMaterial>::default(),
            ExtractResourcePlugin::<BackdropImage>::default(),
        ))
        .add_systems(Startup, init_backdrop_assets);

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_graph_node::<ViewNodeRunner<BackdropBlitNode>>(
                    Core3d,
                    BackdropBlitLabel,
                )
                .add_render_graph_edges(
                    Core3d,
                    (
                        Node3d::EndMainPassPostProcessing,
                        BackdropBlitLabel,
                        NodeUi::UiPass,
                    ),
                );
        }
    }
}
