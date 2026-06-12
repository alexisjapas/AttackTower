use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::Image;
use bevy::light::light_consts::lux;
use bevy::light::{
    CascadeShadowConfigBuilder, FogVolume, NotShadowCaster, VolumetricFog, VolumetricLight,
};
use bevy::pbr::{Atmosphere, AtmosphereSettings, DistanceFog, FogFalloff, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
#[cfg(feature = "raytracing")]
use bevy::solari::prelude::RaytracingMesh3d;

use crate::common::*;

/// World authoring + renderer plumbing: startup scene/assets, the arena built
/// on match entry, day/night cycle, raytracing mesh registration and the debug
/// camera. Settings application lives in `graphics.rs`.
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MatLibrary>()
            .init_resource::<EnvAssets>()
            .init_resource::<UnitModels>()
            .init_resource::<TimeOfDay>()
            .init_resource::<GameTime>()
            .init_resource::<DlssAvailable>()
            // init_mat_library and load_env_assets must precede setup_world
            // (it reads both); load_unit_models is independent.
            .add_systems(
                Startup,
                (
                    (init_mat_library, load_env_assets, setup_world).chain(),
                    load_unit_models,
                ),
            )
            .add_systems(OnEnter(InMatch), spawn_arena)
            .add_systems(
                Update,
                (
                    advance_game_time.run_if(in_state(GameState::Playing)),
                    animate_sun,
                    build_unit_graphs,
                )
                    .in_set(AppSet::World),
            )
            .add_systems(
                Update,
                (update_torches, sync_raytracing_meshes).in_set(AppSet::React),
            )
            .add_systems(Update, debug_camera_control.in_set(AppSet::Visual));
    }
}

pub fn init_mat_library(
    mut lib: ResMut<MatLibrary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
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
    // Procedural three-region ground texture: sand play field, blue no-man's-land,
    // cooler decor outside — with quick fades (see `generate_ground_texture`).
    // Built with the 1v1 zone here; `spawn_arena` regenerates it in place for the
    // active GameMode at match start (the Z extent tracks the tower z-limit).
    let ground_tex = images.add(generate_ground_texture(TOWER_PLACEMENT_Z_LIMIT_1V1));
    lib.ground = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(ground_tex.clone()),
        perceptual_roughness: 1.0,
        ..default()
    });
    lib.ground_tex = ground_tex;
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

    lib.arrow_shaft = meshes.add(Cylinder::new(0.014, 0.55));
    lib.arrow_tip = meshes.add(Cone::new(0.040, 0.10));
    lib.arrow_fletch = meshes.add(Cuboid::new(0.01, 0.08, 0.07));

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
}

/// Startup: kick off the async load of the glTF building + desert prop scenes.
/// Must run before `setup_world` (which spawns the scenery) and `spawn_arena`
/// (bases/rocks) so the handles exist; the scenes themselves instance lazily.
pub fn load_env_assets(asset_server: Res<AssetServer>, mut env: ResMut<EnvAssets>) {
    let scn = |p: &'static str| -> Handle<Scene> {
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(p))
    };
    env.base = scn(BASE_MODEL_PATH);
    env.tower = scn(TOWER_MODEL_PATH);
    env.mountain = scn(MOUNTAIN_MODEL_PATH);
    env.cactus = [scn(PROP_CACTUS_PATHS[0]), scn(PROP_CACTUS_PATHS[1])];
    env.dead_tree = [scn(PROP_DEAD_TREE_PATHS[0]), scn(PROP_DEAD_TREE_PATHS[1])];
    env.ruins = [scn(PROP_RUINS_PATHS[0]), scn(PROP_RUINS_PATHS[1])];
    env.skull = [scn(PROP_SKULL_PATHS[0]), scn(PROP_SKULL_PATHS[1])];
    env.stone = [scn(PROP_STONE_PATHS[0]), scn(PROP_STONE_PATHS[1])];
    env.stone_arch = [scn(PROP_STONE_ARCH_PATHS[0]), scn(PROP_STONE_ARCH_PATHS[1])];
}

/// Startup: kick off the async load of every unit kind's glTF model — the scene
/// (mesh + skeleton, from the Walking file) plus one `AnimationClip` per action
/// (each from its own single-animation file; the path is the source of truth as
/// Meshy scrambles the internal names) and the hand weapon. Scenes instance
/// per-unit in `spawn_unit`; graphs are built once the clips decode (see
/// `build_unit_graphs`).
pub fn load_unit_models(asset_server: Res<AssetServer>, mut models: ResMut<UnitModels>) {
    let scn = |p: &str| asset_server.load(GltfAssetLabel::Scene(0).from_asset(p.to_string()));
    let clip = |p: &str| asset_server.load(GltfAssetLabel::Animation(0).from_asset(p.to_string()));
    let weapon = |path: &str,
                  bone: &'static str,
                  offset: Vec3,
                  rotation: Vec3,
                  self_flip: f32,
                  scale: f32,
                  grip: f32| WeaponDef {
        scene: scn(path),
        bone,
        offset,
        rotation,
        self_flip,
        scale,
        grip,
    };

    *models.get_mut(UnitKind::Soldier) = UnitModel {
        scene: scn(SOLDIER_SCENE_PATH),
        walk: clip(SOLDIER_WALK_PATH),
        attack: Some(clip(SOLDIER_ATTACK_PATH)),
        death: Some(clip(SOLDIER_DEATH_PATH)),
        weapon: Some(weapon(
            SWORD_PATH,
            SWORD_BONE,
            SWORD_OFFSET,
            SWORD_ROTATION,
            SWORD_SELF_FLIP,
            SWORD_SCALE,
            SWORD_GRIP,
        )),
        scale: SOLDIER_MODEL_SCALE,
        yaw_offset: SOLDIER_MODEL_YAW_OFFSET,
        cooldown: SOLDIER_COOLDOWN,
        death_duration: SOLDIER_DEATH_DURATION,
        ..default()
    };

    *models.get_mut(UnitKind::Miner) = UnitModel {
        scene: scn(MINER_SCENE_PATH),
        walk: clip(MINER_WALK_PATH),
        // The miner's only action clip is the mining swing; no death clip.
        attack: Some(clip(MINER_ATTACK_PATH)),
        death: None,
        weapon: Some(weapon(
            PICKAXE_PATH,
            PICKAXE_BONE,
            PICKAXE_OFFSET,
            PICKAXE_ROTATION,
            PICKAXE_SELF_FLIP,
            PICKAXE_SCALE,
            PICKAXE_GRIP,
        )),
        scale: MINER_MODEL_SCALE,
        yaw_offset: MINER_MODEL_YAW_OFFSET,
        cooldown: MINER_COOLDOWN,
        death_duration: DEATH_DURATION,
        ..default()
    };

    *models.get_mut(UnitKind::Archer) = UnitModel {
        scene: scn(ARCHER_SCENE_PATH),
        walk: clip(ARCHER_WALK_PATH),
        attack: Some(clip(ARCHER_SHOT_PATH)),
        death: Some(clip(ARCHER_DEATH_PATH)),
        weapon: Some(weapon(
            ARCHER_BOW_PATH,
            ARCHER_BOW_HAND_BONE,
            ARCHER_BOW_OFFSET,
            ARCHER_BOW_ROTATION,
            ARCHER_BOW_SELF_FLIP,
            ARCHER_BOW_SCALE,
            ARCHER_BOW_GRIP,
        )),
        scale: ARCHER_MODEL_SCALE,
        yaw_offset: ARCHER_MODEL_YAW_OFFSET,
        cooldown: ARCHER_COOLDOWN,
        death_duration: ARCHER_DEATH_DURATION,
        ..default()
    };

    *models.get_mut(UnitKind::Priest) = UnitModel {
        scene: scn(PRIEST_SCENE_PATH),
        walk: clip(PRIEST_WALK_PATH),
        attack: Some(clip(PRIEST_ATTACK_PATH)),
        death: Some(clip(PRIEST_DEATH_PATH)),
        weapon: Some(weapon(
            STAFF_PATH,
            STAFF_BONE,
            STAFF_OFFSET,
            STAFF_ROTATION,
            STAFF_SELF_FLIP,
            STAFF_SCALE,
            STAFF_GRIP,
        )),
        scale: PRIEST_MODEL_SCALE,
        yaw_offset: PRIEST_MODEL_YAW_OFFSET,
        cooldown: PRIEST_COOLDOWN,
        death_duration: PRIEST_DEATH_DURATION,
        ..default()
    };
}

/// Update: for each unit kind, once all its clips have decoded, build the
/// `AnimationGraph` and cache node indices + playback speeds derived from the
/// clip durations. Tolerates kinds with no attack/death clips (the miner).
/// Runs each frame until every kind is built, then no-ops.
pub fn build_unit_graphs(
    mut models: ResMut<UnitModels>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for kind in UNIT_KINDS {
        {
            let m = models.get(kind);
            if m.nodes.is_some() {
                continue;
            }
            let ready = clips.get(&m.walk).is_some()
                && m.attack.as_ref().is_none_or(|a| clips.get(a).is_some())
                && m.death.as_ref().is_none_or(|d| clips.get(d).is_some());
            if !ready {
                continue;
            }
        }

        let m = models.get(kind).clone();
        let mut graph = AnimationGraph::new();
        let root = graph.root;
        let walk = graph.add_clip(m.walk.clone(), 1.0, root);
        let attack = m
            .attack
            .as_ref()
            .map(|a| graph.add_clip(a.clone(), 1.0, root));
        let death = m
            .death
            .as_ref()
            .map(|d| graph.add_clip(d.clone(), 1.0, root));

        let dur = |h: &Handle<AnimationClip>| clips.get(h).map(|c| c.duration()).unwrap_or(1.0);
        let attack_len = m.attack.as_ref().map(&dur).unwrap_or(1.0);
        // Play one attack cycle per cooldown, and the fall within the death window.
        let attack_speed = (attack_len / m.cooldown).max(0.01);
        let death_speed = m
            .death
            .as_ref()
            .map(|d| (dur(d) / m.death_duration).max(0.01))
            .unwrap_or(1.0);

        let handle = graphs.add(graph);
        let mm = models.get_mut(kind);
        mm.graph = Some(handle);
        mm.nodes = Some(ModelAnimNodes {
            walk,
            attack,
            death,
            attack_speed,
            attack_len,
            death_speed,
        });
    }
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    lib: Res<MatLibrary>,
    env: Res<EnvAssets>,
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
            Transform::from_translation(CAMERA_DEFAULT_POS)
                .looking_at(CAMERA_DEFAULT_TARGET, Vec3::Y),
            Atmosphere::earthlike(medium),
            AtmosphereSettings::default(),
            Exposure { ev100: 13.0 },
            Tonemapping::AcesFitted,
            Bloom::NATURAL,
            DistanceFog {
                // Tuned to the sky/horizon tone so the distant ground melts into
                // the sky instead of ending on a hard horizon line. Denser than a
                // pure mood fog: it must reach near-opacity before the (now large)
                // ground plane's edge so that edge is never visible.
                color: Color::srgba(0.60, 0.73, 0.86, 1.0),
                falloff: FogFalloff::ExponentialSquared { density: 0.018 },
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

    // Map-wide haze the sun shines through (subtle god rays). Stretched out to
    // the mountain ring and widened on the sides so the mist reaches the peaks
    // instead of stopping mid-field; density is dropped accordingly so the
    // larger volume stays a light haze rather than turning opaque.
    commands.spawn((
        FogVolume {
            density_factor: 0.012,
            ..default()
        },
        Transform::from_scale(Vec3::new(150.0, 12.0, 130.0))
            .with_translation(Vec3::new(0.0, 5.0, 0.0)),
    ));

    // Grass plain. Made very large (±150) on purpose: with the flatter camera
    // the horizon is in view, so the plane must extend far enough that the
    // distance fog has fully saturated before its edge — the edge then dissolves
    // into the sky tone and no hard horizon line shows behind the mountains.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(GROUND_PLANE_SIZE, 0.2, GROUND_PLANE_SIZE))),
        MeshMaterial3d(lib.ground.clone()),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    spawn_sky(&mut commands, &mut meshes, &mut materials);
    spawn_mountains(&mut commands, &env);

    spawn_scenery(&mut commands, &env);
}

/// OnEnter(InMatch): build bases + rocks for the active GameMode. Despawned by
/// `reset_match` on return to the menu, so the next match can be rebuilt
/// cleanly for either 1v1 or 2v2.
pub fn spawn_arena(
    mut commands: Commands,
    lib: Res<MatLibrary>,
    env: Res<EnvAssets>,
    mode: Res<GameMode>,
    mut images: ResMut<Assets<Image>>,
    bases: Query<Entity, With<Base>>,
) {
    // `Paused → Settings → Paused` re-enters InMatch with the arena already
    // built — keep it (see the `InMatch` doc in common.rs).
    if bases.iter().next().is_some() {
        return;
    }
    // Repaint the ground for the active mode: the sand band's Z extent tracks the
    // mode's tower z-limit, so the visible play field equals the buildable zone.
    if let Some(img) = images.get_mut(&lib.ground_tex) {
        *img = generate_ground_texture(mode.tower_z_limit());
    }
    for &slot in mode.active_slots() {
        let z = slot.base_z(*mode);
        spawn_castle(&mut commands, &lib, &env, slot, z);
        spawn_rock(&mut commands, &env, slot, z);
    }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 1 inside the play-field rectangle, 0 outside, with a `GROUND_COLOR_FADE` ramp.
/// `half_z` is the active mode's tower z-limit so the sand matches the buildable zone.
fn ground_play_mask(x: f32, z: f32, half_z: f32) -> f32 {
    let f = GROUND_COLOR_FADE;
    let in_x = 1.0 - smoothstep(GROUND_PLAY_HALF_X - f, GROUND_PLAY_HALF_X, x.abs());
    let in_z = 1.0 - smoothstep(half_z - f, half_z, z.abs());
    in_x * in_z
}

/// 1 in the central no-man's-land (|x| < ZONE_BOUNDARY), fading to 0 at its edge.
fn ground_nomans_mask(x: f32) -> f32 {
    1.0 - smoothstep(ZONE_BOUNDARY - GROUND_COLOR_FADE, ZONE_BOUNDARY, x.abs())
}

fn lerp_srgba(a: Srgba, b: Srgba, t: f32) -> Srgba {
    Srgba::new(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        1.0,
    )
}

/// Build the ground's base-color texture: decor outside the play rectangle, sand
/// inside, with the central no-man's-land tinted blue — all blended with quick
/// smoothstep fades. `half_z` is the active mode's tower z-limit so the sand band
/// matches the buildable zone. Mapped to the ground cuboid's top face, whose
/// UV 0..1 spans ±`GROUND_PLANE_SIZE`/2 in world XZ.
fn generate_ground_texture(half_z: f32) -> Image {
    let n = GROUND_TEX_SIZE;
    let sand = GROUND_SAND.to_srgba();
    let decor = GROUND_DECOR.to_srgba();
    let blue = GROUND_NOMANS.to_srgba();
    let mut data = vec![0u8; (n * n * 4) as usize];
    for j in 0..n {
        let z = ((j as f32 + 0.5) / n as f32 - 0.5) * GROUND_PLANE_SIZE;
        for i in 0..n {
            let x = ((i as f32 + 0.5) / n as f32 - 0.5) * GROUND_PLANE_SIZE;
            // Inside the play field: sand, turning blue toward the centre line.
            let field = lerp_srgba(sand, blue, ground_nomans_mask(x));
            // Then blend the whole field over the decor at the play-rect edge.
            let c = lerp_srgba(decor, field, ground_play_mask(x, z, half_z));
            let o = ((j * n + i) * 4) as usize;
            data[o] = (c.red * 255.0) as u8;
            data[o + 1] = (c.green * 255.0) as u8;
            data[o + 2] = (c.blue * 255.0) as u8;
            data[o + 3] = 255;
        }
    }
    Image::new(
        Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
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

/// Runs only in `GameState::Playing` (run condition), so pause/menus freeze
/// the in-game clock.
pub fn advance_game_time(time: Res<Time>, mut gtime: ResMut<GameTime>) {
    gtime.0 += time.delta_secs();
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

/// Free-fly debug camera, driven by mouse + keyboard (the shipped game is
/// gamepad-only, so these inputs are otherwise unused and won't clash). Hold the
/// **right mouse button** to look around; **WASD** moves in the view plane,
/// **Space**/**Left Shift** fly up/down, **Left Ctrl** boosts, the **scroll
/// wheel** changes fly speed, and **R** snaps back to the default 3/4 view.
/// Runs in every `GameState` so the scene can be inspected while paused.
pub fn debug_camera_control(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<bevy::input::mouse::AccumulatedMouseMotion>,
    scroll: Res<bevy::input::mouse::AccumulatedMouseScroll>,
    mut speed: Local<f32>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    // R resets to the canonical game view and bails for the frame.
    if keys.just_pressed(KeyCode::KeyR) {
        *transform = Transform::from_translation(CAMERA_DEFAULT_POS)
            .looking_at(CAMERA_DEFAULT_TARGET, Vec3::Y);
        return;
    }

    if *speed <= 0.0 {
        *speed = DEBUG_CAM_BASE_SPEED;
    }
    // Scroll multiplies the fly speed (exponential feel), clamped to a sane band.
    if scroll.delta.y != 0.0 {
        *speed = (*speed * (1.0 + scroll.delta.y * 0.1)).clamp(1.0, 200.0);
    }

    // Mouse look only while RMB is held, so the cursor stays free for menus.
    if mouse_buttons.pressed(MouseButton::Right) && motion.delta != Vec2::ZERO {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        yaw -= motion.delta.x * DEBUG_CAM_SENSITIVITY;
        pitch -= motion.delta.y * DEBUG_CAM_SENSITIVITY;
        // Clamp just shy of straight up/down to avoid gimbal flip.
        let limit = std::f32::consts::FRAC_PI_2 - 0.01;
        pitch = pitch.clamp(-limit, limit);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    // WASD in the view plane + vertical fly, relative to current orientation.
    let mut dir = Vec3::ZERO;
    let forward = *transform.forward();
    let right = *transform.right();
    if keys.pressed(KeyCode::KeyW) {
        dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keys.pressed(KeyCode::Space) {
        dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        dir -= Vec3::Y;
    }
    if dir != Vec3::ZERO {
        let boost = if keys.pressed(KeyCode::ControlLeft) {
            3.0
        } else {
            1.0
        };
        transform.translation += dir.normalize() * *speed * boost * time.delta_secs();
    }
}

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

fn spawn_mountains(commands: &mut Commands, env: &EnvAssets) {
    // The mountain glTF is a single ridge normalized (like the props) to a
    // ~1.9-unit box: half-width ~0.95 on X, ~0.467 tall on Y, base at -0.234.
    // Each peak's `w` (old cone radius) maps to a horizontal scale and `h` to a
    // vertical scale, with the base seated on the ground.
    const MODEL_HALF_X: f32 = 0.95;
    const MODEL_HEIGHT: f32 = 0.467;
    const MODEL_BASE: f32 = 0.234;

    // (x, z, width, height). The front row (negative z, the one the camera
    // faces) is widened out to ±60 so it frames the enlarged plain across the
    // whole horizon instead of leaving flat gaps on the sides; a few taller
    // peaks sit farther back (z ~ -48) for layered depth.
    let peaks: &[(f32, f32, f32, f32)] = &[
        // Front row, left to right.
        (-58.0, -35.0, 9.0, 8.0),
        (-49.0, -33.0, 8.0, 7.0),
        (-40.0, -36.0, 9.0, 9.0),
        (-31.0, -34.0, 7.0, 7.5),
        (-22.0, -37.0, 9.0, 9.5),
        (-13.0, -33.0, 6.0, 6.5),
        (-3.0, -38.0, 10.0, 11.0),
        (6.0, -34.0, 8.0, 8.5),
        (15.0, -36.0, 9.0, 9.0),
        (25.0, -33.0, 7.0, 7.5),
        (35.0, -36.0, 9.0, 9.0),
        (44.0, -34.0, 8.0, 7.5),
        (54.0, -36.0, 10.0, 9.0),
        // Taller layer set farther back for depth.
        (-35.0, -48.0, 12.0, 13.0),
        (-10.0, -50.0, 13.0, 14.0),
        (18.0, -49.0, 12.0, 13.0),
        (42.0, -50.0, 13.0, 13.5),
    ];
    // Deterministic per-peak yaw jitter (plus an occasional 180° flip) so the
    // instanced ridge doesn't read as the same silhouette repeated; kept near
    // 0/π so the wide axis stays roughly parallel to the horizon.
    let mut rng = Rng::new(0xB165_CA1E_5EED_0001);
    for &(x, z, w, h) in peaks {
        let s_h = w / MODEL_HALF_X;
        let s_y = h / MODEL_HEIGHT;
        let flip = if rng.next_u32() & 1 == 0 {
            0.0
        } else {
            std::f32::consts::PI
        };
        let yaw = flip + rng.range(-0.35, 0.35);
        commands.spawn((
            SceneRoot(env.mountain.clone()),
            Transform {
                translation: Vec3::new(x, MODEL_BASE * s_y, z),
                rotation: Quat::from_rotation_y(yaw),
                scale: Vec3::new(s_h, s_y, s_h),
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
    // High and well beyond the mountain ring (z < -45) so they sit in the sky
    // above the peaks now that the flatter camera reveals the horizon, rather
    // than drifting low across the field. Flattened (low sy) to read as clouds.
    for &(x, y, z, sx, sy, sz) in &[
        (-30.0, 30.0, -58.0, 6.0, 1.8, 3.5),
        (14.0, 34.0, -66.0, 7.5, 2.2, 4.0),
        (34.0, 28.0, -52.0, 5.5, 1.6, 3.0),
        (-46.0, 32.0, -48.0, 6.5, 2.0, 3.2),
        (4.0, 38.0, -72.0, 8.5, 2.4, 4.5),
        (48.0, 35.0, -62.0, 6.0, 1.8, 3.4),
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

/// Desert prop kinds scattered around the arena. Each maps to a pair of glTF
/// variants in `EnvAssets` and a scale / ground-lift from the `PROP_*` consts.
#[derive(Clone, Copy)]
enum DesertProp {
    Cactus,
    DeadTree,
    Ruins,
    Skull,
    Stone,
    StoneArch,
}

/// Every kind, in a fixed order, for the weighted random pick.
const ALL_DESERT_PROPS: [DesertProp; 6] = [
    DesertProp::Cactus,
    DesertProp::DeadTree,
    DesertProp::Ruins,
    DesertProp::Skull,
    DesertProp::Stone,
    DesertProp::StoneArch,
];

impl DesertProp {
    /// (scale, ground-lift) for this kind.
    fn dims(self) -> (f32, f32) {
        match self {
            DesertProp::Cactus => (PROP_CACTUS_SCALE, PROP_CACTUS_LIFT),
            DesertProp::DeadTree => (PROP_DEAD_TREE_SCALE, PROP_DEAD_TREE_LIFT),
            DesertProp::Ruins => (PROP_RUINS_SCALE, PROP_RUINS_LIFT),
            DesertProp::Skull => (PROP_SKULL_SCALE, PROP_SKULL_LIFT),
            DesertProp::Stone => (PROP_STONE_SCALE, PROP_STONE_LIFT),
            DesertProp::StoneArch => (PROP_STONE_ARCH_SCALE, PROP_STONE_ARCH_LIFT),
        }
    }

    /// Relative spawn frequency. Rocks/cacti/dead trees are common; ruins and
    /// arches are landmarks and stay rare.
    fn weight(self) -> f32 {
        match self {
            DesertProp::Stone => 5.0,
            DesertProp::Cactus => 4.0,
            DesertProp::DeadTree => 4.0,
            DesertProp::Skull => 2.0,
            DesertProp::Ruins => 1.0,
            DesertProp::StoneArch => 1.0,
        }
    }

    fn scene(self, env: &EnvAssets, variant: usize) -> Handle<Scene> {
        let v = variant & 1;
        match self {
            DesertProp::Cactus => env.cactus[v].clone(),
            DesertProp::DeadTree => env.dead_tree[v].clone(),
            DesertProp::Ruins => env.ruins[v].clone(),
            DesertProp::Skull => env.skull[v].clone(),
            DesertProp::Stone => env.stone[v].clone(),
            DesertProp::StoneArch => env.stone_arch[v].clone(),
        }
    }
}

impl DesertProp {
    /// Weighted random pick over `ALL_DESERT_PROPS` (the shared `common::Rng`
    /// keeps the scatter reproducible across runs — fixed seed, no `rand`).
    fn pick(rng: &mut Rng) -> DesertProp {
        let total: f32 = ALL_DESERT_PROPS.iter().map(|k| k.weight()).sum();
        let mut r = rng.unit() * total;
        for &k in &ALL_DESERT_PROPS {
            if r < k.weight() {
                return k;
            }
            r -= k.weight();
        }
        DesertProp::Stone
    }
}

/// Fill the background with desert props: a jittered grid spanning from just
/// outside the play zone out to the mountains, each cell holding one prop whose
/// kind is drawn by `DesertProp::weight` (rocks/trees common, ruins/arches rare)
/// at a random yaw and slightly randomized scale.
fn spawn_scenery(commands: &mut Commands, env: &EnvAssets) {
    let mut rng = Rng::new(0x5F3A_C0FF_EE15_600D);
    let mut gx = -SCENERY_X_RANGE;
    while gx <= SCENERY_X_RANGE {
        let mut gz = SCENERY_Z_MIN;
        while gz <= SCENERY_Z_MAX {
            let x = gx + rng.range(-SCENERY_JITTER, SCENERY_JITTER);
            let z = gz + rng.range(-SCENERY_JITTER, SCENERY_JITTER);
            gz += SCENERY_GRID_STEP;
            // Keep the bases, lanes and tower zones clear.
            if x.abs() < SCENERY_CLEAR_X && z > SCENERY_CLEAR_Z_MIN && z < SCENERY_CLEAR_Z_MAX {
                continue;
            }
            let kind = DesertProp::pick(&mut rng);
            let variant = (rng.next_u32() & 1) as usize;
            let (base_scale, base_lift) = kind.dims();
            let s = rng.range(SCENERY_SCALE_MIN, SCENERY_SCALE_MAX);
            let yaw = rng.range(0.0, std::f32::consts::TAU);
            commands.spawn((
                SceneRoot(kind.scene(env, variant)),
                Transform::from_xyz(x, base_lift * s, z)
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::splat(base_scale * s)),
            ));
        }
        gx += SCENERY_GRID_STEP;
    }
}

fn spawn_castle(
    commands: &mut Commands,
    lib: &MatLibrary,
    env: &EnvAssets,
    slot: PlayerSlot,
    z: f32,
) {
    let side = slot.side();
    let x = side.base_x();

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
            // Static obstacle so units stop at the wall (Avian-blocked) instead
            // of walking through the keep. Radius kept under melee reach.
            RigidBody::Static,
            Collider::cylinder(BASE_COLLIDER_RADIUS, BASE_COLLIDER_HEIGHT),
            side.structure_layers(),
        ))
        .with_children(|p| {
            // The base building model.
            p.spawn((
                SceneRoot(env.base.clone()),
                Transform::from_xyz(0.0, BASE_MODEL_LIFT, 0.0)
                    .with_rotation(Quat::from_rotation_y(BASE_MODEL_YAW_OFFSET))
                    .with_scale(Vec3::splat(BASE_MODEL_SCALE)),
            ));
            // Corner torches — doused by default; lit at night by `update_torches`.
            let r = BASE_TORCH_RADIUS;
            for &(tx, tz) in &[(r, r), (-r, r), (r, -r), (-r, -r)] {
                spawn_torch(p, lib, Vec3::new(tx, BASE_TORCH_POLE_Y, tz));
            }
        })
        .id();
    crate::healthbar::spawn_health_bar_for_base(commands, base_entity);
}

fn spawn_rock(commands: &mut Commands, env: &EnvAssets, slot: PlayerSlot, z: f32) {
    let side = slot.side();
    // Rocks are placed behind each base (opposite of unit forward).
    let x = side.base_x() - side.forward() * ROCK_OFFSET;

    commands
        .spawn((
            Transform::from_xyz(x, 0.0, z),
            Visibility::default(),
            Rock,
            side,
            slot,
            RigidBody::Static,
            Collider::cylinder(ROCK_COLLIDER_RADIUS, ROCK_COLLIDER_HEIGHT),
            side.structure_layers(),
        ))
        .with_children(|p| {
            // The mining rock reuses the desert stone prop.
            p.spawn((
                SceneRoot(env.stone[0].clone()),
                Transform::from_xyz(0.0, ROCK_MODEL_LIFT, 0.0)
                    .with_scale(Vec3::splat(ROCK_MODEL_SCALE)),
            ));
        });
}

/// Spawn one procedural torch (pole + hidden flame + doused light) as a child,
/// with the pole base at `pole`. Shared by bases and towers; the flame/light sit
/// just above the pole. `update_torches` reveals and brightens them at night.
pub fn spawn_torch(p: &mut ChildSpawnerCommands, lib: &MatLibrary, pole: Vec3) {
    p.spawn((
        Mesh3d(lib.torch_pole_mesh.clone()),
        MeshMaterial3d(lib.wood_mat.clone()),
        Transform::from_translation(pole),
    ));
    p.spawn((
        Mesh3d(lib.flame_mesh.clone()),
        MeshMaterial3d(lib.flame_mat.clone()),
        Transform::from_translation(pole + Vec3::new(0.0, 0.23, 0.0)),
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
        Transform::from_translation(pole + Vec3::new(0.0, 0.25, 0.0)),
        TorchLight,
    ));
}
