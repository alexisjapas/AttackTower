//! Environment & presentation tunables: building/prop glTF paths and their
//! scale/ground-lift, procedural scenery scatter, ground texture regions,
//! torches, sun cycle and the camera (game view + debug fly-cam).

use bevy::prelude::*;

use super::arena::{RIGHT_BASE_X, TOWER_PLACEMENT_MARGIN};

// ─────────────────────────────────────────────────────────────────────────────
// Environment glTF assets (buildings + desert props). Every Meshy export is
// normalized to a ~1.9-unit box centered at the origin, so each kind gets a
// SCALE (multiplier on that ~1.9-unit model) and a LIFT (Y offset ≈ SCALE×|min_y|
// that seats the model's base on the ground). Tune visually if anything floats
// or sinks. Loaded into `EnvAssets` by `setup::load_env_assets`.
// ─────────────────────────────────────────────────────────────────────────────
pub const BASE_MODEL_PATH: &str = "models/adamar/buildings/adamar_base.glb";
pub const TOWER_MODEL_PATH: &str = "models/adamar/buildings/adamar_tower.glb";
/// A single ridge mesh instanced into the background mountain ring (replaces the
/// old procedural cones). Normalized like the props to a ~1.9-unit box.
pub const MOUNTAIN_MODEL_PATH: &str = "models/mountains.glb";
pub const PROP_CACTUS_PATHS: [&str; 2] = [
    "models/adamar/props/irrhakur_cactus_1.glb",
    "models/adamar/props/irrhakur_cactus_2.glb",
];
pub const PROP_DEAD_TREE_PATHS: [&str; 2] = [
    "models/adamar/props/irrhakur_dead_tree_1.glb",
    "models/adamar/props/irrhakur_dead_tree_2.glb",
];
pub const PROP_RUINS_PATHS: [&str; 2] = [
    "models/adamar/props/irrhakur_ruins_1.glb",
    "models/adamar/props/irrhakur_ruins_2.glb",
];
pub const PROP_SKULL_PATHS: [&str; 2] = [
    "models/adamar/props/irrhakur_skull_1.glb",
    "models/adamar/props/irrhakur_skull_2.glb",
];
pub const PROP_STONE_PATHS: [&str; 2] = [
    "models/adamar/props/irrhakur_stone_1.glb",
    "models/adamar/props/irrhakur_stone_2.glb",
];
pub const PROP_STONE_ARCH_PATHS: [&str; 2] = [
    "models/adamar/props/irrhakur_stone_arch_1.glb",
    "models/adamar/props/irrhakur_stone_arch_2.glb",
];

// Building scale + ground-lift.
pub const BASE_MODEL_SCALE: f32 = 2.73;
pub const BASE_MODEL_LIFT: f32 = 2.61;
/// Yaw applied to the base model within the (already side-rotated) entity frame,
/// so its front faces the center: left bases turn toward the right, right bases
/// (mirrored by `Side::base_rotation`) toward the left. Tune by ±90° if off.
pub const BASE_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
pub const TOWER_MODEL_SCALE: f32 = 1.4;
pub const TOWER_MODEL_LIFT: f32 = 1.34;
/// Yaw applied to the tower model so it also faces the center, combined with the
/// side mirroring (`Side::base_rotation`) in `spawn_tower`. Tune by ±90° if off.
pub const TOWER_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
/// The mining rock now reuses the desert stone prop (variant 0).
pub const ROCK_MODEL_SCALE: f32 = 1.0;
pub const ROCK_MODEL_LIFT: f32 = 0.70;

// Desert prop scale + ground-lift, per kind.
pub const PROP_CACTUS_SCALE: f32 = 1.1;
pub const PROP_CACTUS_LIFT: f32 = 1.0;
pub const PROP_DEAD_TREE_SCALE: f32 = 1.6;
pub const PROP_DEAD_TREE_LIFT: f32 = 1.5;
pub const PROP_RUINS_SCALE: f32 = 1.4;
pub const PROP_RUINS_LIFT: f32 = 1.3;
pub const PROP_SKULL_SCALE: f32 = 0.7;
pub const PROP_SKULL_LIFT: f32 = 0.27;
pub const PROP_STONE_SCALE: f32 = 0.8;
pub const PROP_STONE_LIFT: f32 = 0.55;
pub const PROP_STONE_ARCH_SCALE: f32 = 1.8;
pub const PROP_STONE_ARCH_LIFT: f32 = 1.35;

// Procedural scenery scatter: a jittered grid of desert props filling the
// background from just outside the play zone out to the mountains. Density comes
// from the grid step; per-kind frequency is weighted in `DesertProp::weight`.
pub const SCENERY_GRID_STEP: f32 = 5.0;
pub const SCENERY_JITTER: f32 = 2.6;
/// Scatter spans x ∈ [−RANGE, RANGE], z ∈ [Z_MIN, Z_MAX] (Z_MIN stops short of
/// the front mountain row at z ≈ −33; +z is capped near the camera).
pub const SCENERY_X_RANGE: f32 = 52.0;
pub const SCENERY_Z_MIN: f32 = -30.0;
pub const SCENERY_Z_MAX: f32 = 12.0;
/// Keep-clear gameplay rectangle (no props): |x| < CLEAR_X and z ∈ [CLEAR_Z_MIN,
/// CLEAR_Z_MAX] — covers the bases, lanes and tower zones.
pub const SCENERY_CLEAR_X: f32 = 23.0;
pub const SCENERY_CLEAR_Z_MIN: f32 = -10.0;
pub const SCENERY_CLEAR_Z_MAX: f32 = 10.0;
/// Per-instance random scale spread applied on top of each kind's base scale.
pub const SCENERY_SCALE_MIN: f32 = 0.8;
pub const SCENERY_SCALE_MAX: f32 = 1.25;

// ─────────────────────────────────────────────────────────────────────────────
// Ground coloring. A procedural texture (built in `init_mat_library`) paints the
// ground in three regions with quick smoothstep fades: the play field (sand),
// the surrounding decor (a cooler tone), and the central no-man's-land (blue).
// ─────────────────────────────────────────────────────────────────────────────
/// XZ extent of the ground cuboid — the texture's UV 0..1 maps to ±half this.
pub const GROUND_PLANE_SIZE: f32 = 300.0;
/// Resolution of the generated ground texture (square).
pub const GROUND_TEX_SIZE: u32 = 1024;
/// Width (world units) of the fade between adjacent ground regions ("fondu rapide").
pub const GROUND_COLOR_FADE: f32 = 1.0;
/// The sand play field is the actual tower-buildable strip, so the visible sand
/// matches where you can build. In X it runs from the centre no-man's-land out to
/// the base minus the placement margin; in Z it uses the active mode's tower
/// z-limit (passed to `generate_ground_texture` at match start). Bases, mining
/// rocks and the far field read as decor.
pub const GROUND_PLAY_HALF_X: f32 = RIGHT_BASE_X - TOWER_PLACEMENT_MARGIN;
pub const GROUND_SAND: Color = Color::srgb(0.78, 0.66, 0.45);
pub const GROUND_DECOR: Color = Color::srgb(0.50, 0.44, 0.36);
pub const GROUND_NOMANS: Color = Color::srgb(0.20, 0.33, 0.55);

// Torch placement on the new building models (torches stay procedural so the
// day/night system can light them; tune to sit them on the art).
pub const BASE_TORCH_RADIUS: f32 = 0.85;
pub const BASE_TORCH_POLE_Y: f32 = 1.7;
pub const TOWER_TORCH_POLE_Y: f32 = 1.9;
pub const TOWER_TORCH_FORWARD: f32 = 0.5;

pub const TORCH_INTENSITY: f32 = 250_000.0;
pub const TORCH_RANGE: f32 = 10.0;
pub const TORCH_COLOR: Color = Color::srgb(1.0, 0.65, 0.30);

pub const SUN_DAY_PERIOD: f32 = 90.0;
pub const SUN_DISTANCE: f32 = 55.0;

// Camera
/// Eye position of the fixed 3/4 game view. Both `setup_world` and the debug
/// camera's "reset" use this so they can't drift apart.
///
/// Dollied straight back (×1.5 along the eye→target ray) from the old
/// (0,13,30) when the bases were widened ±14→±21, so the battlefield keeps the
/// same on-screen size. Caveat: the eye (17.5) now sits above the tallest peaks
/// (~13.9), so mountains read as ground-beyond rather than silhouetted against
/// the sky — the height was previously kept below the peaks for that skyline.
pub const CAMERA_DEFAULT_POS: Vec3 = Vec3::new(0.0, 17.5, 45.0);
/// Look-at point of the default view. Above the ground (not the origin) to tilt
/// the view up so the horizon enters the frame (otherwise the sky is above the
/// top edge and the area behind the peaks reads as dark "void"). Chosen with the
/// eye height for a ~13° downward pitch (`atan((11-4)/30)`): horizon ~top fifth,
/// battlefield just below centre. Raise its Y for more sky / a flatter view,
/// lower it to centre the battlefield higher.
pub const CAMERA_DEFAULT_TARGET: Vec3 = Vec3::new(0.0, 4.0, 0.0);
/// Free-fly debug camera (mouse + keyboard). Base WASD/fly speed in units/s.
pub const DEBUG_CAM_BASE_SPEED: f32 = 12.0;
/// Radians of rotation per pixel of mouse motion while looking around.
pub const DEBUG_CAM_SENSITIVITY: f32 = 0.003;
