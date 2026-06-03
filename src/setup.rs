use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::light_consts::lux;
use bevy::light::{
    CascadeShadowConfigBuilder, FogVolume, NotShadowCaster, VolumetricFog, VolumetricLight,
};
use bevy::pbr::{Atmosphere, AtmosphereSettings, DistanceFog, FogFalloff, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
#[cfg(feature = "raytracing")]
use bevy::solari::prelude::{RaytracingMesh3d, SolariLighting};

use crate::common::*;

pub fn init_mat_library(
    mut lib: ResMut<MatLibrary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    lib.left = materials.add(StandardMaterial {
        base_color: Side::Left.color(),
        perceptual_roughness: 0.7,
        ..default()
    });
    lib.right = materials.add(StandardMaterial {
        base_color: Side::Right.color(),
        perceptual_roughness: 0.7,
        ..default()
    });
    lib.left_dark = materials.add(StandardMaterial {
        base_color: Side::Left.color_dark(),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.right_dark = materials.add(StandardMaterial {
        base_color: Side::Right.color_dark(),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.eye_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.04, 0.04, 0.06),
        perceptual_roughness: 0.4,
        ..default()
    });
    lib.ground = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.45, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.26, 0.13),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.metal_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.74, 0.78),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        ..default()
    });
    lib.stone_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.76, 0.70),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.stone_dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.52, 0.48),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.43, 0.40),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.62, 0.24),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.bush_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.48, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    lib.flower_red_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.30, 0.30),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.flower_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.85, 0.25),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.flower_violet_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.40, 0.85),
        perceptual_roughness: 0.85,
        ..default()
    });
    lib.flame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.65, 0.25),
        // Strong emissive so the mesh acts as a light source under Solari
        // (raytraced lighting ignores PointLight, only DirectionalLight +
        // emissive meshes contribute).
        //
        // NOTE: `unlit` is intentionally OFF. Bevy 0.18's deferred path
        // (forced by Solari) rewrites the G-buffer emissive to base_color
        // for unlit materials (see pbr_deferred_functions.wgsl), which
        // discards our high emissive value and leaves the flame both
        // visually dim and unable to inject light through GI bounces.
        // Keeping it lit means the cone receives a tiny bit of ambient
        // lighting too, but at this emissive level it stays clearly glowing.
        //
        // Emissive is pumped 8× higher than visually needed: Solari's ReSTIR
        // DI picks emissive triangles by area, so a tiny cone is under-sampled
        // unless its per-area radiance is very high.
        emissive: LinearRgba::rgb(480.0, 220.0, 60.0),
        ..default()
    });
    lib.flame_mesh = meshes.add(Cone::new(0.14, 0.32));
    lib.torch_pole_mesh = meshes.add(Cylinder::new(0.025, 0.30));

    lib.body_mesh = meshes.add(Capsule3d::new(0.20, 0.28));
    lib.head_mesh = meshes.add(Sphere::new(0.17));
    lib.limb_mesh = meshes.add(Cylinder::new(0.085, 0.36));
    lib.eye_mesh = meshes.add(Sphere::new(0.035));

    lib.spear_shaft = meshes.add(Cylinder::new(0.025, 0.85));
    lib.spear_tip = meshes.add(Cone::new(0.06, 0.18));
    lib.pickaxe_handle = meshes.add(Cylinder::new(0.025, 0.55));
    lib.pickaxe_head = meshes.add(Cuboid::new(0.34, 0.07, 0.07));
    lib.bow_limb = meshes.add(Cylinder::new(0.035, 0.36));
    lib.bow_string = meshes.add(Cylinder::new(0.010, 0.66));
    lib.arrow_shaft = meshes.add(Cylinder::new(0.014, 0.55));
    lib.arrow_tip = meshes.add(Cone::new(0.040, 0.10));
    lib.arrow_fletch = meshes.add(Cuboid::new(0.01, 0.08, 0.07));

    lib.grass_blade = meshes.add(Cone::new(0.045, 0.22));
    lib.bush_mesh = meshes.add(Sphere::new(0.22));
    lib.plant_stem = meshes.add(Cylinder::new(0.012, 0.28));
    lib.plant_flower = meshes.add(Sphere::new(0.065));

    // Tower (assembled from stacked stone primitives).
    lib.tower_foundation = meshes.add(Cuboid::new(1.05, 0.3, 1.05));
    lib.tower_shaft = meshes.add(Cylinder::new(0.42, 1.6));
    lib.tower_top_slab = meshes.add(Cuboid::new(1.15, 0.16, 1.15));
    lib.tower_crenel = meshes.add(Cuboid::new(0.2, 0.22, 0.2));
    lib.tower_roof = meshes.add(Cone::new(0.55, 0.55));

    // Ghost preview: vertical cylinder shown at the cursor during placement.
    lib.tower_ghost_mesh = meshes.add(Cylinder::new(0.55, TOWER_HEIGHT));
    lib.ghost_valid_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.30, 1.0, 0.45, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    lib.ghost_invalid_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.30, 0.30, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    // Thin marker strip painted on the ground at each zone boundary.
    lib.zone_marker_mesh = meshes.add(Cuboid::new(0.12, 0.02, 12.0));
    lib.zone_marker_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.95, 0.95, 0.55),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // Castle pieces — shared across all bases (1v1 and 2v2).
    lib.castle_foundation = meshes.add(Cuboid::new(2.0, 0.4, 2.0));
    lib.castle_keep = meshes.add(Cuboid::new(1.1, 1.2, 1.1));
    lib.castle_top_slab = meshes.add(Cuboid::new(1.3, 0.12, 1.3));
    lib.castle_crenel = meshes.add(Cuboid::new(0.22, 0.22, 0.22));
    lib.castle_tower = meshes.add(Cuboid::new(0.45, 1.6, 0.45));
    lib.castle_roof = meshes.add(Cone::new(0.36, 0.55));
    lib.castle_door = meshes.add(Cuboid::new(0.08, 0.55, 0.36));
    lib.castle_pole = meshes.add(Cylinder::new(0.03, 0.9));
    lib.castle_flag = meshes.add(Cuboid::new(0.34, 0.22, 0.02));

    // Rock — three sphere sizes reused per side.
    lib.rock_large = meshes.add(Sphere::new(0.65));
    lib.rock_medium = meshes.add(Sphere::new(0.42));
    lib.rock_small = meshes.add(Sphere::new(0.36));
}

/// Startup: kick off the async load of the archer scene (mesh + skeleton, from
/// the Walking file) and one `AnimationClip` per action, each from its own
/// single-animation file. The scene is instanced per-archer in `spawn_unit`; the
/// animation graph is built once the clips decode (see `build_archer_graph`).
pub fn load_archer_assets(asset_server: Res<AssetServer>, mut assets: ResMut<ArcherAssets>) {
    let clip =
        |path: &'static str| asset_server.load(GltfAssetLabel::Animation(0).from_asset(path));
    assets.scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(ARCHER_SCENE_PATH));
    assets.walk = clip(ARCHER_WALK_PATH);
    assets.attack = clip(ARCHER_SHOT_PATH);
    assets.hurts = [clip(ARCHER_HURT_PATHS[0]), clip(ARCHER_HURT_PATHS[1])];
    assets.death = clip(ARCHER_DEATH_PATH);
}

/// Update: once every archer clip has decoded, build the `AnimationGraph` and
/// cache the node indices. Waits for the clips so the playback speeds can be
/// derived from their real durations. Runs each frame until built, then no-ops.
pub fn build_archer_graph(
    mut assets: ResMut<ArcherAssets>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    if assets.nodes.is_some() {
        return;
    }
    // Bail (retry next frame) until all clips are loaded.
    let handles = [
        &assets.walk,
        &assets.attack,
        &assets.hurts[0],
        &assets.hurts[1],
        &assets.death,
    ];
    if handles.iter().any(|h| clips.get(*h).is_none()) {
        return;
    }

    let mut graph = AnimationGraph::new();
    let root = graph.root;
    let walk = graph.add_clip(assets.walk.clone(), 1.0, root);
    let attack = graph.add_clip(assets.attack.clone(), 1.0, root);
    let hurts = [
        graph.add_clip(assets.hurts[0].clone(), 1.0, root),
        graph.add_clip(assets.hurts[1].clone(), 1.0, root),
    ];
    let death = graph.add_clip(assets.death.clone(), 1.0, root);

    // Match clip playback to gameplay timing: one shot loop per cooldown, and
    // the fall finishing within the archer's death window.
    let speed_for = |h: &Handle<AnimationClip>, target: f32| {
        clips
            .get(h)
            .map(|c| (c.duration() / target).max(0.01))
            .unwrap_or(1.0)
    };
    let attack_speed = speed_for(&assets.attack, ARCHER_COOLDOWN);
    let death_speed = speed_for(&assets.death, ARCHER_DEATH_DURATION);

    assets.graph = Some(graphs.add(graph));
    assets.nodes = Some(ArcherAnimNodes {
        walk,
        attack,
        hurts,
        death,
        attack_speed,
        death_speed,
    });
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    lib: Res<MatLibrary>,
) {
    let medium = scattering_mediums.add(ScatteringMedium::default());
    commands.insert_resource(AtmosphereHandle(medium.clone()));
    let camera = commands
        .spawn((
            Camera3d::default(),
            // `apply_graphics_settings` keeps `Hdr` in sync with the saved
            // setting; starting with HDR on avoids a one-frame SDR fallback
            // when the persisted config has it enabled.
            bevy::render::view::Hdr,
            Transform::from_xyz(0.0, 20.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
            Atmosphere::earthlike(medium),
            AtmosphereSettings::default(),
            Exposure { ev100: 13.0 },
            Tonemapping::AcesFitted,
            Bloom::NATURAL,
            DistanceFog {
                color: Color::srgba(0.55, 0.70, 0.85, 1.0),
                falloff: FogFalloff::ExponentialSquared { density: 0.012 },
                ..default()
            },
            VolumetricFog {
                ambient_intensity: 0.05,
                ..default()
            },
        ))
        .id();
    // SolariLightingPlugin sets DefaultOpaqueRendererMethod::deferred() globally,
    // so the camera needs the deferred prepass machinery from frame 1, otherwise
    // the first render produces a gray screen (deferred materials with no
    // gbuffer). We add them up front when the feature is enabled.
    #[cfg(feature = "raytracing")]
    commands.entity(camera).insert((
        bevy::core_pipeline::prepass::DeferredPrepass,
        bevy::core_pipeline::prepass::DepthPrepass,
        bevy::core_pipeline::prepass::MotionVectorPrepass,
    ));
    let _ = camera;

    spawn_sun(&mut commands);

    // Fog volume that the sun shines through to produce subtle god rays.
    commands.spawn((
        FogVolume::default(),
        Transform::from_scale(Vec3::new(60.0, 6.0, 30.0))
            .with_translation(Vec3::new(0.0, 3.0, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(80.0, 0.2, 40.0))),
        MeshMaterial3d(lib.ground.clone()),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    spawn_sky(&mut commands, &mut meshes, &mut materials);
    spawn_mountains(&mut commands, &mut meshes, &mut materials);

    spawn_zone_markers(&mut commands, &lib);
    spawn_scenery(&mut commands, &mut meshes, &lib);
}

/// Build bases + rocks for the active GameMode when entering Playing with an
/// empty arena. Despawned on return-to-menu paths so the next match can be
/// rebuilt cleanly for either 1v1 or 2v2. Guarded by `state.is_changed()` so
/// the bases query isn't iterated every frame during a match.
pub fn spawn_arena(
    mut commands: Commands,
    lib: Res<MatLibrary>,
    state: Res<GameState>,
    mode: Res<GameMode>,
    bases: Query<Entity, With<Base>>,
) {
    if !state.is_changed() || *state != GameState::Playing {
        return;
    }
    // Resume from Paused → Playing must keep the existing arena.
    if bases.iter().next().is_some() {
        return;
    }
    for &slot in mode.active_slots() {
        let z = slot.base_z(*mode);
        spawn_castle(&mut commands, &lib, slot, z);
        spawn_rock(&mut commands, &lib, slot, z);
    }
}

fn spawn_zone_markers(commands: &mut Commands, lib: &MatLibrary) {
    for x in [-ZONE_BOUNDARY, ZONE_BOUNDARY] {
        commands.spawn((
            Mesh3d(lib.zone_marker_mesh.clone()),
            MeshMaterial3d(lib.zone_marker_mat.clone()),
            Transform::from_xyz(x, 0.02, 0.0),
        ));
    }
}

fn spawn_sun(commands: &mut Commands) {
    let cascade = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 8.0,
        maximum_distance: 60.0,
        ..default()
    }
    .build();
    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        // Initial transform; animate_sun overwrites it every frame.
        Transform::from_xyz(0.0, SUN_DISTANCE, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        cascade,
        VolumetricLight,
        Sun,
    ));
}

pub fn advance_game_time(time: Res<Time>, state: Res<GameState>, mut gtime: ResMut<GameTime>) {
    if *state == GameState::Playing {
        gtime.0 += time.delta_secs();
    }
}

pub fn animate_sun(
    gtime: Res<GameTime>,
    mut sun: Query<&mut Transform, With<Sun>>,
    mut tod: ResMut<TimeOfDay>,
) {
    // Full day/night arc, behind the camera (+Z half) so the sun lights the
    // camera-facing side of buildings and units instead of backlighting them.
    let angle = (gtime.0 / SUN_DAY_PERIOD) * std::f32::consts::TAU;
    let raw_y = angle.sin();
    let dir = Vec3::new(angle.cos(), raw_y, 0.45).normalize();
    for mut t in &mut sun {
        t.translation = dir * SUN_DISTANCE;
        let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        t.look_at(Vec3::ZERO, up);
    }
    let new_tod = if raw_y > 0.0 {
        TimeOfDay::Day
    } else {
        TimeOfDay::Night
    };
    if *tod != new_tod {
        *tod = new_tod;
    }
}

#[cfg(feature = "raytracing")]
pub fn sync_raytracing_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // Skip entities flagged as not-shadow-casters (e.g. health bars): Solari
    // ignores NotShadowCaster on the rasterized path but still picks up any
    // mesh registered in the BVH for raytraced shadows.
    new: Query<
        (Entity, &Mesh3d),
        (
            Added<Mesh3d>,
            Without<RaytracingMesh3d>,
            Without<NotShadowCaster>,
        ),
    >,
) {
    for (entity, mesh3d) in &new {
        // Solari needs POSITION/NORMAL/UV_0/TANGENT. Bevy primitives don't ship
        // tangents — generate them once per asset on first sight.
        if let Some(mesh) = meshes.get_mut(&mesh3d.0)
            && mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_none()
        {
            let _ = mesh.generate_tangents();
        }
        commands
            .entity(entity)
            .insert(RaytracingMesh3d(mesh3d.0.clone()));
    }
}

#[cfg(not(feature = "raytracing"))]
pub fn sync_raytracing_meshes() {}

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

/// Probes the default adapter for the wgpu features AND limits Solari needs.
/// Run once before `App::new()` so we can skip loading `SolariPlugins` (and
/// stop requesting its features) on machines that would crash at first frame.
/// Some adapters (e.g. AMD Vega) advertise the raytracing feature flag but
/// report `max_blas_geometry_count = 0`, so checking features alone isn't
/// enough — both must be present.
#[cfg(feature = "raytracing")]
pub fn probe_raytracing_support() -> bool {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let Ok(adapter) =
        bevy::tasks::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
    else {
        return false;
    };
    let needs = bevy::solari::prelude::SolariPlugins::required_wgpu_features();
    let limits = adapter.limits();
    adapter.features().contains(needs)
        && limits.max_blas_geometry_count > 0
        && limits.max_tlas_instance_count > 0
}

#[cfg(not(feature = "raytracing"))]
pub fn probe_raytracing_support() -> bool {
    false
}

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

pub fn update_torches(
    tod: Res<TimeOfDay>,
    mut lights: Query<&mut PointLight, With<TorchLight>>,
    mut flames: Query<&mut Visibility, With<TorchFlame>>,
    new_light: Query<Entity, Added<TorchLight>>,
    new_flame: Query<Entity, Added<TorchFlame>>,
) {
    // Run on tod change *or* whenever a fresh torch is spawned (e.g. a newly
    // built tower at night needs to light up right away).
    if !tod.is_changed() && new_light.is_empty() && new_flame.is_empty() {
        return;
    }
    let on = *tod == TimeOfDay::Night;
    for mut light in &mut lights {
        light.intensity = if on { TORCH_INTENSITY } else { 0.0 };
    }
    for mut vis in &mut flames {
        *vis = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn spawn_mountains(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.40, 0.44, 0.50),
        perceptual_roughness: 0.95,
        ..default()
    });
    let cone = meshes.add(Cone::new(1.0, 1.0));

    // (x, z, width, height)
    let peaks: &[(f32, f32, f32, f32)] = &[
        (-28.0, -34.0, 7.0, 7.0),
        (-19.0, -37.0, 9.0, 9.5),
        (-11.0, -33.0, 6.0, 6.5),
        (-3.0, -38.0, 10.0, 11.0),
        (5.0, -34.0, 8.0, 8.5),
        (14.0, -36.0, 9.0, 9.0),
        (24.0, -33.0, 7.0, 7.5),
        (-26.0, 35.0, 8.0, 7.5),
        (-15.0, 37.0, 7.0, 7.0),
        (-5.0, 34.0, 9.0, 8.5),
        (6.0, 36.0, 6.0, 6.0),
        (16.0, 35.0, 9.0, 9.5),
        (26.0, 37.0, 7.0, 8.0),
    ];
    for &(x, z, w, h) in peaks {
        commands.spawn((
            Mesh3d(cone.clone()),
            MeshMaterial3d(rock_mat.clone()),
            Transform {
                translation: Vec3::new(x, h * 0.5 - 0.1, z),
                scale: Vec3::new(w, h, w),
                ..default()
            },
        ));
    }
}

fn spawn_sky(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // A few cloud puffs scattered high overhead.
    let cloud_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.98, 0.98, 0.98),
        unlit: true,
        ..default()
    });
    let cloud_mesh = meshes.add(Sphere::new(1.0));
    for &(x, y, z, sx, sy, sz) in &[
        (-12.0, 14.0, -18.0, 2.4, 1.2, 1.6),
        (8.0, 16.0, -22.0, 3.0, 1.4, 2.0),
        (18.0, 13.0, -12.0, 2.2, 1.0, 1.5),
        (-22.0, 15.0, -8.0, 2.8, 1.3, 1.7),
        (2.0, 18.0, -28.0, 3.4, 1.5, 2.1),
    ] {
        commands.spawn((
            Mesh3d(cloud_mesh.clone()),
            MeshMaterial3d(cloud_mat.clone()),
            Transform {
                translation: Vec3::new(x, y, z),
                scale: Vec3::new(sx, sy, sz),
                ..default()
            },
            NotShadowCaster,
        ));
    }
}

fn spawn_scenery(commands: &mut Commands, meshes: &mut Assets<Mesh>, lib: &MatLibrary) {
    spawn_trees(commands, meshes, lib);
    // Grass tufts: cone clusters. Placed outside the central walking strip
    // (|z| > 1.4) and away from base/rock footprints.
    let grass_spots: &[(f32, f32)] = &[
        (-13.0, 2.0),
        (-11.5, 3.4),
        (-9.0, -2.6),
        (-6.5, 2.8),
        (-4.0, -3.2),
        (-2.0, 3.1),
        (0.5, -2.4),
        (3.0, 2.7),
        (5.5, -3.5),
        (7.0, 2.2),
        (9.5, -2.5),
        (11.5, 3.0),
        (13.5, -3.2),
        (-15.5, -4.5),
        (-7.5, -4.8),
        (1.5, -5.0),
        (10.0, 4.6),
        (-3.5, 5.4),
        (6.0, 5.2),
        (15.0, 4.0),
    ];
    for &(x, z) in grass_spots {
        spawn_grass_tuft(commands, lib, x, z);
    }

    // Bushes (slightly larger filler).
    let bush_spots: &[(f32, f32)] = &[
        (-14.5, -2.6),
        (-10.0, 4.2),
        (-5.0, -4.5),
        (4.5, 4.4),
        (12.5, -4.5),
        (16.0, 2.5),
        (-16.5, 3.5),
        (8.5, -5.2),
    ];
    for &(x, z) in bush_spots {
        commands.spawn((
            Mesh3d(lib.bush_mesh.clone()),
            MeshMaterial3d(lib.bush_mat.clone()),
            Transform {
                translation: Vec3::new(x, 0.18, z),
                scale: Vec3::new(1.0, 0.85, 1.0),
                ..default()
            },
        ));
    }

    // Flowers: stem + colored top.
    let flower_spots: &[(f32, f32, u8)] = &[
        (-12.5, -3.4, 0),
        (-8.0, 3.6, 1),
        (-2.5, -2.2, 2),
        (2.5, 3.6, 0),
        (5.0, -2.2, 1),
        (11.0, 2.5, 2),
        (14.0, -2.0, 0),
        (-5.5, 3.8, 2),
        (-15.0, 2.4, 1),
        (15.5, 3.2, 0),
    ];
    for &(x, z, color_idx) in flower_spots {
        let petal_mat = match color_idx {
            0 => lib.flower_red_mat.clone(),
            1 => lib.flower_yellow_mat.clone(),
            _ => lib.flower_violet_mat.clone(),
        };
        commands
            .spawn((Transform::from_xyz(x, 0.0, z), Visibility::default()))
            .with_children(|f| {
                f.spawn((
                    Mesh3d(lib.plant_stem.clone()),
                    MeshMaterial3d(lib.bush_mat.clone()),
                    Transform::from_xyz(0.0, 0.14, 0.0),
                ));
                f.spawn((
                    Mesh3d(lib.plant_flower.clone()),
                    MeshMaterial3d(petal_mat),
                    Transform::from_xyz(0.0, 0.30, 0.0),
                ));
            });
    }
}

fn spawn_trees(commands: &mut Commands, meshes: &mut Assets<Mesh>, lib: &MatLibrary) {
    let trunk_mat = lib.wood_mat.clone();
    let foliage_mat = lib.bush_mat.clone();
    let trunk_mesh = meshes.add(Cylinder::new(0.10, 1.0));
    let foliage_low = meshes.add(Cone::new(0.55, 1.0));
    let foliage_high = meshes.add(Cone::new(0.40, 0.9));

    // (x, z, height_scale)
    let trees: &[(f32, f32, f32)] = &[
        (-16.0, -6.5, 1.2),
        (-13.5, -8.0, 0.9),
        (-10.0, -7.5, 1.1),
        (-6.5, -6.0, 1.0),
        (-4.0, -7.5, 1.3),
        (2.5, -6.5, 0.95),
        (5.5, -7.5, 1.15),
        (9.0, -6.2, 1.05),
        (12.5, -7.0, 0.9),
        (16.0, -7.5, 1.2),
        (-15.0, 6.5, 1.1),
        (-11.0, 7.5, 0.95),
        (-7.5, 6.0, 1.25),
        (-3.5, 7.5, 0.9),
        (3.5, 6.5, 1.1),
        (8.0, 7.5, 1.0),
        (12.0, 6.5, 1.2),
        (15.5, 7.5, 0.95),
    ];
    for &(x, z, h) in trees {
        let trunk_h = 0.7 * h;
        let foliage1_h = 1.1 * h;
        let foliage2_h = 0.9 * h;
        commands
            .spawn((Transform::from_xyz(x, 0.0, z), Visibility::default()))
            .with_children(|t| {
                t.spawn((
                    Mesh3d(trunk_mesh.clone()),
                    MeshMaterial3d(trunk_mat.clone()),
                    Transform {
                        translation: Vec3::new(0.0, trunk_h * 0.5, 0.0),
                        scale: Vec3::new(1.0, trunk_h, 1.0),
                        ..default()
                    },
                ));
                t.spawn((
                    Mesh3d(foliage_low.clone()),
                    MeshMaterial3d(foliage_mat.clone()),
                    Transform {
                        translation: Vec3::new(0.0, trunk_h + foliage1_h * 0.5 - 0.05, 0.0),
                        scale: Vec3::new(1.0, foliage1_h, 1.0),
                        ..default()
                    },
                ));
                t.spawn((
                    Mesh3d(foliage_high.clone()),
                    MeshMaterial3d(foliage_mat.clone()),
                    Transform {
                        translation: Vec3::new(
                            0.0,
                            trunk_h + foliage1_h * 0.85 + foliage2_h * 0.5 - 0.1,
                            0.0,
                        ),
                        scale: Vec3::new(1.0, foliage2_h, 1.0),
                        ..default()
                    },
                ));
            });
    }
}

fn spawn_grass_tuft(commands: &mut Commands, lib: &MatLibrary, x: f32, z: f32) {
    // Three small cones leaning slightly outward form a grass tuft.
    let blades = [
        (0.0, 0.0, 0.0_f32),
        (0.07, 0.04, 0.15),
        (-0.06, -0.05, -0.18),
        (0.04, -0.07, 0.10),
    ];
    commands
        .spawn((Transform::from_xyz(x, 0.0, z), Visibility::default()))
        .with_children(|t| {
            for (i, &(dx, dz, tilt)) in blades.iter().enumerate() {
                let height_scale = 0.75 + 0.25 * ((i as f32) * 0.7).sin().abs();
                t.spawn((
                    Mesh3d(lib.grass_blade.clone()),
                    MeshMaterial3d(lib.grass_mat.clone()),
                    Transform {
                        translation: Vec3::new(dx, 0.10, dz),
                        rotation: Quat::from_rotation_z(tilt),
                        scale: Vec3::new(1.0, height_scale, 1.0),
                    },
                ));
            }
        });
}

fn spawn_castle(commands: &mut Commands, lib: &MatLibrary, slot: PlayerSlot, z: f32) {
    let side = slot.side();
    let x = match side {
        Side::Left => LEFT_BASE_X,
        Side::Right => RIGHT_BASE_X,
    };
    let main = match side {
        Side::Left => lib.left.clone(),
        Side::Right => lib.right.clone(),
    };

    let base_entity = commands
        .spawn((
            Transform {
                translation: Vec3::new(x, 0.0, z),
                rotation: side.base_rotation(),
                scale: Vec3::ONE,
            },
            Visibility::default(),
            Base,
            side,
            slot,
            Health::new(BASE_HP),
        ))
        .with_children(|p| {
            // Foundation
            p.spawn((
                Mesh3d(lib.castle_foundation.clone()),
                MeshMaterial3d(lib.stone_dark.clone()),
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
            // Central keep
            p.spawn((
                Mesh3d(lib.castle_keep.clone()),
                MeshMaterial3d(lib.stone_light.clone()),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
            // Battlement slab
            p.spawn((
                Mesh3d(lib.castle_top_slab.clone()),
                MeshMaterial3d(lib.stone_dark.clone()),
                Transform::from_xyz(0.0, 1.66, 0.0),
            ));
            // Crenellations around the slab edge
            let crenel_y = 1.83;
            for &(cx, cz) in &[
                (0.55, 0.0),
                (-0.55, 0.0),
                (0.0, 0.55),
                (0.0, -0.55),
                (0.40, 0.40),
                (-0.40, 0.40),
                (0.40, -0.40),
                (-0.40, -0.40),
            ] {
                p.spawn((
                    Mesh3d(lib.castle_crenel.clone()),
                    MeshMaterial3d(lib.stone_light.clone()),
                    Transform::from_xyz(cx, crenel_y, cz),
                ));
            }
            // Four corner towers with cone roofs (roofs use side color).
            for &(tx, tz) in &[(0.78, 0.78), (-0.78, 0.78), (0.78, -0.78), (-0.78, -0.78)] {
                p.spawn((
                    Mesh3d(lib.castle_tower.clone()),
                    MeshMaterial3d(lib.stone_light.clone()),
                    Transform::from_xyz(tx, 1.2, tz),
                ));
                p.spawn((
                    Mesh3d(lib.castle_roof.clone()),
                    MeshMaterial3d(main.clone()),
                    Transform::from_xyz(tx, 2.28, tz),
                ));
                // Torch at the corner top, doused by default (intensity set by night system).
                p.spawn((
                    Mesh3d(lib.torch_pole_mesh.clone()),
                    MeshMaterial3d(lib.wood_mat.clone()),
                    Transform::from_xyz(tx, 1.95, tz),
                ));
                p.spawn((
                    Mesh3d(lib.flame_mesh.clone()),
                    MeshMaterial3d(lib.flame_mat.clone()),
                    Transform::from_xyz(tx, 2.18, tz),
                    Visibility::Hidden,
                    TorchFlame,
                ));
                p.spawn((
                    PointLight {
                        color: TORCH_COLOR,
                        intensity: 0.0,
                        range: TORCH_RANGE,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(tx, 2.20, tz),
                    TorchLight,
                ));
            }
            // Door at the back (toward this side's miners).
            p.spawn((
                Mesh3d(lib.castle_door.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::from_xyz(-1.0 + 0.04, 0.67, 0.0),
            ));
            // Flag pole + flag on top of the keep.
            p.spawn((
                Mesh3d(lib.castle_pole.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::from_xyz(0.0, 2.3, 0.0),
            ));
            p.spawn((
                Mesh3d(lib.castle_flag.clone()),
                MeshMaterial3d(main.clone()),
                Transform::from_xyz(0.18, 2.65, 0.0),
            ));
        })
        .id();
    crate::healthbar::spawn_health_bar_for_base(commands, base_entity);
}

fn spawn_rock(commands: &mut Commands, lib: &MatLibrary, slot: PlayerSlot, z: f32) {
    let side = slot.side();
    let base_x = match side {
        Side::Left => LEFT_BASE_X,
        Side::Right => RIGHT_BASE_X,
    };
    // Rocks are placed behind each base (opposite of unit forward).
    let x = base_x - side.forward() * ROCK_OFFSET;

    commands
        .spawn((
            Transform::from_xyz(x, 0.0, z),
            Visibility::default(),
            Rock,
            side,
            slot,
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(lib.rock_large.clone()),
                MeshMaterial3d(lib.rock_mat.clone()),
                Transform::from_xyz(0.0, 0.45, 0.0),
            ));
            p.spawn((
                Mesh3d(lib.rock_medium.clone()),
                MeshMaterial3d(lib.rock_mat.clone()),
                Transform {
                    translation: Vec3::new(0.32, 0.28, 0.30),
                    rotation: Quat::from_rotation_y(0.6),
                    scale: Vec3::ONE,
                },
            ));
            p.spawn((
                Mesh3d(lib.rock_small.clone()),
                MeshMaterial3d(lib.rock_mat.clone()),
                Transform::from_xyz(-0.38, 0.22, -0.28),
            ));
        });
}
