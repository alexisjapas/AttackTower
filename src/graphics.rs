use bevy::anti_alias::fxaa::Fxaa;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::VolumetricFog;
use bevy::pbr::{
    Atmosphere, AtmosphereSettings, DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion,
    ScreenSpaceAmbientOcclusionQualityLevel,
};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::motion_blur::MotionBlur;
use bevy::prelude::*;
use bevy::render::view::Msaa;
#[cfg(feature = "raytracing")]
use bevy::solari::prelude::SolariLighting;
use bevy::window::{PresentMode, WindowMode};
use std::path::PathBuf;

use crate::common::*;

/// Settings UX backend AND applier: preset detection, invariants between
/// graphics settings, persistence, the FPS cap, and every system that pushes
/// `GameSettings` onto the world (camera components, window mode, raytracing/
/// DLSS toggling, colorblind palette). The `GameSettings` resource itself is
/// inserted by `main` (loaded + sanitized before the App is built).
pub struct GraphicsSettingsPlugin;

impl Plugin for GraphicsSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsPreset>()
            .init_resource::<SettingsTab>()
            .init_resource::<SettingsOrigin>()
            .add_systems(Update, update_graphics_preset.in_set(AppSet::React))
            .add_systems(
                Update,
                (
                    enforce_settings_invariants,
                    persist_settings,
                    apply_graphics_settings,
                    apply_raytracing_setting,
                    detect_dlss_support,
                    apply_dlss_setting,
                    apply_colorblind_palette,
                )
                    .in_set(AppSet::Visual),
            )
            .add_systems(Update, limit_fps.in_set(AppSet::FrameLimit));
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Parameters
// ────────────────────────────────────────────────────────────────────────────

/// Identifier for every graphics/video parameter that can be tweaked. Using a
/// named enum (instead of bare indices) keeps the menu layout (positions per
/// tab) decoupled from label/description lookup.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParamId {
    // Video / display
    Fullscreen,
    VSync,
    Msaa,
    Hdr,
    Exposure, // sub-param of Hdr
    Tonemapping,
    FpsCap,
    Colorblind,
    // Graphics / quality
    Raytracing,
    Dlss,
    DlssQuality, // sub-param of Dlss
    Taa,
    Fxaa,
    Bloom,
    BloomIntensity, // sub-param of Bloom
    Atmosphere,
    VolumetricFog,
    FogDensity, // sub-param of VolumetricFog
    DistanceFog,
    Ssao,
    SsaoQuality, // sub-param of Ssao
    Shadows,
    MotionBlur,
}

/// A row in the settings menu. The `Preset` slot only exists on the Graphics
/// tab; `Back` closes the menu. `PartialEq` lets the overlay compare slot
/// lists to rebuild only on structural changes (rows appearing/disappearing).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuSlot {
    Preset,
    Param(ParamId),
    Back,
}

/// Build the menu rows for `tab` given the current settings. Sub-parameters
/// only appear when their parent toggle is on, so the slot list shrinks /
/// grows as the user enables features.
pub fn tab_slots(tab: SettingsTab, s: &GameSettings) -> Vec<MenuSlot> {
    let mut v = Vec::with_capacity(16);
    match tab {
        SettingsTab::Video => {
            v.push(MenuSlot::Param(ParamId::Fullscreen));
            v.push(MenuSlot::Param(ParamId::VSync));
            v.push(MenuSlot::Param(ParamId::Msaa));
            v.push(MenuSlot::Param(ParamId::Hdr));
            if s.hdr {
                v.push(MenuSlot::Param(ParamId::Exposure));
            }
            v.push(MenuSlot::Param(ParamId::Tonemapping));
            v.push(MenuSlot::Param(ParamId::FpsCap));
            v.push(MenuSlot::Param(ParamId::Colorblind));
        }
        SettingsTab::Graphics => {
            v.push(MenuSlot::Preset);
            v.push(MenuSlot::Param(ParamId::Raytracing));
            v.push(MenuSlot::Param(ParamId::Dlss));
            if s.dlss {
                v.push(MenuSlot::Param(ParamId::DlssQuality));
            }
            v.push(MenuSlot::Param(ParamId::Taa));
            v.push(MenuSlot::Param(ParamId::Fxaa));
            v.push(MenuSlot::Param(ParamId::Bloom));
            if s.bloom {
                v.push(MenuSlot::Param(ParamId::BloomIntensity));
            }
            v.push(MenuSlot::Param(ParamId::Atmosphere));
            v.push(MenuSlot::Param(ParamId::VolumetricFog));
            if s.volumetric_fog {
                v.push(MenuSlot::Param(ParamId::FogDensity));
            }
            v.push(MenuSlot::Param(ParamId::DistanceFog));
            v.push(MenuSlot::Param(ParamId::Ssao));
            if s.ssao {
                v.push(MenuSlot::Param(ParamId::SsaoQuality));
            }
            v.push(MenuSlot::Param(ParamId::Shadows));
            v.push(MenuSlot::Param(ParamId::MotionBlur));
        }
    }
    v.push(MenuSlot::Back);
    v
}

pub fn slot_count(tab: SettingsTab, s: &GameSettings) -> usize {
    tab_slots(tab, s).len()
}

// ────────────────────────────────────────────────────────────────────────────
// Preset
// ────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GraphicsPreset {
    Low,
    Medium,
    High,
    Ultra,
    #[default]
    Custom,
}

impl GraphicsPreset {
    pub fn label(self) -> &'static str {
        match self {
            GraphicsPreset::Low => "Low",
            GraphicsPreset::Medium => "Medium",
            GraphicsPreset::High => "High",
            GraphicsPreset::Ultra => "Ultra",
            GraphicsPreset::Custom => "Custom",
        }
    }

    /// Cycle through the four named presets. `Custom` is never produced by
    /// activation — it is derived by `detect` when no preset matches.
    pub fn cycle(self) -> Self {
        match self {
            GraphicsPreset::Low => GraphicsPreset::Medium,
            GraphicsPreset::Medium => GraphicsPreset::High,
            GraphicsPreset::High => GraphicsPreset::Ultra,
            GraphicsPreset::Ultra | GraphicsPreset::Custom => GraphicsPreset::Low,
        }
    }

    /// Apply this preset's quality settings to `s`. Display/video fields
    /// (fullscreen, vsync, hdr, msaa, tonemapping) and sub-parameters
    /// (exposure, bloom intensity, dlss quality, ...) are left untouched —
    /// they are managed independently from the preset.
    pub fn apply(self, s: &mut GameSettings, dlss_supported: bool, rt_supported: bool) {
        match self {
            GraphicsPreset::Low => {
                s.raytracing = false;
                s.dlss = false;
                s.taa = false;
                s.fxaa = true; // cheap fallback AA
                s.bloom = false;
                s.atmosphere = false;
                s.volumetric_fog = false;
                s.distance_fog = false;
                s.ssao = false;
                s.shadows = false;
                s.motion_blur = false;
            }
            GraphicsPreset::Medium => {
                s.raytracing = false;
                s.dlss = false;
                s.taa = false;
                s.fxaa = true;
                s.bloom = true;
                s.atmosphere = false;
                s.volumetric_fog = false;
                s.distance_fog = true;
                s.ssao = false;
                s.shadows = true;
                s.motion_blur = false;
            }
            GraphicsPreset::High => {
                s.raytracing = false;
                s.dlss = false;
                s.taa = true;
                s.fxaa = false;
                s.bloom = true;
                s.atmosphere = true;
                s.volumetric_fog = false;
                s.distance_fog = true;
                s.ssao = true;
                s.shadows = true;
                s.motion_blur = false;
            }
            GraphicsPreset::Ultra => {
                s.raytracing = cfg!(feature = "raytracing") && rt_supported;
                s.dlss = cfg!(feature = "dlss") && dlss_supported;
                s.taa = true;
                s.fxaa = false;
                s.bloom = true;
                s.atmosphere = true;
                s.volumetric_fog = true;
                s.distance_fog = true;
                s.ssao = true;
                s.shadows = true;
                s.motion_blur = true;
            }
            GraphicsPreset::Custom => {}
        }
    }

    pub fn detect(s: &GameSettings, dlss_supported: bool, rt_supported: bool) -> Self {
        for preset in [Self::Low, Self::Medium, Self::High, Self::Ultra] {
            let mut probe = *s;
            preset.apply(&mut probe, dlss_supported, rt_supported);
            if quality_matches(s, &probe) {
                return preset;
            }
        }
        Self::Custom
    }
}

fn quality_matches(a: &GameSettings, b: &GameSettings) -> bool {
    a.raytracing == b.raytracing
        && a.dlss == b.dlss
        && a.taa == b.taa
        && a.fxaa == b.fxaa
        && a.bloom == b.bloom
        && a.atmosphere == b.atmosphere
        && a.volumetric_fog == b.volumetric_fog
        && a.distance_fog == b.distance_fog
        && a.ssao == b.ssao
        && a.shadows == b.shadows
        && a.motion_blur == b.motion_blur
}

pub fn update_graphics_preset(
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    rt_avail: Res<RaytracingAvailable>,
    mut preset: ResMut<GraphicsPreset>,
) {
    if !settings.is_changed() && !dlss_avail.is_changed() && !rt_avail.is_changed() {
        return;
    }
    let new = GraphicsPreset::detect(&settings, dlss_avail.0, rt_avail.0);
    if *preset != new {
        *preset = new;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Per-parameter descriptions
// ────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum Impact {
    /// No measurable cost on this resource.
    None,
    Low,
    Medium,
    High,
}

impl Impact {
    pub fn label(self) -> &'static str {
        match self {
            Impact::None => "none",
            Impact::Low => "low",
            Impact::Medium => "medium",
            Impact::High => "high",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Impact::None => Color::srgb(0.55, 0.60, 0.70),
            Impact::Low => Color::srgb(0.55, 0.85, 0.55),
            Impact::Medium => Color::srgb(0.95, 0.85, 0.45),
            Impact::High => Color::srgb(0.95, 0.55, 0.45),
        }
    }
}

pub struct ParamDescription {
    pub title: &'static str,
    pub functional: &'static str,
    pub technical: &'static str,
    pub cpu: Impact,
    pub gpu: Impact,
    pub ram: Impact,
    pub vram: Impact,
}

pub enum DescriptionKind {
    Param(ParamDescription),
    Preset {
        title: &'static str,
        functional: &'static str,
        technical: &'static str,
    },
    None,
}

/// Description block for the menu slot at `menu_index` within `tab`.
pub fn description_for(
    tab: SettingsTab,
    menu_index: usize,
    preset: GraphicsPreset,
    settings: &GameSettings,
) -> DescriptionKind {
    let slots = tab_slots(tab, settings);
    let Some(slot) = slots.get(menu_index) else {
        return DescriptionKind::None;
    };
    match slot {
        MenuSlot::Preset => DescriptionKind::Preset {
            title: "Graphics preset",
            functional: concat!(
                "Apply a coherent quality bundle in one click.\n",
                "Low: everything off, max FPS. Medium: comfortable visuals. ",
                "High: rich visuals. Ultra: every effect on (RT + DLSS when available).\n",
                "Switches to Custom as soon as any graphics parameter diverges from the chosen preset."
            ),
            technical: match preset {
                GraphicsPreset::Low => "Active: Low. No optional effect, cheapest render path.",
                GraphicsPreset::Medium => "Active: Medium. Bloom + distance fog enabled.",
                GraphicsPreset::High => "Active: High. TAA + atmosphere + bloom + distance fog.",
                GraphicsPreset::Ultra => {
                    "Active: Ultra. Raytracing + DLSS (if supported), volumetric fog and full post-process."
                }
                GraphicsPreset::Custom => {
                    "Active: Custom (current values do not match any preset)."
                }
            },
        },
        MenuSlot::Param(id) => DescriptionKind::Param(param_description(*id)),
        MenuSlot::Back => DescriptionKind::None,
    }
}

pub fn param_description(id: ParamId) -> ParamDescription {
    match id {
        ParamId::Fullscreen => ParamDescription {
            title: "Fullscreen",
            functional: concat!(
                "Switches between windowed and borderless fullscreen.\n",
                "Fullscreen covers the whole display with no window decorations."
            ),
            technical: concat!(
                "Bevy swaps WindowMode between Windowed and BorderlessFullscreen.\n",
                "The compositor can fast-path frames, often reducing input latency."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::VSync => ParamDescription {
            title: "VSync",
            functional: concat!(
                "Synchronises frame presentation with the monitor refresh rate.\n",
                "Eliminates tearing at the cost of mild input lag and a refresh-rate frame cap."
            ),
            technical: concat!(
                "Switches PresentMode between AutoVsync (FIFO) and AutoNoVsync (Mailbox/Immediate).\n",
                "Frames wait for the next VBLANK before being presented to the swapchain."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Hdr => ParamDescription {
            title: "HDR",
            functional: concat!(
                "Renders the scene to a high-dynamic-range buffer for better\n",
                "highlight preservation and richer bloom/tonemapping fidelity."
            ),
            technical: concat!(
                "Toggles Camera::hdr, switching the main render target from\n",
                "Rgba8 (SDR) to Rgba16Float (HDR). Required for proper bloom."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::Low,
        },
        ParamId::Tonemapping => ParamDescription {
            title: "Tonemapping",
            functional: concat!(
                "Maps the HDR render to displayable SDR colours.\n",
                "ACES = cinematic, Tony McMapface = neutral, Reinhard = simple, None = raw HDR."
            ),
            technical: concat!(
                "3D LUT or analytic curve applied as a post-process pass,\n",
                "after bloom and before swapchain present."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Raytracing => ParamDescription {
            title: "Raytracing (Solari)",
            functional: concat!(
                "Global illumination, shadows and reflections evaluated by ray tracing.\n",
                "Indirect light bounces and contact shadows look physically accurate."
            ),
            technical: concat!(
                "Bevy Solari: ReSTIR DI/GI on GPU, requires an RTX-class card.\n",
                "Builds a BVH (RaytracingMesh3d) and forces the deferred + prepass pipeline."
            ),
            cpu: Impact::Low,
            gpu: Impact::High,
            ram: Impact::Medium,
            vram: Impact::High,
        },
        ParamId::Dlss => ParamDescription {
            title: "DLSS",
            functional: concat!(
                "AI upscaler: renders the scene at a lower internal resolution then\n",
                "reconstructs the native-resolution image, usually a big FPS win.\n",
                "Shows \"N/A\" when unavailable — either the GPU isn't an RTX card,\n",
                "or the build is shipping the default mock (NVIDIA NGX SDK not linked).\n",
                "Rebuild with `--no-default-features --features raytracing,dlss` for real DLSS."
            ),
            technical: concat!(
                "NVIDIA NGX Ray Reconstruction on RTX GPUs.\n",
                "Requires motion vectors + depth prepass; runs on the Tensor cores."
            ),
            cpu: Impact::None,
            // Net GPU cost is negative (DLSS saves more than it adds), so we
            // report Low to convey "extra work is small".
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::Taa => ParamDescription {
            title: "TAA",
            functional: concat!(
                "Temporal anti-aliasing: smooths edges by accumulating several\n",
                "subpixel-jittered frames over time."
            ),
            technical: concat!(
                "Reprojection via motion vectors + history buffer in VRAM.\n",
                "Requires MotionVectorPrepass and a jittered camera; MSAA must stay off."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::Bloom => ParamDescription {
            title: "Bloom",
            functional: concat!(
                "Soft halo around very bright pixels (sun, flames).\n",
                "Sells the impression of HDR luminance and contrast."
            ),
            technical: concat!(
                "Mip down/up chain + additive blend in post-process.\n",
                "Reads the HDR framebuffer; cost scales with screen resolution."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Atmosphere => ParamDescription {
            title: "Atmosphere",
            functional: concat!(
                "Physical atmospheric scattering: sky colour, distant haze\n",
                "and sun colour that follows its elevation angle."
            ),
            technical: concat!(
                "Pre-computed LUTs (transmittance, sky-view, multiscattering)\n",
                "plus Rayleigh + Mie integral over scene depth."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::VolumetricFog => ParamDescription {
            title: "Volumetric fog",
            functional: concat!(
                "Fog volumes crossed by the directional light produce visible\n",
                "rays of light (god rays)."
            ),
            technical: concat!(
                "Raymarching in a 3D froxel grid (frustum voxels).\n",
                "Requires VolumetricLight on the sun and a FogVolume in the scene."
            ),
            cpu: Impact::None,
            gpu: Impact::High,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::DistanceFog => ParamDescription {
            title: "Distance fog",
            functional: concat!(
                "Fades far-away objects into a fog tint that grows\n",
                "with camera distance."
            ),
            technical: concat!(
                "Exponential-squared mix evaluated in the fragment shader.\n",
                "Adds a couple of ALU ops per pixel, no extra buffer."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Msaa => ParamDescription {
            title: "MSAA",
            functional: concat!(
                "Multi-sample anti-aliasing: super-samples geometry edges\n",
                "using 2/4/8 hardware samples per pixel.",
                "\nIgnored when Raytracing or TAA force the deferred pipeline."
            ),
            technical: concat!(
                "GPU-resolved multisampled render target.\n",
                "Bandwidth + memory scale linearly with the sample count."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::Fxaa => ParamDescription {
            title: "FXAA",
            functional: concat!(
                "Fast approximate anti-aliasing: a cheap post-process pass\n",
                "that softens jagged edges from luma contrast."
            ),
            technical: concat!(
                "Single fullscreen pass over the LDR/HDR colour buffer.\n",
                "Trades some sharpness for an O(1) screen-space cost."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Ssao => ParamDescription {
            title: "SSAO",
            functional: concat!(
                "Screen-space ambient occlusion: darkens crevices and contact\n",
                "points where indirect light would be blocked."
            ),
            technical: concat!(
                "GTAO-style sampling in screen space. Requires depth +\n",
                "normal prepasses; cost scales with the quality preset."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Low,
        },
        ParamId::Shadows => ParamDescription {
            title: "Shadows",
            functional: concat!(
                "Cascaded shadow maps cast by the sun.\n",
                "Off means flat lighting with no occlusion from the sun."
            ),
            technical: concat!(
                "Multiple shadow cascades rendered each frame and sampled\n",
                "in the PBR shader. Cost scales with cascade resolution + count."
            ),
            cpu: Impact::Low,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::Medium,
        },
        ParamId::MotionBlur => ParamDescription {
            title: "Motion blur",
            functional: concat!(
                "Smears moving objects in the direction of their motion,\n",
                "selling speed and reducing judder at low frame rates."
            ),
            technical: concat!(
                "Per-pixel directional blur driven by motion vectors.\n",
                "Requires Depth + MotionVector prepasses."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::Low,
        },
        ParamId::Exposure => ParamDescription {
            title: "Exposure",
            functional: concat!(
                "Selects how bright the HDR scene appears after tonemapping.\n",
                "Cycles Low / Default / High (EV100 11 / 13 / 15)."
            ),
            technical: concat!(
                "Updates the Exposure component (ev100) on the camera.\n",
                "Acts as a multiplier on linear light before tonemapping."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::BloomIntensity => ParamDescription {
            title: "Bloom intensity",
            functional: concat!(
                "Controls the visible amount of bloom glow.\n",
                "Cycles Low / Default / High."
            ),
            technical: concat!(
                "Scales the `intensity` field of the Bloom post-process.\n",
                "Same shader cost regardless of intensity."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::DlssQuality => ParamDescription {
            title: "DLSS quality",
            functional: concat!(
                "DLSS render preset: trades internal resolution for FPS.\n",
                "Performance < Balanced < Quality < DLAA < Auto (driver-picked)."
            ),
            technical: concat!(
                "Maps to DlssPerfQualityMode. Lower quality renders the scene\n",
                "at a smaller internal resolution then upscales with the NN."
            ),
            cpu: Impact::None,
            gpu: Impact::Low,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::SsaoQuality => ParamDescription {
            title: "SSAO quality",
            functional: concat!(
                "Number of samples used by SSAO per pixel.\n",
                "Cycles Low / Medium / High / Ultra (4 / 8 / 18 / 54 samples)."
            ),
            technical: concat!(
                "Maps to ScreenSpaceAmbientOcclusionQualityLevel.\n",
                "Cost grows roughly linearly with sample count."
            ),
            cpu: Impact::None,
            gpu: Impact::Medium,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::FpsCap => ParamDescription {
            title: "FPS cap",
            functional: concat!(
                "Caps the maximum number of frames rendered per second.\n",
                "Cycles Unlimited / 30 / 60 / 120 / 144 / 240."
            ),
            technical: concat!(
                "End-of-frame sleep matched to the target frame interval.\n",
                "Independent from VSync; useful to reduce GPU heat or noise."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::FogDensity => ParamDescription {
            title: "Fog density",
            functional: concat!(
                "Thickness of the volumetric fog volume.\n",
                "Higher density = more visible god rays, more atmospheric depth."
            ),
            technical: concat!(
                "Scales the ambient intensity / scattering density inside the\n",
                "VolumetricFog component. Cost stays the same."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
        ParamId::Colorblind => ParamDescription {
            title: "Colorblind palette",
            functional: concat!(
                "Swaps the Right side from red to orange so the two sides remain\n",
                "easy to tell apart for deuteranopia / protanopia.\n",
                "Only affects in-world units, towers and bases — UI accents stay\n",
                "in the default palette."
            ),
            technical: concat!(
                "Re-tints the shared StandardMaterials in MatLibrary. Existing\n",
                "entities pick up the new colour on the next frame; no respawn."
            ),
            cpu: Impact::None,
            gpu: Impact::None,
            ram: Impact::None,
            vram: Impact::None,
        },
    }
}

/// Resolve a short human-readable label for a single parameter, suitable as
/// the button text. Sub-parameters use a `   - ` indent prefix so the
/// hierarchy reads at a glance.
pub fn param_label(
    id: ParamId,
    s: &GameSettings,
    dlss_supported: bool,
    rt_supported: bool,
) -> String {
    match id {
        ParamId::Fullscreen => format!("Fullscreen: {}", on_off(s.fullscreen)),
        ParamId::VSync => format!("VSync: {}", on_off(s.vsync)),
        ParamId::Msaa => format!("MSAA: {}", msaa_label(s.msaa)),
        ParamId::Hdr => format!("HDR: {}", on_off(s.hdr)),
        ParamId::Exposure => format!("  - Exposure: {}", exposure_label(s.exposure)),
        ParamId::Tonemapping => format!("Tonemapping: {}", tonemapping_label(s.tonemapping)),
        ParamId::FpsCap => format!("FPS cap: {}", fps_cap_label(s.fps_cap)),
        ParamId::Colorblind => format!("Colorblind palette: {}", on_off(s.colorblind)),
        ParamId::Raytracing => {
            if cfg!(feature = "raytracing") && rt_supported {
                format!("Raytracing (Solari): {}", on_off(s.raytracing))
            } else {
                "Raytracing (Solari): N/A".into()
            }
        }
        ParamId::Dlss => {
            if cfg!(feature = "dlss") && dlss_supported {
                format!("DLSS: {}", on_off(s.dlss))
            } else {
                "DLSS: N/A".into()
            }
        }
        ParamId::DlssQuality => format!("  - DLSS quality: {}", dlss_quality_label(s.dlss_quality)),
        ParamId::Taa => format!("TAA: {}", on_off(s.taa)),
        ParamId::Fxaa => format!("FXAA: {}", on_off(s.fxaa)),
        ParamId::Bloom => format!("Bloom: {}", on_off(s.bloom)),
        ParamId::BloomIntensity => {
            format!("  - Bloom intensity: {}", level_label(s.bloom_intensity))
        }
        ParamId::Atmosphere => format!("Atmosphere: {}", on_off(s.atmosphere)),
        ParamId::VolumetricFog => format!("Volumetric fog: {}", on_off(s.volumetric_fog)),
        ParamId::FogDensity => format!("  - Fog density: {}", level_label(s.fog_density)),
        ParamId::DistanceFog => format!("Distance fog: {}", on_off(s.distance_fog)),
        ParamId::Ssao => format!("SSAO: {}", on_off(s.ssao)),
        ParamId::SsaoQuality => format!("  - SSAO quality: {}", ssao_quality_label(s.ssao_quality)),
        ParamId::Shadows => format!("Shadows: {}", on_off(s.shadows)),
        ParamId::MotionBlur => format!("Motion blur: {}", on_off(s.motion_blur)),
    }
}

fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

pub fn fps_cap_label(idx: u8) -> &'static str {
    match idx {
        1 => "30",
        2 => "60",
        3 => "120",
        4 => "144",
        5 => "240",
        _ => "Unlimited",
    }
}

/// Target FPS, or `None` for unlimited.
pub fn fps_cap_value(idx: u8) -> Option<u32> {
    match idx {
        1 => Some(30),
        2 => Some(60),
        3 => Some(120),
        4 => Some(144),
        5 => Some(240),
        _ => None,
    }
}

pub fn tonemapping_label(idx: u8) -> &'static str {
    match idx {
        0 => "ACES Fitted",
        1 => "Tony McMapface",
        2 => "Reinhard",
        _ => "None",
    }
}

pub fn msaa_label(idx: u8) -> &'static str {
    match idx {
        2 => "2x",
        4 => "4x",
        8 => "8x",
        _ => "OFF",
    }
}

pub fn exposure_label(idx: u8) -> &'static str {
    match idx {
        0 => "Low",
        2 => "High",
        _ => "Default",
    }
}

/// EV100 value matching `exposure_label`.
pub fn exposure_ev100(idx: u8) -> f32 {
    match idx {
        0 => 11.0,
        2 => 15.0,
        _ => 13.0,
    }
}

pub fn level_label(idx: u8) -> &'static str {
    match idx {
        0 => "Low",
        2 => "High",
        _ => "Default",
    }
}

/// Bloom intensity multiplier matching `level_label`.
pub fn bloom_intensity_value(idx: u8) -> f32 {
    match idx {
        0 => 0.06,
        2 => 0.30,
        _ => 0.15, // Bloom::NATURAL default
    }
}

/// Volumetric-fog ambient intensity matching `level_label`.
pub fn fog_density_value(idx: u8) -> f32 {
    match idx {
        0 => 0.02,
        2 => 0.12,
        _ => 0.05,
    }
}

pub fn dlss_quality_label(idx: u8) -> &'static str {
    match idx {
        0 => "Performance",
        1 => "Balanced",
        2 => "Quality",
        3 => "DLAA",
        _ => "Auto",
    }
}

pub fn ssao_quality_label(idx: u8) -> &'static str {
    match idx {
        0 => "Low",
        1 => "Medium",
        3 => "Ultra",
        _ => "High",
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Persistence
// ────────────────────────────────────────────────────────────────────────────

fn settings_path() -> Option<PathBuf> {
    let dir = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        return None;
    };
    Some(dir.join("attack_tower").join("settings.cfg"))
}

pub fn load_settings() -> GameSettings {
    let mut s = GameSettings::default();
    let Some(path) = settings_path() else {
        return s;
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return s;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim();
        let v = v.trim();
        match k {
            "fullscreen" => s.fullscreen = parse_bool(v).unwrap_or(s.fullscreen),
            "vsync" => s.vsync = parse_bool(v).unwrap_or(s.vsync),
            "hdr" => s.hdr = parse_bool(v).unwrap_or(s.hdr),
            "msaa" => s.msaa = v.parse().unwrap_or(s.msaa),
            "tonemapping" => s.tonemapping = v.parse().unwrap_or(s.tonemapping),
            "fps_cap" => s.fps_cap = v.parse().unwrap_or(s.fps_cap),
            "raytracing" => s.raytracing = parse_bool(v).unwrap_or(s.raytracing),
            "dlss" => s.dlss = parse_bool(v).unwrap_or(s.dlss),
            "taa" => s.taa = parse_bool(v).unwrap_or(s.taa),
            "fxaa" => s.fxaa = parse_bool(v).unwrap_or(s.fxaa),
            "bloom" => s.bloom = parse_bool(v).unwrap_or(s.bloom),
            "atmosphere" => s.atmosphere = parse_bool(v).unwrap_or(s.atmosphere),
            "volumetric_fog" => s.volumetric_fog = parse_bool(v).unwrap_or(s.volumetric_fog),
            "distance_fog" => s.distance_fog = parse_bool(v).unwrap_or(s.distance_fog),
            "ssao" => s.ssao = parse_bool(v).unwrap_or(s.ssao),
            "shadows" => s.shadows = parse_bool(v).unwrap_or(s.shadows),
            "motion_blur" => s.motion_blur = parse_bool(v).unwrap_or(s.motion_blur),
            "colorblind" => s.colorblind = parse_bool(v).unwrap_or(s.colorblind),
            "exposure" => s.exposure = v.parse().unwrap_or(s.exposure),
            "bloom_intensity" => s.bloom_intensity = v.parse().unwrap_or(s.bloom_intensity),
            "dlss_quality" => s.dlss_quality = v.parse().unwrap_or(s.dlss_quality),
            "ssao_quality" => s.ssao_quality = v.parse().unwrap_or(s.ssao_quality),
            "fog_density" => s.fog_density = v.parse().unwrap_or(s.fog_density),
            _ => {}
        }
    }
    // Single source of truth for feature gates and value clamps. Hardware
    // availability isn't probed yet at load time, so pass `true` for both and
    // let the caller (main) / `enforce_settings_invariants` re-sanitize once
    // the real support flags are known.
    sanitize_settings(&mut s, true, true);
    s
}

pub fn save_settings(s: &GameSettings) {
    let Some(path) = settings_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let text = format!(
        "fullscreen = {}\nvsync = {}\nhdr = {}\nmsaa = {}\ntonemapping = {}\nfps_cap = {}\n\
         raytracing = {}\ndlss = {}\ntaa = {}\nfxaa = {}\nbloom = {}\n\
         atmosphere = {}\nvolumetric_fog = {}\ndistance_fog = {}\nssao = {}\n\
         shadows = {}\nmotion_blur = {}\ncolorblind = {}\n\
         exposure = {}\nbloom_intensity = {}\ndlss_quality = {}\nssao_quality = {}\nfog_density = {}\n",
        s.fullscreen,
        s.vsync,
        s.hdr,
        s.msaa,
        s.tonemapping,
        s.fps_cap,
        s.raytracing,
        s.dlss,
        s.taa,
        s.fxaa,
        s.bloom,
        s.atmosphere,
        s.volumetric_fog,
        s.distance_fog,
        s.ssao,
        s.shadows,
        s.motion_blur,
        s.colorblind,
        s.exposure,
        s.bloom_intensity,
        s.dlss_quality,
        s.ssao_quality,
        s.fog_density,
    );
    let _ = std::fs::write(&path, text);
}

fn parse_bool(v: &str) -> Option<bool> {
    match v {
        "true" | "1" | "on" | "yes" => Some(true),
        "false" | "0" | "off" | "no" => Some(false),
        _ => None,
    }
}

/// Enforce dependencies between graphics settings so we never end up with a
/// combination that crashes the renderer. Called at load time, at every
/// toggle and whenever hardware availability changes.
///
/// Rules:
/// - Build/hardware off → feature off (raytracing, dlss).
/// - DLSS implies Raytracing (Solari ray reconstruction lives on the
///   deferred + RT path).
/// - Raytracing / DLSS / TAA require the HDR (Rgba16Float) main texture:
///   STORAGE_BINDING and HDR-aware accumulation are only valid there.
///   Without it, wgpu rejects the texture creation and the app panics.
/// - Range/value clamps for every multi-value parameter.
pub fn sanitize_settings(s: &mut GameSettings, dlss_supported: bool, rt_supported: bool) {
    if !cfg!(feature = "raytracing") || !rt_supported {
        s.raytracing = false;
    }
    if !cfg!(feature = "dlss") || !dlss_supported {
        s.dlss = false;
    }
    if s.dlss {
        s.raytracing = true;
    }
    if s.raytracing || s.dlss || s.taa {
        s.hdr = true;
    }
    if s.tonemapping > 3 {
        s.tonemapping = 0;
    }
    if !matches!(s.msaa, 0 | 2 | 4 | 8) {
        s.msaa = 0;
    }
    if s.fps_cap > 5 {
        s.fps_cap = 0;
    }
    if s.exposure > 2 {
        s.exposure = 1;
    }
    if s.bloom_intensity > 2 {
        s.bloom_intensity = 1;
    }
    if s.dlss_quality > 4 {
        s.dlss_quality = 4;
    }
    if s.ssao_quality > 3 {
        s.ssao_quality = 2;
    }
    if s.fog_density > 2 {
        s.fog_density = 1;
    }
}

/// Re-runs `sanitize_settings` whenever the user-tweakable settings or the
/// runtime hardware-availability flags change. Catches both the "DLSS just
/// got detected, apply the saved-on value" case and the "user toggled HDR
/// off, drop the dependents" case. Persistence is handled separately by
/// `persist_settings`, so this system is allowed to write through.
pub fn enforce_settings_invariants(
    mut settings: ResMut<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    rt_avail: Res<RaytracingAvailable>,
) {
    if !settings.is_changed() && !dlss_avail.is_changed() && !rt_avail.is_changed() {
        return;
    }
    let mut next = *settings;
    sanitize_settings(&mut next, dlss_avail.0, rt_avail.0);
    if next != *settings {
        *settings = next;
    }
}

/// End-of-frame sleep that caps the framerate at `settings.fps_cap`. Runs
/// last in the schedule so we measure the full frame time and pad it out
/// before yielding to the next iteration. Uses `thread::sleep` for the bulk
/// of the wait, then a short spin on the final ~1ms so the OS quantum
/// (1–15 ms on most platforms) doesn't undershoot the target FPS.
pub fn limit_fps(settings: Res<GameSettings>, mut last: Local<Option<std::time::Instant>>) {
    let Some(target) = fps_cap_value(settings.fps_cap) else {
        *last = Some(std::time::Instant::now());
        return;
    };
    let target_dt = std::time::Duration::from_secs_f64(1.0 / target as f64);
    if let Some(prev) = *last {
        let deadline = prev + target_dt;
        let spin_window = std::time::Duration::from_millis(1);
        let now = std::time::Instant::now();
        if now < deadline {
            let remaining = deadline - now;
            if remaining > spin_window {
                std::thread::sleep(remaining - spin_window);
            }
            while std::time::Instant::now() < deadline {
                std::hint::spin_loop();
            }
        }
    }
    *last = Some(std::time::Instant::now());
}

pub fn persist_settings(settings: Res<GameSettings>, mut started: Local<bool>) {
    if !*started {
        *started = true;
        return;
    }
    if !settings.is_changed() {
        return;
    }
    save_settings(&settings);
}

// ────────────────────────────────────────────────────────────────────────────
// Applying settings to the world (camera components, window, sun, materials).
// ────────────────────────────────────────────────────────────────────────────

pub fn apply_graphics_settings(
    settings: Res<GameSettings>,
    atmo: Res<AtmosphereHandle>,
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    mut tonemap: Query<&mut Tonemapping>,
    mut exposures: Query<&mut Exposure, With<Camera3d>>,
    mut sun: Query<&mut DirectionalLight, With<Sun>>,
    mut windows: Query<&mut Window>,
    // Cached copy of the last fully-applied settings. We only touch the camera
    // components whose underlying fields actually moved, instead of reinserting
    // a dozen renderer features on every settings change.
    mut last_applied: Local<Option<GameSettings>>,
) {
    if !settings.is_changed() {
        return;
    }
    let first = last_applied.is_none();
    let prev = last_applied.unwrap_or(*settings);
    let curr = *settings;
    let changed_any = |fields: &[bool]| first || fields.iter().any(|b| *b);

    // Window mode + vsync.
    if first || curr.fullscreen != prev.fullscreen || curr.vsync != prev.vsync {
        let mode = if curr.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
        let present = if curr.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        for mut window in &mut windows {
            if window.mode != mode {
                window.mode = mode;
            }
            if window.present_mode != present {
                window.present_mode = present;
            }
        }
    }
    // Per-camera components. Both Solari (raytracing) and TAA force the
    // deferred renderer, which is incompatible with MSAA — Bevy logs a warning
    // every frame the camera setting changes if we'd insert MSAA anyway. Drop
    // it silently in both cases.
    let msaa_changed = first
        || curr.msaa != prev.msaa
        || curr.raytracing != prev.raytracing
        || curr.taa != prev.taa;
    let msaa = if curr.raytracing || curr.taa {
        Msaa::Off
    } else {
        match curr.msaa {
            2 => Msaa::Sample2,
            4 => Msaa::Sample4,
            8 => Msaa::Sample8,
            _ => Msaa::Off,
        }
    };
    let hdr_changed = first || curr.hdr != prev.hdr;
    let bloom_changed =
        first || curr.bloom != prev.bloom || curr.bloom_intensity != prev.bloom_intensity;
    let atmo_changed = first || curr.atmosphere != prev.atmosphere;
    let vfog_changed = changed_any(&[
        curr.volumetric_fog != prev.volumetric_fog,
        curr.fog_density != prev.fog_density,
    ]);
    let dfog_changed = first || curr.distance_fog != prev.distance_fog;
    let taa_changed = first || curr.taa != prev.taa;
    let fxaa_changed = first || curr.fxaa != prev.fxaa;
    let ssao_changed = first || curr.ssao != prev.ssao || curr.ssao_quality != prev.ssao_quality;
    let mblur_changed = first || curr.motion_blur != prev.motion_blur;
    for cam in &cameras {
        let mut e = commands.entity(cam);
        if msaa_changed {
            e.insert(msaa);
        }
        if hdr_changed {
            if curr.hdr {
                e.insert(bevy::render::view::Hdr);
            } else {
                e.remove::<bevy::render::view::Hdr>();
            }
        }
        if bloom_changed {
            if curr.bloom {
                e.insert(Bloom {
                    intensity: bloom_intensity_value(curr.bloom_intensity),
                    ..Bloom::NATURAL
                });
            } else {
                e.remove::<Bloom>();
            }
        }
        if atmo_changed {
            if curr.atmosphere {
                e.insert((
                    Atmosphere::earthlike(atmo.0.clone()),
                    AtmosphereSettings::default(),
                ));
            } else {
                e.remove::<Atmosphere>().remove::<AtmosphereSettings>();
            }
        }
        if vfog_changed {
            if curr.volumetric_fog {
                e.insert(VolumetricFog {
                    ambient_intensity: fog_density_value(curr.fog_density),
                    ..default()
                });
            } else {
                e.remove::<VolumetricFog>();
            }
        }
        if dfog_changed {
            if curr.distance_fog {
                e.insert(DistanceFog {
                    color: Color::srgba(0.55, 0.70, 0.85, 1.0),
                    falloff: FogFalloff::ExponentialSquared { density: 0.012 },
                    ..default()
                });
            } else {
                e.remove::<DistanceFog>();
            }
        }
        if taa_changed {
            if curr.taa {
                e.insert(TemporalAntiAliasing::default());
            } else {
                e.remove::<TemporalAntiAliasing>();
            }
        }
        if fxaa_changed {
            if curr.fxaa {
                e.insert(Fxaa::default());
            } else {
                e.remove::<Fxaa>();
            }
        }
        if ssao_changed {
            if curr.ssao {
                e.insert(ScreenSpaceAmbientOcclusion {
                    quality_level: match curr.ssao_quality {
                        0 => ScreenSpaceAmbientOcclusionQualityLevel::Low,
                        1 => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                        3 => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
                        _ => ScreenSpaceAmbientOcclusionQualityLevel::High,
                    },
                    ..default()
                });
            } else {
                e.remove::<ScreenSpaceAmbientOcclusion>();
            }
        }
        if mblur_changed {
            if curr.motion_blur {
                e.insert(MotionBlur::default());
            } else {
                e.remove::<MotionBlur>();
            }
        }
    }
    // Tonemapping (mutates existing component on the camera).
    if first || curr.tonemapping != prev.tonemapping {
        for mut t in &mut tonemap {
            *t = match curr.tonemapping {
                0 => Tonemapping::AcesFitted,
                1 => Tonemapping::TonyMcMapface,
                2 => Tonemapping::Reinhard,
                _ => Tonemapping::None,
            };
        }
    }
    // Exposure (HDR sub-parameter; meaningful only when HDR is on but applying
    // is harmless either way).
    if first || curr.exposure != prev.exposure {
        let target_ev100 = exposure_ev100(curr.exposure);
        for mut exp in &mut exposures {
            if (exp.ev100 - target_ev100).abs() > f32::EPSILON {
                exp.ev100 = target_ev100;
            }
        }
    }
    // Sun shadows on/off.
    if first || curr.shadows != prev.shadows {
        for mut light in &mut sun {
            if light.shadows_enabled != curr.shadows {
                light.shadows_enabled = curr.shadows;
            }
        }
    }
    *last_applied = Some(curr);
}

#[cfg(feature = "raytracing")]
pub fn apply_raytracing_setting(
    settings: Res<GameSettings>,
    avail: Res<RaytracingAvailable>,
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    enabled: Query<Entity, With<SolariLighting>>,
) {
    if !settings.is_changed() {
        return;
    }
    // SolariPlugins isn't loaded when the adapter can't support it, so
    // inserting SolariLighting would be a no-op at best and a crash at worst.
    if settings.raytracing && avail.0 {
        for cam in &cameras {
            if enabled.get(cam).is_err() {
                commands.entity(cam).insert((
                    SolariLighting::default(),
                    Msaa::Off,
                    bevy::camera::CameraMainTextureUsages::default()
                        .with(bevy::render::render_resource::TextureUsages::STORAGE_BINDING),
                ));
            }
        }
    } else {
        for e in &enabled {
            commands
                .entity(e)
                .remove::<SolariLighting>()
                .remove::<bevy::camera::CameraMainTextureUsages>();
        }
    }
}

#[cfg(not(feature = "raytracing"))]
pub fn apply_raytracing_setting() {}

#[cfg(all(feature = "dlss", not(feature = "force_disable_dlss")))]
pub fn detect_dlss_support(
    // SR is the broad gate — RR is a strict subset (RR-capable cards also support SR).
    sr_supported: Option<Res<bevy::anti_alias::dlss::DlssSuperResolutionSupported>>,
    mut avail: ResMut<DlssAvailable>,
) {
    let new = sr_supported.is_some();
    if avail.0 != new {
        avail.0 = new;
    }
}

#[cfg(not(all(feature = "dlss", not(feature = "force_disable_dlss"))))]
pub fn detect_dlss_support(_: ResMut<DlssAvailable>) {}

/// Applies the DLSS setting to every camera. Picks Ray Reconstruction when
/// raytracing is on (and supported) — RR is the denoiser variant designed to
/// pair with Solari. Falls back to Super Resolution otherwise. Removes TAA
/// when DLSS is active since the two are mutually exclusive.
#[cfg(all(feature = "dlss", not(feature = "force_disable_dlss")))]
pub fn apply_dlss_setting(
    settings: Res<GameSettings>,
    avail: Res<DlssAvailable>,
    rr_supported: Option<Res<bevy::anti_alias::dlss::DlssRayReconstructionSupported>>,
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
) {
    use bevy::anti_alias::dlss::{
        Dlss, DlssPerfQualityMode, DlssRayReconstructionFeature, DlssSuperResolutionFeature,
    };
    use bevy::anti_alias::taa::TemporalAntiAliasing;

    if !settings.is_changed() && !avail.is_changed() {
        return;
    }
    let enabled = settings.dlss && avail.0;
    let use_rr = enabled && settings.raytracing && rr_supported.is_some();
    let mode = match settings.dlss_quality {
        0 => DlssPerfQualityMode::Performance,
        1 => DlssPerfQualityMode::Balanced,
        2 => DlssPerfQualityMode::Quality,
        3 => DlssPerfQualityMode::Dlaa,
        _ => DlssPerfQualityMode::Auto,
    };
    for cam in &cameras {
        let mut e = commands.entity(cam);
        if enabled {
            e.remove::<TemporalAntiAliasing>().insert(Msaa::Off);
            if use_rr {
                e.remove::<Dlss<DlssSuperResolutionFeature>>()
                    .insert(Dlss::<DlssRayReconstructionFeature> {
                        perf_quality_mode: mode,
                        reset: false,
                        _phantom_data: core::marker::PhantomData,
                    });
            } else {
                e.remove::<Dlss<DlssRayReconstructionFeature>>()
                    .insert(Dlss::<DlssSuperResolutionFeature> {
                        perf_quality_mode: mode,
                        reset: false,
                        _phantom_data: core::marker::PhantomData,
                    });
            }
        } else {
            e.remove::<Dlss<DlssSuperResolutionFeature>>()
                .remove::<Dlss<DlssRayReconstructionFeature>>();
        }
    }
}

#[cfg(not(all(feature = "dlss", not(feature = "force_disable_dlss"))))]
pub fn apply_dlss_setting() {}

/// Mutate the shared side colour materials whenever the colorblind toggle
/// flips, so every entity that references them (units, towers, castle accents,
/// arrows) picks up the new palette without a respawn.
pub fn apply_colorblind_palette(
    settings: Res<GameSettings>,
    lib: Res<MatLibrary>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.is_changed() {
        return;
    }
    let cb = settings.colorblind;
    for (handle, color) in [
        (&lib.left, Side::Left.color_for(cb)),
        (&lib.right, Side::Right.color_for(cb)),
        (&lib.left_dark, Side::Left.color_dark_for(cb)),
        (&lib.right_dark, Side::Right.color_dark_for(cb)),
    ] {
        if let Some(mat) = materials.get_mut(handle) {
            mat.base_color = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_accepts_common_spellings() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("no"), Some(false));
        assert_eq!(parse_bool("bogus"), None);
    }

    #[test]
    fn fps_cap_label_and_value_agree() {
        assert_eq!(fps_cap_value(0), None);
        assert_eq!(fps_cap_value(2), Some(60));
        assert_eq!(fps_cap_label(2), "60");
        assert_eq!(fps_cap_label(99), "Unlimited");
    }

    #[test]
    fn preset_apply_and_detect_round_trip() {
        // After applying a preset, detect should return that same preset
        // (modulo gating on raytracing/dlss support).
        for preset in [
            GraphicsPreset::Low,
            GraphicsPreset::Medium,
            GraphicsPreset::High,
        ] {
            let mut s = GameSettings::default();
            preset.apply(&mut s, false, false);
            assert_eq!(GraphicsPreset::detect(&s, false, false), preset);
        }
    }

    #[test]
    fn detect_returns_custom_when_no_preset_matches() {
        let mut s = GameSettings::default();
        GraphicsPreset::Low.apply(&mut s, false, false);
        s.bloom = true; // diverges from Low
        assert_eq!(
            GraphicsPreset::detect(&s, false, false),
            GraphicsPreset::Custom
        );
    }

    #[test]
    fn sanitize_drops_raytracing_when_unsupported() {
        let mut s = GameSettings {
            raytracing: true,
            dlss: true,
            ..GameSettings::default()
        };
        sanitize_settings(&mut s, false, false);
        assert!(!s.raytracing);
        assert!(!s.dlss);
    }

    #[test]
    fn sanitize_clamps_out_of_range_indices() {
        let mut s = GameSettings {
            tonemapping: 99,
            fps_cap: 99,
            msaa: 99,
            exposure: 99,
            bloom_intensity: 99,
            dlss_quality: 99,
            ssao_quality: 99,
            fog_density: 99,
            ..GameSettings::default()
        };
        sanitize_settings(&mut s, false, false);
        assert!(s.tonemapping <= 3);
        assert!(s.fps_cap <= 5);
        assert!(matches!(s.msaa, 0 | 2 | 4 | 8));
        assert!(s.exposure <= 2);
        assert!(s.bloom_intensity <= 2);
        assert!(s.dlss_quality <= 4);
        assert!(s.ssao_quality <= 3);
        assert!(s.fog_density <= 2);
    }
}
