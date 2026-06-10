use avian3d::prelude::*;
use bevy::prelude::*;

pub const BASE_HP: i32 = 40;

// Soldier
pub const SOLDIER_HP: i32 = 10;
pub const SOLDIER_DAMAGE: i32 = 3;
pub const SOLDIER_COST: u32 = 1;
pub const SOLDIER_SPEED: f32 = 1.8;
pub const SOLDIER_COOLDOWN: f32 = 1.0;

// Miner
pub const MINER_HP: i32 = 8;
pub const MINER_COST: u32 = 4;
pub const MINER_SPEED: f32 = 1.4;
pub const MINER_COOLDOWN: f32 = 1.1;
pub const MINER_GOLD_PER_HIT: u32 = 1;
pub const MAX_MINERS_PER_PLAYER: usize = 5;
/// Gold a miner deposits per round-trip. At 1 (with `MINER_GOLD_PER_HIT` = 1)
/// the miner returns after a single swing, so the economy is deliberately slow.
pub const MINER_CAPACITY: u32 = 1;
pub const MINER_RING_RADIUS: f32 = 1.6;
pub const MINER_DEPOSIT_RANGE: f32 = 1.4;
/// How close (XZ) a velocity-driven miner must get to its mining slot before it
/// stops and starts swinging. Replaces the old exact transform snap-to-slot.
pub const MINER_ARRIVE_RANGE: f32 = 0.25;

// Priest — support unit: no attack, heals and armors a nearby ally.
pub const PRIEST_HP: i32 = 9;
pub const PRIEST_COST: u32 = 5;
pub const PRIEST_SPEED: f32 = 1.5;
/// Seconds between casts (one cast clip per cooldown).
pub const PRIEST_COOLDOWN: f32 = 2.0;
/// The priest stops and supports the nearest ally ahead within this range.
pub const PRIEST_RANGE: f32 = 3.0;
pub const PRIEST_SPAWN_OFFSET: f32 = 1.5;
/// HP restored to the target ally per cast (clamped to its max).
pub const PRIEST_HEAL: i32 = 3;
/// Flat damage reduction granted to the target ally per cast.
pub const PRIEST_ARMOR: i32 = 2;
/// Seconds the armor buff lasts (refreshed on every cast).
pub const PRIEST_ARMOR_DURATION: f32 = 5.0;
/// Floor so armor never makes a unit invincible: every hit deals at least this.
pub const MIN_DAMAGE: i32 = 1;

// Archer
pub const ARCHER_HP: i32 = 7;
pub const ARCHER_DAMAGE: i32 = 2;
pub const ARCHER_COST: u32 = 3;
pub const ARCHER_SPEED: f32 = 1.5;
pub const ARCHER_COOLDOWN: f32 = 1.7;
pub const ARCHER_RANGE: f32 = 8.0;
pub const ARCHER_SPAWN_OFFSET: f32 = 1.5;
/// The archer is rendered from a rigged glTF model (Meshy export) instead of
/// the procedural capsule rig the soldier/miner use. The mesh/skeleton come from
/// the Walking file's scene; each clip is loaded from its own one-animation file
/// (Meshy scrambles the internal animation names, so the file path — not the
/// name — is the source of truth). All files share the same rig, so the clips
/// retarget onto the scene's skeleton.
pub const ARCHER_SCENE_PATH: &str =
    "models/adamar/characters/adamar_archer_biped_Animation_Walking_withSkin.glb";
pub const ARCHER_WALK_PATH: &str = ARCHER_SCENE_PATH;
pub const ARCHER_SHOT_PATH: &str =
    "models/adamar/characters/adamar_archer_biped_Animation_Archery_Shot_withSkin.glb";
pub const ARCHER_DEATH_PATH: &str =
    "models/adamar/characters/adamar_archer_biped_Animation_Shot_in_the_Back_and_Fall_withSkin.glb";
/// The glTF already bakes the Mixamo cm→m 0.01 at its Armature root, so the
/// scene instances ~1.8 units tall on its own. This extra factor brings the
/// archer down to roughly the procedural units' height (~1.3 world units).
pub const ARCHER_MODEL_SCALE: f32 = 0.7;
/// The model faces +Z in its own space; the game's forward is +X. This yaw on
/// the SceneRoot child maps model-forward onto the unit's facing direction.
pub const ARCHER_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
/// The Meshy `Archery_Shot` clip releases the arrow toward the model's left
/// rather than straight ahead. To make the shot read as aimed at the target we
/// rotate the whole archer by this offset when attacking, so its left side
/// faces the target and the leftward release visually points at it. Derived:
/// the model's left maps to the entity's `-Z`, so the entity yaw that puts `-Z`
/// on the target is `target_angle - FRAC_PI_2`.
pub const ARCHER_SHOT_YAW_OFFSET: f32 = -std::f32::consts::FRAC_PI_2;
/// The archer plays a full "shot in the back and fall" clip on death, longer
/// than the generic `DEATH_DURATION`; hold the corpse until it lands.
pub const ARCHER_DEATH_DURATION: f32 = 1.8;
/// How fast (rad/s) the archer pivots toward its target before shooting (and
/// back toward the advance direction when it departs). The `Idle_Turn_*` clips
/// play while this rotation is in progress.
pub const ARCHER_TURN_SPEED: f32 = 6.0;
/// Below this facing error (rad) the archer is considered aimed: it stops
/// turning and may shoot / walk.
pub const ARCHER_TURN_EPS: f32 = 0.06;
/// How long (s) the archer keeps playing the shot animation after its target
/// briefly leaves range, so the pose doesn't flicker to idle between volleys.
pub const ARCHER_ATTACK_HOLD: f32 = 0.6;
/// Fraction through the `Archery_Shot` clip at which the arrow leaves the bow.
/// The clip ends with the archer lowering the bow arm, so releasing slightly
/// before the end (rather than at the cycle boundary) reads as the actual loose.
pub const ARCHER_SHOT_RELEASE_FRACTION: f32 = 0.78;
/// Extra lead (real seconds) before the release point, so the arrow leaves a
/// touch earlier than the pose would suggest. Converted into clip-time with the
/// clip's playback speed where it is applied (`animate_unit_model`).
pub const ARCHER_SHOT_RELEASE_LEAD: f32 = 0.1;
/// Name of the skeleton bone the arrow leaves from — the bow (left) hand. The
/// Meshy rig keeps standard bone names even though it scrambles clip names.
pub const ARCHER_BOW_HAND_BONE: &str = "LeftHand";
/// Fallback arrow origin used only for the frame or two before the `LeftHand`
/// bone is resolved, in the archer entity's local frame (model rigidly parented
/// at `ARCHER_MODEL_YAW_OFFSET`). Negative Z = the model's left side, where the
/// bow hand is; matches `ARCHER_SHOT_YAW_OFFSET`.
pub const ARCHER_HAND_OFFSET: Vec3 = Vec3::new(0.3, 0.95, -0.35);
/// Path of the bow mesh attached to the archer's left hand.
pub const ARCHER_BOW_PATH: &str = "models/adamar/weapons/adamar_bow.glb";
/// Local transform applied to the bow scene once parented to the `LeftHand` bone.
/// The `LeftHand` bone inherits the skeleton root's cm→m scale (`0.01`) times
/// `ARCHER_MODEL_SCALE` (`0.7`), i.e. a world scale of ~0.007. The bow mesh is
/// ~1.9 m tall in its own glTF space, so at scale 1.0 it would render at ~1.3 cm
/// (invisible). This factor cancels that shrink and sizes the bow to the archer.
/// Tune if the bow sits slightly off once the real rig is visible.
pub const ARCHER_BOW_SCALE: f32 = 63.0;
/// Local rotation of the bow within the `LeftHand` bone frame, as Euler XYZ in
/// radians (applied `Quat::from_euler(EulerRot::XYZ, x, y, z)`). The Meshy hand
/// bone's axes don't line up with the bow mesh, so without this the bow lies flat
/// against the forearm instead of standing perpendicular with the limbs vertical.
/// Tune these three angles visually — they are the bow's own orientation in hand.
pub const ARCHER_BOW_ROTATION: Vec3 = Vec3::new(
    0.0,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
/// Extra 180°-style spin about the bow's *own* long (vertical) axis, applied as a
/// right-multiply after `ARCHER_BOW_ROTATION`. Use this — not the placement Euler
/// — to flip the bow when it reads "à l'envers" (belly/back or tips reversed),
/// since it rotates the mesh on itself rather than around the hand-bone frame.
pub const ARCHER_BOW_SELF_FLIP: f32 = std::f32::consts::PI;
pub const ARCHER_BOW_OFFSET: Vec3 = Vec3::ZERO;
/// The bow is gripped at its middle, so no shift along its long axis.
pub const ARCHER_BOW_GRIP: f32 = 0.0;

// ─────────────────────────────────────────────────────────────────────────────
// Soldier / Miner / Priest glTF models. Same Meshy convention as the archer
// (0.01 cm→m baked at the Armature root, +Z model-forward → game +X via a yaw
// offset). One AnimationClip per file; the file path is the source of truth
// (Meshy scrambles internal clip names). Loaded into `UnitModels` by
// `setup::load_unit_models`; clip→graph built by `setup::build_unit_graphs`.
// ─────────────────────────────────────────────────────────────────────────────
pub const SOLDIER_SCENE_PATH: &str =
    "models/adamar/characters/adamar_soldier_biped_Animation_Walking_withSkin.glb";
pub const SOLDIER_WALK_PATH: &str = SOLDIER_SCENE_PATH;
pub const SOLDIER_ATTACK_PATH: &str =
    "models/adamar/characters/adamar_soldier_biped_Animation_Left_Slash_withSkin.glb";
pub const SOLDIER_DEATH_PATH: &str = "models/adamar/characters/adamar_soldier_biped_Animation_Fall_Dead_from_Abdominal_Injury_withSkin.glb";
pub const SOLDIER_MODEL_SCALE: f32 = 0.7;
pub const SOLDIER_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
pub const SOLDIER_DEATH_DURATION: f32 = 1.4;

pub const MINER_SCENE_PATH: &str =
    "models/adamar/characters/adamar_miner_biped_Animation_Walking_withSkin.glb";
pub const MINER_WALK_PATH: &str = MINER_SCENE_PATH;
/// The miner's "attack" clip is the mining swing (no enemy combat).
pub const MINER_ATTACK_PATH: &str =
    "models/adamar/characters/adamar_miner_biped_Animation_Heavy_Hammer_Swing_withSkin.glb";
pub const MINER_MODEL_SCALE: f32 = 0.7;
pub const MINER_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;

pub const PRIEST_SCENE_PATH: &str =
    "models/adamar/characters/adamar_priest_biped_Animation_Walking_withSkin.glb";
pub const PRIEST_WALK_PATH: &str = PRIEST_SCENE_PATH;
/// The priest's "attack" clip is the spell cast (heal + armor, no damage).
pub const PRIEST_ATTACK_PATH: &str =
    "models/adamar/characters/adamar_priest_biped_Animation_mage_spell_cast_1_withSkin.glb";
pub const PRIEST_DEATH_PATH: &str =
    "models/adamar/characters/adamar_priest_biped_Animation_Shot_and_Fall_Backward_withSkin.glb";
pub const PRIEST_MODEL_SCALE: f32 = 0.7;
pub const PRIEST_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
pub const PRIEST_DEATH_DURATION: f32 = 1.8;

// Hand weapons/tools. Each attaches to a skeleton hand bone whose world scale is
// ~0.007 (0.7 model × 0.01 armature), so weapon `SCALE` is large like the bow.
// `ROTATION`/`SELF_FLIP` follow the bow's semantics (Euler in the bone frame,
// then a spin about the weapon's own long axis); all need visual tuning.
// `ROTATION` = `(0, π/2, π/2)` mirrors the (working) bow placement — two 90°
// axes that stand the weapon upright in the hand instead of lying horizontal.
// `SELF_FLIP` spins it about its own long axis (which face/end points out).
// `GRIP` (~0.85) slides the handle into the hand so the blade/head doesn't pass
// through the wrist. All still need visual tuning per weapon.
// `ROTATION.x = π` flips the weapon to point up/forward (it pointed down/back at
// x = 0). `ROTATION.{y,z} = π/2` is the bow-style two-axis upright placement.
// `SELF_FLIP` rolls it about its own long axis. `GRIP`: which mesh end sits in
// the hand (sword's handle is the opposite end from the pickaxe/staff → negative;
// 0 = held at the centre, e.g. the staff).
// `OFFSET` nudges the weapon along the hand bone (≈ +Y → toward the fingers) so
// it sits in the hand instead of anchored at the wrist (bone-local units: the
// bone's world scale is ~0.007, so ~10 ≈ 7 cm).
pub const HAND_OFFSET: Vec3 = Vec3::new(0.0, 10.0, 0.0);

pub const SWORD_PATH: &str = "models/adamar/weapons/adamar_sword.glb";
pub const SWORD_BONE: &str = "RightHand";
pub const SWORD_SCALE: f32 = 63.0;
pub const SWORD_OFFSET: Vec3 = HAND_OFFSET;
// x = 0 (not π): the sword grips the opposite mesh end (negative GRIP) which
// reverses its extension, so it points up/forward like the others without the flip.
pub const SWORD_ROTATION: Vec3 = Vec3::new(
    0.0,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
pub const SWORD_SELF_FLIP: f32 = std::f32::consts::PI;
pub const SWORD_GRIP: f32 = -0.55;

pub const PICKAXE_PATH: &str = "models/adamar/weapons/adamar_pickaxe.glb";
pub const PICKAXE_BONE: &str = "RightHand";
pub const PICKAXE_SCALE: f32 = 35.3;
pub const PICKAXE_OFFSET: Vec3 = HAND_OFFSET;
pub const PICKAXE_ROTATION: Vec3 = Vec3::new(
    std::f32::consts::PI,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
pub const PICKAXE_SELF_FLIP: f32 = std::f32::consts::PI;
pub const PICKAXE_GRIP: f32 = 0.65;

pub const STAFF_PATH: &str = "models/adamar/weapons/adamar_staff.glb";
pub const STAFF_BONE: &str = "RightHand";
pub const STAFF_SCALE: f32 = 63.0;
pub const STAFF_OFFSET: Vec3 = HAND_OFFSET;
pub const STAFF_ROTATION: Vec3 = Vec3::new(
    std::f32::consts::PI,
    std::f32::consts::FRAC_PI_2,
    std::f32::consts::FRAC_PI_2,
);
pub const STAFF_SELF_FLIP: f32 = std::f32::consts::PI + std::f32::consts::FRAC_PI_2;
pub const STAFF_GRIP: f32 = 0.0;

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
/// Radius kept free of props around each possible mining-rock position. The
/// rocks sit at x ≈ ±(base + ROCK_OFFSET), *outside* the keep-clear rectangle,
/// so without this a prop could spawn on the rock or the miners' arc.
pub const SCENERY_ROCK_CLEARANCE: f32 = 4.0;

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

// Tower
pub const TOWER_HP: i32 = 30;
pub const TOWER_DAMAGE: i32 = 3;
pub const TOWER_COST: u32 = 6;
pub const TOWER_RANGE: f32 = 8.5;
pub const TOWER_COOLDOWN: f32 = 1.5;
pub const TOWER_HEIGHT: f32 = 2.6;
pub const TOWER_RADIUS: f32 = 0.7;
pub const TOWER_MIN_SEPARATION: f32 = 1.8;
pub const TOWER_ARROW_HEIGHT: f32 = 2.1;

// Arrows
pub const ARROW_TRAVEL_SPEED: f32 = 7.0;
pub const ARROW_ARC_FRACTION: f32 = 0.22;
pub const ARROW_MIN_ARC: f32 = 1.0;
/// Volley targeting: for each enemy in range an archer counts the enemies within
/// this radius and aims at the centroid of the densest such knot. The arrow then
/// damages any enemy it physically passes through.
pub const VOLLEY_RADIUS: f32 = 2.5;
/// Sphere radius the flying arrow is tested with (via `SpatialQuery`) against
/// enemy colliders each frame.
pub const ARROW_HIT_RADIUS: f32 = 0.3;
/// Seconds a missed arrow stays planted in the ground before despawning.
pub const ARROW_STICK_DURATION: f32 = 3.0;

pub const STARTING_GOLD: u32 = 10;
pub const ENGAGE_RANGE: f32 = 1.4;

// ─────────────────────────────────────────────────────────────────────────────
// Unit AI (combat_tick). Units march straight toward the enemy base and only
// peel off to fight an enemy that comes within a short aggro radius; they answer
// fire from an attacker even with no target in sight.
// ─────────────────────────────────────────────────────────────────────────────
/// Units march straight by default and only redirect toward an enemy (unit or
/// tower) that comes within this short radius — the "redirect only at relatively
/// short distance" rule.
pub const AGGRO_RADIUS: f32 = 3.0;
/// Once committed to a target, a unit keeps chasing it until the target dies or
/// moves beyond this (larger) distance — prevents yo-yoing at the aggro edge.
pub const TARGET_LEASH: f32 = 5.5;
/// An idle unit (no target in view) that gets hit charges its attacker if the
/// attacker is within this distance (lets a marching unit answer ranged fire).
pub const RETALIATE_LEASH: f32 = 8.0;
/// While marching with no enemy target, a unit goes dead-straight until it is
/// within this distance of the enemy base ON THE MARCH (X) AXIS, then steers onto
/// it to attack. X-based (not radial) so units in a far lane still converge onto
/// the base instead of marching straight past it.
pub const BASE_SEEK_RANGE: f32 = 8.0;
/// When attacking, a soldier closes only to this distance — just past body
/// contact (`2·UNIT_RADIUS`) — so it strikes a target instead of shoving it,
/// while still creeping in to follow a kiting target and stay in reach.
pub const MELEE_STANDOFF: f32 = UNIT_RADIUS * 2.0 + 0.25;

// ─────────────────────────────────────────────────────────────────────────────
// Battalion formation. Self-organized by simple per-unit rules in `combat_tick`
// (no global controller): while a unit marches with no enemy target, its forward
// speed is scaled by the SMALLEST of two graduated slow-downs —
//   1. RANK COHESION: it slows the further it has pulled ahead of nearby
//      SAME-ROLE peers (weighted toward lateral "lane" neighbours), so each rank
//      advances abreast — "ralentit pour chaque voisin, surtout ses voisins de
//      couloir devant lui";
//   2. RANGED PACING (archer/priest only): it keeps a per-role gap behind the
//      nearest allied SOLDIER *ahead of it*, slowing only when it crowds that
//      soldier — so it paces the soldiers (never freezes) and the army layers by
//      range (soldiers → priests → archers).
// All comparisons are limited to allies within `FORMATION_RADIUS`, which keeps
// the behaviour local: a straggler outside that radius never stalls the front.
// ─────────────────────────────────────────────────────────────────────────────
/// How far (XZ) a marching unit "sees" allies for its formation decisions.
pub const FORMATION_RADIUS: f32 = 6.0;
/// Gap (march axis) a marching priest keeps behind the nearest soldier ahead.
/// Shorter range than the archer ⇒ sits closer to the front.
pub const FORMATION_PRIEST_GAP: f32 = 1.8;
/// Gap a marching archer keeps behind the nearest soldier ahead — larger than the
/// priest's so the longest-range role ends up at the back ("par portée").
pub const FORMATION_ARCHER_GAP: f32 = 3.5;
/// How far ahead (march axis) a unit must be of its reference before its speed
/// bottoms out at `FORMATION_MIN_FACTOR`. Smaller ⇒ tighter, snappier ranks.
pub const FORMATION_DECAY: f32 = 2.0;
/// Floor of the formation speed factor: a waiting unit still creeps forward at
/// this fraction of its speed (never a dead stop, so it never looks frozen).
pub const FORMATION_MIN_FACTOR: f32 = 0.3;
/// Lateral (Z) distance over which a same-rank neighbour's pull fades. Neighbours
/// beside you (your "lane" flankers) weigh most when deciding to wait for them.
pub const FORMATION_LATERAL_RANGE: f32 = 3.0;
/// Minimum weight kept for a same-rank neighbour far away in Z, so distant-lane
/// peers still count a little toward rank cohesion.
pub const FORMATION_LATERAL_FLOOR: f32 = 0.15;

pub const LEFT_BASE_X: f32 = -21.0;
pub const RIGHT_BASE_X: f32 = 21.0;
// Z offset between the two bases of a same side in 2v2 mode (scaled with the
// widened lateral battlefront so the two allied lanes stay separated).
pub const BASE_Z_OFFSET: f32 = 9.0;
// Half-width of the central neutral no-man's-land. Pinned to an absolute size
// (the old 28/6 derived from ±14 bases) and NOT recomputed from the base
// separation, so moving the bases farther apart lengthens each side's buildable
// zone while the no-man's-land keeps the same footprint.
pub const ZONE_BOUNDARY: f32 = 14.0 / 3.0;
pub const TOWER_PLACEMENT_MARGIN: f32 = 1.6;
/// Z half-extent of the placement zone in 1v1: lanes are centred on Z=0 so the
/// usable strip is narrow. Wider would let you drop a tower way off any lane.
pub const TOWER_PLACEMENT_Z_LIMIT_1V1: f32 = LANE_HALF_WIDTH_1V1 + 0.4;
/// Z half-extent in 2v2: each side has two bases spread on Z, with lanes
/// reaching `BASE_Z_OFFSET + LANE_HALF_WIDTH_2V2`. The limit covers both.
pub const TOWER_PLACEMENT_Z_LIMIT_2V2: f32 = BASE_Z_OFFSET + LANE_HALF_WIDTH_2V2 + 0.5;

pub const GAMEPAD_STICK_DEADZONE: f32 = 0.25;
pub const GAMEPAD_CURSOR_SPEED: f32 = 6.0;
pub const PLAYER_PANEL_SLOTS: usize = 5;

pub const UNIT_RADIUS: f32 = 0.35;
pub const SOLDIER_SPAWN_OFFSET: f32 = 1.5;
/// Number of parallel lanes (Z offsets) units cycle through on spawn so that
/// successive same-kind units don't pile on top of each other.
pub const LANE_COUNT: usize = 5;
/// Half-width of the lateral **battlefront** in 1v1 (NOT the spawn spread):
/// drives the tower z-limit and the sand ground band. Widened ×3 for a roomy
/// field. Units now spawn in a tight column (`SPAWN_LANE_HALF_WIDTH_*`) and
/// fan out into a battalion via the formation rules, so this is decoupled from
/// where they exit the base.
pub const LANE_HALF_WIDTH_1V1: f32 = 7.8;
/// Half-width of the battlefront in 2v2: tighter than 1v1 so each ally's half of
/// the field doesn't bleed into the other (their bases are `BASE_Z_OFFSET` apart).
pub const LANE_HALF_WIDTH_2V2: f32 = 4.5;
/// Half-width of the **spawn** lane spread. Deliberately tight so units leave the
/// base clustered (a column at the gate) instead of fanning across the whole
/// battlefront; the per-unit formation rules then organize them into a battalion.
pub const SPAWN_LANE_HALF_WIDTH_1V1: f32 = 2.5;
pub const SPAWN_LANE_HALF_WIDTH_2V2: f32 = 1.6;
pub const MINER_SPAWN_OFFSET: f32 = 1.0;
pub const ROCK_OFFSET: f32 = 8.25;
/// Spread applied to non-laned units' Z at spawn so consecutive same-side
/// spawns don't appear in a perfect line. ±half the value, around the slot's
/// base Z.
pub const SPAWN_Z_JITTER: f32 = 0.6;

// ─────────────────────────────────────────────────────────────────────────────
// Physics colliders (Avian). Units are dynamic capsules driven by LinearVelocity;
// bases/towers/rocks are static obstacles. Avian resolves all overlap (units push
// apart) and blocking, replacing the old manual queue/sidestep logic.
// ─────────────────────────────────────────────────────────────────────────────
/// Cylindrical part of the unit capsule (total height ≈ LENGTH + 2·UNIT_RADIUS).
pub const UNIT_CAPSULE_LENGTH: f32 = 0.6;
/// Base obstacle radius. Kept below `ENGAGE_RANGE − UNIT_RADIUS` so attackers can
/// still reach melee at the wall (refined with a base engage range in step 3).
pub const BASE_COLLIDER_RADIUS: f32 = 0.9;
pub const BASE_COLLIDER_HEIGHT: f32 = 3.0;
pub const ROCK_COLLIDER_RADIUS: f32 = 0.9;
pub const ROCK_COLLIDER_HEIGHT: f32 = 2.0;

// ─────────────────────────────────────────────────────────────────────────────
// Unit animation timing. All units are now rigged glTF models driven by
// `animate_unit_model`; these are the shared timing values it reads.
// ─────────────────────────────────────────────────────────────────────────────
/// Generic seconds a corpse is held before despawn, for kinds without a longer
/// dedicated fall clip duration (see per-kind `*_DEATH_DURATION`).
pub const DEATH_DURATION: f32 = 0.6;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerSlot {
    LeftBottom,
    LeftTop,
    RightBottom,
    RightTop,
}

impl PlayerSlot {
    pub const ALL: [PlayerSlot; 4] = [
        PlayerSlot::LeftBottom,
        PlayerSlot::LeftTop,
        PlayerSlot::RightBottom,
        PlayerSlot::RightTop,
    ];

    pub fn side(self) -> Side {
        match self {
            PlayerSlot::LeftBottom | PlayerSlot::LeftTop => Side::Left,
            PlayerSlot::RightBottom | PlayerSlot::RightTop => Side::Right,
        }
    }

    pub fn is_top(self) -> bool {
        matches!(self, PlayerSlot::LeftTop | PlayerSlot::RightTop)
    }

    pub fn base_z(self, mode: GameMode) -> f32 {
        match mode {
            GameMode::OneVsOne => 0.0,
            GameMode::TwoVsTwo => {
                if self.is_top() {
                    -BASE_Z_OFFSET
                } else {
                    BASE_Z_OFFSET
                }
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PlayerSlot::LeftBottom => "Left",
            PlayerSlot::LeftTop => "Left Top",
            PlayerSlot::RightBottom => "Right",
            PlayerSlot::RightTop => "Right Top",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    OneVsOne,
    TwoVsTwo,
}

impl GameMode {
    pub fn active_slots(self) -> &'static [PlayerSlot] {
        match self {
            GameMode::OneVsOne => &[PlayerSlot::LeftBottom, PlayerSlot::RightBottom],
            GameMode::TwoVsTwo => &[
                PlayerSlot::LeftBottom,
                PlayerSlot::LeftTop,
                PlayerSlot::RightBottom,
                PlayerSlot::RightTop,
            ],
        }
    }

    pub fn tower_z_limit(self) -> f32 {
        match self {
            GameMode::OneVsOne => TOWER_PLACEMENT_Z_LIMIT_1V1,
            GameMode::TwoVsTwo => TOWER_PLACEMENT_Z_LIMIT_2V2,
        }
    }

    /// Half-width of the (much tighter) spawn lane spread — where units exit the
    /// base, before the formation rules fan them out into a battalion.
    pub fn spawn_lane_half_width(self) -> f32 {
        match self {
            GameMode::OneVsOne => SPAWN_LANE_HALF_WIDTH_1V1,
            GameMode::TwoVsTwo => SPAWN_LANE_HALF_WIDTH_2V2,
        }
    }
}

impl Side {
    pub fn forward(self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }

    pub fn color(self) -> Color {
        self.color_for(false)
    }

    pub fn color_dark(self) -> Color {
        self.color_dark_for(false)
    }

    /// Side colour respecting the colorblind toggle. Standard palette pits
    /// blue vs. red; the colorblind variant swaps Right to orange so the two
    /// sides remain distinguishable under deuteranopia/protanopia.
    pub fn color_for(self, colorblind: bool) -> Color {
        match (self, colorblind) {
            (Side::Left, _) => Color::srgb(0.25, 0.55, 1.0),
            (Side::Right, false) => Color::srgb(1.0, 0.40, 0.35),
            (Side::Right, true) => Color::srgb(1.0, 0.68, 0.10),
        }
    }

    pub fn color_dark_for(self, colorblind: bool) -> Color {
        match (self, colorblind) {
            (Side::Left, _) => Color::srgb(0.14, 0.32, 0.70),
            (Side::Right, false) => Color::srgb(0.70, 0.24, 0.20),
            (Side::Right, true) => Color::srgb(0.65, 0.42, 0.05),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Side::Left => "Left",
            Side::Right => "Right",
        }
    }

    pub fn base_rotation(self) -> Quat {
        match self {
            Side::Left => Quat::IDENTITY,
            Side::Right => Quat::from_rotation_y(std::f32::consts::PI),
        }
    }

    pub fn opposite(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    fn unit_layer(self) -> GameLayer {
        match self {
            Side::Left => GameLayer::UnitLeft,
            Side::Right => GameLayer::UnitRight,
        }
    }

    fn struct_layer(self) -> GameLayer {
        match self {
            Side::Left => GameLayer::StructLeft,
            Side::Right => GameLayer::StructRight,
        }
    }

    /// Units collide with every unit (separation) and every structure (blocking).
    pub fn unit_layers(self) -> CollisionLayers {
        CollisionLayers::new(
            self.unit_layer(),
            [
                GameLayer::UnitLeft,
                GameLayer::UnitRight,
                GameLayer::StructLeft,
                GameLayer::StructRight,
            ],
        )
    }

    /// Static obstacles (base/tower/rock) block only ENEMY units: allied units
    /// pass through their own buildings, so a unit marching straight never gets
    /// stuck on its own tower (no manual sidestep needed), while enemy units are
    /// blocked — the defensive value of a tower.
    pub fn structure_layers(self) -> CollisionLayers {
        CollisionLayers::new(self.struct_layer(), [self.opposite().unit_layer()])
    }

    /// Collision-layer mask of the ENEMY side's units and structures — what an
    /// arrow fired by this side may strike. Used as the `SpatialQuery` filter mask
    /// in `arrow_flight_system`, so arrows hit only enemies (no friendly fire) and
    /// never need a collider of their own.
    pub fn arrow_target_mask(self) -> LayerMask {
        let enemy = self.opposite();
        [enemy.unit_layer(), enemy.struct_layer()].into()
    }
}

/// Physics collision layers. Separate per-side layers let units separate from
/// allies while (in step 4) arrows filter to the enemy only. Bit 0 (`Default`)
/// is the layer everything unassigned falls into.
#[derive(PhysicsLayer, Default, Clone, Copy)]
pub enum GameLayer {
    #[default]
    Default,
    UnitLeft,
    UnitRight,
    StructLeft,
    StructRight,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Soldier,
    Miner,
    Archer,
    Priest,
}

/// All unit kinds, in `index()` order — used to build/iterate the per-kind
/// `UnitModels` table.
pub const UNIT_KINDS: [UnitKind; 4] = [
    UnitKind::Soldier,
    UnitKind::Miner,
    UnitKind::Archer,
    UnitKind::Priest,
];

impl UnitKind {
    /// Stable index into `UnitModels.models`.
    pub fn index(self) -> usize {
        match self {
            UnitKind::Soldier => 0,
            UnitKind::Miner => 1,
            UnitKind::Archer => 2,
            UnitKind::Priest => 3,
        }
    }
}

#[derive(Component)]
pub struct Base;

/// Added to a `Base` whose HP hit 0. Targeting filters (`combat_tick`,
/// `tower_attack_tick`) exclude these so allied units retarget the surviving
/// enemy base, and the HUD greys out the owning player's panel. Its collider is
/// removed on insertion (a ruin neither blocks units nor soaks arrows) and `t`
/// drives the sink animation in `collapse_destroyed_bases`.
#[derive(Component, Default)]
pub struct BaseDestroyed {
    pub t: f32,
}

/// Seconds the destroyed-base sink animation lasts.
pub const BASE_COLLAPSE_DURATION: f32 = 2.5;
/// How deep (world units) a destroyed base sinks into the ground.
pub const BASE_COLLAPSE_SINK: f32 = 2.2;

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Rock;

#[derive(Component)]
pub struct Tower;

/// Marker added when a tower's HP hits 0. Drives a brief collapse animation
/// (tilt + sink) over `TOWER_DEATH_DURATION` before the entity is despawned.
#[derive(Component, Default)]
pub struct TowerDying {
    pub t: f32,
}

pub const TOWER_DEATH_DURATION: f32 = 0.45;

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct TorchLight;

#[derive(Component)]
pub struct TorchFlame;

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

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    #[default]
    Day,
    Night,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct DlssAvailable(pub bool);

#[derive(Resource, Default, Clone, Copy)]
pub struct RaytracingAvailable(pub bool);

#[derive(Resource, Default, Clone, Copy)]
pub struct GameTime(pub f32);

#[derive(Resource)]
pub struct AtmosphereHandle(pub Handle<bevy::pbr::ScatteringMedium>);

#[derive(Component, Default)]
pub struct MinerCarry {
    pub current: u32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MinerPhase {
    ToRock,
    Mining,
    Returning,
}

#[derive(Component, Clone, Copy)]
pub struct MinerSlot(pub usize);

/// Per-unit targeting memory for `combat_tick`'s AI: the enemy the unit is
/// currently committed to (acquired within `AGGRO_RADIUS`, kept until it dies or
/// leaves `TARGET_LEASH`) and the last enemy that damaged it (so an idle unit
/// retaliates against an attacker within `RETALIATE_LEASH`).
#[derive(Component, Default)]
pub struct CombatTarget {
    pub current: Option<Entity>,
    pub last_attacker: Option<Entity>,
}

#[derive(Component)]
pub struct TowerGhost;

#[derive(Default, Clone, Copy)]
pub struct PlacementSeat {
    pub world_pos: Vec3,
    pub armed: bool,
}

#[derive(Resource, Default)]
pub struct PlacementMode {
    seats: [Option<PlacementSeat>; 4],
}

impl PlacementMode {
    pub fn get(&self, slot: PlayerSlot) -> Option<PlacementSeat> {
        self.seats[slot.index()]
    }

    pub fn set(&mut self, slot: PlayerSlot, seat: PlacementSeat) {
        self.seats[slot.index()] = Some(seat);
    }

    pub fn clear(&mut self, slot: PlayerSlot) {
        self.seats[slot.index()] = None;
    }
}

#[derive(Resource, Default)]
pub struct PlayerControllers {
    entities: [Option<Entity>; 4],
}

impl PlayerControllers {
    pub fn get(&self, slot: PlayerSlot) -> Option<Entity> {
        self.entities[slot.index()]
    }

    pub fn set(&mut self, slot: PlayerSlot, entity: Option<Entity>) {
        self.entities[slot.index()] = entity;
    }
}

#[derive(Resource, Default)]
pub struct MenuFocus {
    pub index: usize,
}

/// Playable nations. Only one exists today; the enum + `ALL` make adding more a
/// one-line change and the SideSelect nation picker iterates `ALL`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Nation {
    #[default]
    AdaRam,
}

impl Nation {
    pub const ALL: &'static [Nation] = &[Nation::AdaRam];

    pub fn label(self) -> &'static str {
        match self {
            Nation::AdaRam => "Ada'Ram",
        }
    }
}

/// Per-`PlayerSlot` nation chosen on the SideSelect screen, committed when the
/// match starts (mirrors `PlayerControllers`/`Gold`).
#[derive(Resource, Default)]
pub struct PlayerNations {
    nations: [Nation; 4],
}

impl PlayerNations {
    // Read by gameplay once nations diverge; unused while only Ada'Ram exists.
    #[allow(dead_code)]
    pub fn get(&self, slot: PlayerSlot) -> Nation {
        self.nations[slot.index()]
    }

    pub fn set(&mut self, slot: PlayerSlot, nation: Nation) {
        self.nations[slot.index()] = nation;
    }
}

/// Where a pad is in the SideSelect flow: hovering a seat, then (after claiming
/// it) picking a nation, then fully locked in and counted for launch.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SeatPhase {
    PickingSeat,
    PickingNation,
    Locked,
}

#[derive(Component, Clone, Copy)]
pub struct SeatSelection {
    pub hovered: PlayerSlot,
    pub phase: SeatPhase,
    /// Index into `Nation::ALL` for the nation step.
    pub nation: usize,
}

impl SeatSelection {
    /// The pad has taken the seat (no one else may hover/claim it): it's past
    /// seat selection, choosing a nation or already locked.
    pub fn claims_seat(self) -> bool {
        matches!(self.phase, SeatPhase::PickingNation | SeatPhase::Locked)
    }
}

#[derive(Component, Clone, Copy)]
pub struct PlayerFocus {
    pub slot: PlayerSlot,
    pub index: usize,
}

#[derive(Component)]
pub struct Arrow {
    pub start: Vec3,
    /// Fixed ground point the arrow arcs toward (no homing). It damages any enemy
    /// it passes through; if it reaches the ground untouched it plants here.
    pub aim: Vec3,
    pub elapsed: f32,
    pub total: f32,
    pub apex: f32,
    pub damage: i32,
    /// Who fired it — recorded as the victim's `last_attacker` on hit (so a unit
    /// retaliates against a distant archer/tower that shot it).
    pub shooter: Entity,
    /// Shooter's side — selects which side's units/structures the arrow can hit.
    pub side: Side,
    /// Once it lands without hitting anything it sticks in the ground, ageing by
    /// `stick_t` until `ARROW_STICK_DURATION`.
    pub stuck: bool,
    pub stick_t: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component)]
pub struct Damage(pub i32);

#[derive(Component)]
pub struct MoveSpeed(pub f32);

#[derive(Component)]
pub struct AttackCooldown(pub Timer);

impl AttackCooldown {
    pub fn ready(duration: f32) -> Self {
        let mut t = Timer::from_seconds(duration, TimerMode::Repeating);
        t.set_elapsed(t.duration());
        Self(t)
    }
}

#[derive(Component, Default)]
pub struct UnitAnim {
    pub walking: bool,
    pub attacking: bool,
    pub dying: bool,
    pub death_t: f32,
    /// Desired entity yaw (rotation around Y) for aiming kinds, set by
    /// `combat_tick` (target + `ARCHER_SHOT_YAW_OFFSET` when shooting, the
    /// advance direction otherwise; the priest faces its ally). `animate_unit_model`
    /// smoothly rotates to it for kinds where `uses_face_yaw` is true.
    pub face_yaw: f32,
}

/// Marker on the root of a unit rendered from a glTF model (now every unit).
/// Drives `animate_unit_model` through a descendant `AnimationPlayer`.
#[derive(Component)]
pub struct ModeledUnit;

/// Temporary flat damage reduction granted by the priest. Present on every unit
/// (amount 0 when unbuffed); `tick_armor_buffs` zeroes `amount` when `timer`
/// finishes. Damage sites subtract `amount` (with a `MIN_DAMAGE` floor).
#[derive(Component)]
pub struct Armor {
    pub amount: i32,
    pub timer: Timer,
}

impl Default for Armor {
    fn default() -> Self {
        // Start expired so unbuffed units have no armor.
        let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
        timer.tick(timer.duration());
        Self { amount: 0, timer }
    }
}

impl Armor {
    /// Effective reduction right now (0 once the buff has expired).
    pub fn active(&self) -> i32 {
        if self.timer.is_finished() {
            0
        } else {
            self.amount
        }
    }
}

/// Which logical clip a modeled unit is currently playing. Lets
/// `animate_unit_model` avoid re-issuing `play` every frame.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModelClip {
    #[default]
    None,
    Idle,
    Walk,
    Attack,
    Death,
}

/// A shot queued by `combat_tick` and released by `animate_unit_model` (archer
/// only) when the shot clip reaches its release point. Carries the ground point
/// (densest enemy cluster centroid) the volley is aimed at — the arrow is a dumb
/// projectile that damages whatever enemy it flies through.
#[derive(Clone, Copy)]
pub struct PendingShot {
    pub aim: Vec3,
    pub damage: i32,
}

/// Per-unit animation bookkeeping: the descendant `AnimationPlayer` entity
/// (instanced asynchronously with the scene) plus the small state machine that
/// `animate_unit_model` runs off `UnitAnim`.
#[derive(Component, Default)]
pub struct UnitAnimState {
    pub player: Option<Entity>,
    /// The skeleton hand bone the weapon is parented to (and, for the archer,
    /// the world position arrows leave from). Resolved by `bind_unit_weapon_hand`.
    pub weapon_hand: Option<Entity>,
    pub current: ModelClip,
    /// Countdown that keeps the attack animation playing through brief target
    /// losses (see `ARCHER_ATTACK_HOLD`).
    pub attack_hold: f32,
    /// Attack clip `seek_time()` last frame, so the archer fires one arrow per
    /// cycle the moment playback crosses `ARCHER_SHOT_RELEASE_FRACTION`.
    pub last_attack_seek: f32,
    /// Archer-only: target snapshot to release on the next shot-cycle end.
    pub pending_shot: Option<PendingShot>,
}

/// Indices (and precomputed playback speeds) of one unit kind's clips inside its
/// shared `AnimationGraph`. `attack`/`death` are optional (e.g. the miner has
/// only walk + mining clips).
#[derive(Clone)]
pub struct ModelAnimNodes {
    pub walk: AnimationNodeIndex,
    pub attack: Option<AnimationNodeIndex>,
    pub death: Option<AnimationNodeIndex>,
    /// Speed that makes one loop of the attack clip last the unit's cooldown.
    pub attack_speed: f32,
    /// Clip-local duration (s) of the attack clip (archer release timing).
    pub attack_len: f32,
    /// Speed that makes the fall clip finish within the kind's death duration.
    pub death_speed: f32,
}

/// How a weapon/tool scene is parented to a unit's hand bone. Same semantics as
/// the bow: an Euler placement in the bone frame, then a spin about the weapon's
/// own long axis, plus a (large) scale to cancel the hand bone's tiny world scale.
#[derive(Clone)]
pub struct WeaponDef {
    pub scene: Handle<Scene>,
    pub bone: &'static str,
    pub offset: Vec3,
    pub rotation: Vec3,
    pub self_flip: f32,
    pub scale: f32,
    /// Shift along the weapon's own long axis (its local Y, in mesh units ≈ ±0.95)
    /// applied *after* rotation, so the hand grips an end instead of the centre.
    /// 0 = held at the middle (the bow); ~0.9 lifts the lower end to the hand.
    pub grip: f32,
}

/// All glTF data for one unit kind: the scene (mesh + skeleton), one clip per
/// action (the file path is the source of truth), the hand weapon, and — once
/// the clips decode — the built graph + cached nodes. `attack`/`death` are
/// optional so kinds with fewer clips (miner) are handled.
#[derive(Default, Clone)]
pub struct UnitModel {
    pub scene: Handle<Scene>,
    pub walk: Handle<AnimationClip>,
    pub attack: Option<Handle<AnimationClip>>,
    pub death: Option<Handle<AnimationClip>>,
    pub weapon: Option<WeaponDef>,
    pub graph: Option<Handle<AnimationGraph>>,
    pub nodes: Option<ModelAnimNodes>,
    /// Uniform model scale and the +Z→facing yaw offset applied to the SceneRoot.
    pub scale: f32,
    pub yaw_offset: f32,
    /// Gameplay cooldown (drives `attack_speed`) and corpse-hold duration.
    pub cooldown: f32,
    pub death_duration: f32,
}

/// Per-`UnitKind` glTF models, indexed by `UnitKind::index`. Populated by
/// `setup::load_unit_models`; graphs/nodes filled in by `build_unit_graphs`.
#[derive(Resource, Default)]
pub struct UnitModels {
    pub models: [UnitModel; 4],
}

impl UnitModels {
    pub fn get(&self, kind: UnitKind) -> &UnitModel {
        &self.models[kind.index()]
    }
    pub fn get_mut(&mut self, kind: UnitKind) -> &mut UnitModel {
        &mut self.models[kind.index()]
    }
}

/// Marker on a weapon scene entity parented to a unit's hand bone.
#[derive(Component)]
pub struct UnitWeapon;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Menu,
    Settings,
    SideSelect,
    Playing,
    Paused,
    /// Match over. `Some(side)` is the winner; `None` is a draw (both sides'
    /// last bases fell on the same frame).
    Ended(Option<Side>),
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct GameSettings {
    // Video / display
    pub fullscreen: bool,
    pub vsync: bool,
    pub hdr: bool,
    pub msaa: u8,        // 0=Off 2=2x 4=4x 8=8x
    pub tonemapping: u8, // 0=AcesFitted 1=TonyMcMapface 2=Reinhard 3=None

    // Graphics / quality on-off toggles
    pub raytracing: bool,
    pub dlss: bool,
    pub taa: bool,
    pub fxaa: bool,
    pub bloom: bool,
    pub atmosphere: bool,
    pub volumetric_fog: bool,
    pub distance_fog: bool,
    pub ssao: bool,
    pub shadows: bool,
    pub motion_blur: bool,

    // FPS cap (0=Unlimited 1=30 2=60 3=120 4=144 5=240)
    pub fps_cap: u8,

    // Accessibility: swap the Right side from red to orange so the two sides
    // stay distinguishable under deuteranopia/protanopia.
    pub colorblind: bool,

    // Sub-parameters (only meaningful when their parent is on; persist regardless)
    pub exposure: u8,        // 0=Low 1=Default 2=High (EV100 11 / 13 / 15)
    pub bloom_intensity: u8, // 0=Low 1=Default 2=High
    pub dlss_quality: u8,    // 0=Performance 1=Balanced 2=Quality 3=DLAA 4=Auto
    pub ssao_quality: u8,    // 0=Low 1=Medium 2=High 3=Ultra
    pub fog_density: u8,     // 0=Low 1=Default 2=High
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fullscreen: true,
            vsync: true,
            hdr: true,
            msaa: 0,
            tonemapping: 0,
            raytracing: false,
            dlss: false,
            taa: false,
            fxaa: false,
            bloom: true,
            atmosphere: true,
            volumetric_fog: true,
            distance_fog: true,
            ssao: false,
            shadows: true,
            motion_blur: false,
            fps_cap: 0,
            colorblind: false,
            exposure: 1,
            bloom_intensity: 1,
            dlss_quality: 2,
            ssao_quality: 2,
            fog_density: 1,
        }
    }
}

/// Which page of the settings menu is active. The selector lives at the top
/// of the overlay and is switched with the shoulder buttons.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    Video,
    Graphics,
}

impl SettingsTab {
    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Video => "Video",
            SettingsTab::Graphics => "Graphics",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            SettingsTab::Video => SettingsTab::Graphics,
            SettingsTab::Graphics => SettingsTab::Video,
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub enum SettingsOrigin {
    #[default]
    Menu,
    Paused,
}

impl SettingsOrigin {
    pub fn to_state(self) -> GameState {
        match self {
            SettingsOrigin::Menu => GameState::Menu,
            SettingsOrigin::Paused => GameState::Paused,
        }
    }
}

#[derive(Resource)]
pub struct Gold {
    pools: [u32; 4],
}

impl Default for Gold {
    fn default() -> Self {
        Self {
            pools: [STARTING_GOLD; 4],
        }
    }
}

impl Gold {
    pub fn get(&self, slot: PlayerSlot) -> u32 {
        self.pools[slot.index()]
    }

    pub fn add(&mut self, slot: PlayerSlot, amount: u32) {
        let p = &mut self.pools[slot.index()];
        *p = p.saturating_add(amount);
    }

    pub fn try_spend(&mut self, slot: PlayerSlot, amount: u32) -> bool {
        let p = &mut self.pools[slot.index()];
        if *p >= amount {
            *p -= amount;
            true
        } else {
            false
        }
    }
}

#[derive(Resource, Default)]
pub struct MatLibrary {
    // Side colors
    pub left: Handle<StandardMaterial>,
    pub right: Handle<StandardMaterial>,
    pub left_dark: Handle<StandardMaterial>,
    pub right_dark: Handle<StandardMaterial>,
    // Misc materials
    pub ground: Handle<StandardMaterial>,
    /// Base-color texture of the ground; regenerated per `GameMode` at match start.
    pub ground_tex: Handle<Image>,
    pub wood_mat: Handle<StandardMaterial>,
    pub metal_mat: Handle<StandardMaterial>,
    // Arrow meshes (the archer's projectile is still procedural).
    pub arrow_shaft: Handle<Mesh>,
    pub arrow_tip: Handle<Mesh>,
    pub arrow_fletch: Handle<Mesh>,
    // Torches (still procedural — lit at night on bases/towers).
    pub flame_mat: Handle<StandardMaterial>,
    pub flame_mesh: Handle<Mesh>,
    pub torch_pole_mesh: Handle<Mesh>,
    // Tower ghost (placement preview)
    pub tower_ghost_mesh: Handle<Mesh>,
    pub ghost_valid_mat: Handle<StandardMaterial>,
    pub ghost_invalid_mat: Handle<StandardMaterial>,
}

/// Handles for the glTF environment scenes (buildings + desert props), loaded
/// once at startup by `load_env_assets`. Each Meshy export is normalized to a
/// ~1.9-unit box centered at the origin, so a per-kind scale + ground-lift
/// (see the `*_SCALE` / `*_LIFT` consts) sizes and seats them on the ground.
#[derive(Resource, Default)]
pub struct EnvAssets {
    pub base: Handle<Scene>,
    pub tower: Handle<Scene>,
    pub mountain: Handle<Scene>,
    pub cactus: [Handle<Scene>; 2],
    pub dead_tree: [Handle<Scene>; 2],
    pub ruins: [Handle<Scene>; 2],
    pub skull: [Handle<Scene>; 2],
    pub stone: [Handle<Scene>; 2],
    pub stone_arch: [Handle<Scene>; 2],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_forward_is_opposite() {
        assert_eq!(Side::Left.forward(), 1.0);
        assert_eq!(Side::Right.forward(), -1.0);
    }

    #[test]
    fn player_slot_side_groups() {
        assert_eq!(PlayerSlot::LeftBottom.side(), Side::Left);
        assert_eq!(PlayerSlot::LeftTop.side(), Side::Left);
        assert_eq!(PlayerSlot::RightBottom.side(), Side::Right);
        assert_eq!(PlayerSlot::RightTop.side(), Side::Right);
    }

    #[test]
    fn base_z_centred_in_1v1_and_offset_in_2v2() {
        assert_eq!(PlayerSlot::LeftBottom.base_z(GameMode::OneVsOne), 0.0);
        assert_eq!(PlayerSlot::RightBottom.base_z(GameMode::OneVsOne), 0.0);
        assert_eq!(
            PlayerSlot::LeftTop.base_z(GameMode::TwoVsTwo),
            -BASE_Z_OFFSET
        );
        assert_eq!(
            PlayerSlot::LeftBottom.base_z(GameMode::TwoVsTwo),
            BASE_Z_OFFSET
        );
    }

    #[test]
    fn tower_z_limit_differs_by_mode() {
        assert!(GameMode::TwoVsTwo.tower_z_limit() > GameMode::OneVsOne.tower_z_limit());
    }

    #[test]
    fn spawn_lane_width_tighter_in_2v2() {
        assert!(
            GameMode::OneVsOne.spawn_lane_half_width() > GameMode::TwoVsTwo.spawn_lane_half_width()
        );
    }

    #[test]
    fn spawn_spread_tighter_than_battlefront() {
        // Units exit clustered, not fanned across the whole field. Const
        // blocks: pure constant relations, checked at compile time (and they
        // keep clippy's assertions_on_constants happy under -D warnings).
        const {
            assert!(SPAWN_LANE_HALF_WIDTH_1V1 < LANE_HALF_WIDTH_1V1);
            assert!(SPAWN_LANE_HALF_WIDTH_2V2 < LANE_HALF_WIDTH_2V2);
        }
    }

    #[test]
    fn health_starts_full() {
        let h = Health::new(42);
        assert_eq!(h.current, 42);
        assert_eq!(h.max, 42);
    }

    #[test]
    fn gold_pools_are_per_slot() {
        let mut g = Gold::default();
        g.add(PlayerSlot::LeftBottom, 5);
        assert_eq!(g.get(PlayerSlot::LeftBottom), STARTING_GOLD + 5);
        assert_eq!(g.get(PlayerSlot::RightBottom), STARTING_GOLD);
        assert!(g.try_spend(PlayerSlot::LeftBottom, STARTING_GOLD + 5));
        assert!(!g.try_spend(PlayerSlot::LeftBottom, 1));
    }
}
