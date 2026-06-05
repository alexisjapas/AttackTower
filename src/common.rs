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
pub const MINER_CAPACITY: u32 = 4;
pub const MINER_RING_RADIUS: f32 = 1.6;
pub const MINER_DEPOSIT_RANGE: f32 = 1.4;

// Archer
pub const ARCHER_HP: i32 = 7;
pub const ARCHER_DAMAGE: i32 = 2;
pub const ARCHER_COST: u32 = 3;
pub const ARCHER_SPEED: f32 = 1.5;
pub const ARCHER_COOLDOWN: f32 = 1.7;
pub const ARCHER_RANGE: f32 = 8.0;
pub const ARCHER_SPAWN_OFFSET: f32 = 1.5;
/// If the closest enemy is within this distance, the archer steps backward
/// while continuing to shoot (kiting). Cheap, gives the archer a tactical
/// identity vs. soldiers.
pub const ARCHER_KITE_RANGE: f32 = 2.5;
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
pub const ARCHER_HURT_PATHS: [&str; 2] = [
    "models/adamar/characters/adamar_archer_biped_Animation_Face_Punch_Reaction_withSkin.glb",
    "models/adamar/characters/adamar_archer_biped_Animation_Slap_Reaction_withSkin.glb",
];
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
/// clip's playback speed where it is applied (`animate_archer`).
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

// ─────────────────────────────────────────────────────────────────────────────
// Environment glTF assets (buildings + desert props). Every Meshy export is
// normalized to a ~1.9-unit box centered at the origin, so each kind gets a
// SCALE (multiplier on that ~1.9-unit model) and a LIFT (Y offset ≈ SCALE×|min_y|
// that seats the model's base on the ground). Tune visually if anything floats
// or sinks. Loaded into `EnvAssets` by `setup::load_env_assets`.
// ─────────────────────────────────────────────────────────────────────────────
pub const BASE_MODEL_PATH: &str = "models/adamar/buildings/adamar_base.glb";
pub const TOWER_MODEL_PATH: &str = "models/adamar/buildings/adamar_tower.glb";
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
pub const SCENERY_CLEAR_Z_MIN: f32 = -7.0;
pub const SCENERY_CLEAR_Z_MAX: f32 = 9.0;
/// Per-instance random scale spread applied on top of each kind's base scale.
pub const SCENERY_SCALE_MIN: f32 = 0.8;
pub const SCENERY_SCALE_MAX: f32 = 1.25;

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

pub const STARTING_GOLD: u32 = 10;
pub const ENGAGE_RANGE: f32 = 1.4;

pub const LEFT_BASE_X: f32 = -14.0;
pub const RIGHT_BASE_X: f32 = 14.0;
// Z offset between the two bases of a same side in 2v2 mode.
pub const BASE_Z_OFFSET: f32 = 3.0;
// Terrain between bases is split into three equal parts: left zone, neutral, right zone.
pub const ZONE_BOUNDARY: f32 = (RIGHT_BASE_X - LEFT_BASE_X) / 6.0;
pub const TOWER_PLACEMENT_MARGIN: f32 = 1.6;
/// Z half-extent of the placement zone in 1v1: lanes are centred on Z=0 so the
/// usable strip is narrow. Wider would let you drop a tower way off any lane.
pub const TOWER_PLACEMENT_Z_LIMIT_1V1: f32 = LANE_HALF_WIDTH_1V1 + 0.4;
/// Z half-extent in 2v2: each side has two bases spread on Z, with lanes
/// reaching `BASE_Z_OFFSET + LANE_HALF_WIDTH_2V2`. The limit covers both.
pub const TOWER_PLACEMENT_Z_LIMIT_2V2: f32 = BASE_Z_OFFSET + LANE_HALF_WIDTH_2V2 + 0.5;

pub const GAMEPAD_STICK_DEADZONE: f32 = 0.25;
pub const GAMEPAD_CURSOR_SPEED: f32 = 6.0;
pub const PLAYER_PANEL_SLOTS: usize = 4;

pub const UNIT_RADIUS: f32 = 0.35;
pub const SOLDIER_SPAWN_OFFSET: f32 = 1.5;
/// Number of parallel lanes (Z offsets) units cycle through on spawn so that
/// successive same-kind units don't pile on top of each other.
pub const LANE_COUNT: usize = 5;
/// Half-width of the lane spread in 1v1: lanes are centred on each base's Z
/// (which is 0 in 1v1) and span ±LANE_HALF_WIDTH_1V1.
pub const LANE_HALF_WIDTH_1V1: f32 = 2.6;
/// Half-width in 2v2: tighter so each ally's lanes don't bleed into the
/// allied lanes (their bases are only `BASE_Z_OFFSET` apart on Z).
pub const LANE_HALF_WIDTH_2V2: f32 = 1.5;
pub const MINER_SPAWN_OFFSET: f32 = 1.0;
pub const ROCK_OFFSET: f32 = 5.5;
/// Spread applied to non-laned units' Z at spawn so consecutive same-side
/// spawns don't appear in a perfect line. ±half the value, around the slot's
/// base Z.
pub const SPAWN_Z_JITTER: f32 = 0.6;

// ─────────────────────────────────────────────────────────────────────────────
// Character rig geometry (units assemble themselves from primitives; these
// constants describe the local-space placement of the limbs/bob node).
//
//   bob (Y = BOB_BASE_Y) ──── body + head + arms
//      └─ limbs pivot at hip/shoulder, mesh hangs LEG_PIVOT_OFFSET below.
//
//   Y axis up; X is the unit's facing direction.
// ─────────────────────────────────────────────────────────────────────────────
/// Local Y of the body/head/arms "bob" node above the unit's root.
pub const BOB_BASE_Y: f32 = 0.55;
/// Local Y of the leg pivot.
pub const HIP_Y: f32 = 0.40;
/// Y distance from the pivot to the mesh centre — lets the limb rotate around
/// its top end instead of its midpoint.
pub const LEG_PIVOT_OFFSET: f32 = 0.18;
pub const LEG_SPREAD_Z: f32 = 0.13;
pub const ARM_PIVOT_OFFSET: f32 = 0.18;
pub const ARM_SPREAD_Z: f32 = 0.27;
pub const ARM_SHOULDER_Y: f32 = 0.10;

/// Walk cycle frequency (rad/s).
pub const WALK_FREQUENCY: f32 = 10.0;
pub const LEG_SWING: f32 = 0.55;
pub const ARM_SWING: f32 = 0.40;
pub const BOB_AMPLITUDE: f32 = 0.05;
pub const ATTACK_SWING_AMPLITUDE: f32 = 1.2;
/// Seconds the "hurt flash" / tilt lasts after a damage event.
pub const HURT_DURATION: f32 = 0.18;
pub const HURT_TILT: f32 = 0.28;
/// Seconds the death animation takes before the unit is despawned.
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

    pub fn lane_half_width(self) -> f32 {
        match self {
            GameMode::OneVsOne => LANE_HALF_WIDTH_1V1,
            GameMode::TwoVsTwo => LANE_HALF_WIDTH_2V2,
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
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Soldier,
    Miner,
    Archer,
}

#[derive(Component)]
pub struct Base;

/// Marker added to a `Base` whose HP hit 0. Targeting filters (`combat_tick`,
/// `tower_attack_tick`) exclude these so allied units retarget the surviving
/// enemy base, and the HUD greys out the owning player's panel.
#[derive(Component)]
pub struct BaseDestroyed;

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
/// The height (Y) is the key knob for the skyline: the horizon always sits at
/// the camera's eye level, so a mountain only has **sky directly behind it** if
/// its top rises above this Y. The tallest peaks top out at ~13.9, so the eye
/// must sit below that — hence 11. At 11 the tall ridge clears the horizon by
/// ~2° (`atan((13.9-11)/80)`), silhouetting it against the sky instead of the
/// camera looking down over it onto the ground beyond.
pub const CAMERA_DEFAULT_POS: Vec3 = Vec3::new(0.0, 13.0, 30.0);
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
    pub target_entity: Entity,
    pub target_pos: Vec3,
    pub elapsed: f32,
    pub total: f32,
    pub apex: f32,
    pub damage: i32,
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
    pub walk_phase: f32,
    pub walk_amp: f32,
    pub attacking: bool,
    pub attack_phase: f32,
    pub hurt_t: f32,
    pub dying: bool,
    pub death_t: f32,
    /// Desired entity yaw (rotation around Y) for the archer, set by
    /// `combat_tick` (target + `ARCHER_SHOT_YAW_OFFSET` when shooting, the
    /// advance direction otherwise). `animate_archer` smoothly rotates to it.
    /// Unused by procedural units.
    pub face_yaw: f32,
    /// Last observed `Health.current`. Lets `process_damage_effects` flash
    /// only when HP actually drops (a heal that leaves current<max should
    /// not look like a hit).
    pub last_hp: Option<i32>,
}

#[derive(Component)]
pub struct UnitRig {
    pub bob: Entity,
    pub leg_left: Entity,
    pub leg_right: Entity,
    pub arm_left: Entity,
    pub arm_right: Entity,
}

/// Marker on the root of an archer rendered from the glTF model. Such units
/// carry no `UnitRig` (so `animate_units` skips them) and are driven instead by
/// `animate_archer` through a descendant `AnimationPlayer`.
#[derive(Component)]
pub struct ArcherModel;

/// Which logical clip an archer is currently playing. Lets `animate_archer`
/// avoid re-issuing `play` every frame.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArcherClip {
    #[default]
    None,
    Idle,
    Walk,
    Attack,
    Hurt,
    Death,
}

/// A shot queued by `combat_tick` and released by `animate_archer` when the
/// shot clip reaches the end of its cycle. The target is snapshotted at queue
/// time; the arrow's light homing (`arrow_flight_system`) corrects for movement
/// during the short draw.
#[derive(Clone, Copy)]
pub struct PendingShot {
    pub target: Entity,
    pub target_pos: Vec3,
    pub damage: i32,
}

/// Per-archer animation bookkeeping: the descendant `AnimationPlayer` entity
/// (instanced asynchronously with the scene) plus the small state machine that
/// `animate_archer` runs off `UnitAnim`.
#[derive(Component, Default)]
pub struct ArcherAnimState {
    pub player: Option<Entity>,
    /// The skeleton's `LeftHand` bone (the bow hand), resolved once by
    /// `bind_archer_bow_hand`. Arrows leave from its world position.
    pub left_hand: Option<Entity>,
    pub current: ArcherClip,
    pub hurt_index: usize,
    pub oneshot_active: bool,
    /// Previous-frame `UnitAnim.hurt_t`, to detect a fresh hit (rising edge).
    pub last_hurt_t: f32,
    /// Countdown that keeps the shot animation playing through brief target
    /// losses (see `ARCHER_ATTACK_HOLD`).
    pub attack_hold: f32,
    /// Shot clip `seek_time()` last frame, so `animate_archer` can fire one
    /// arrow per cycle the moment playback crosses `ARCHER_SHOT_RELEASE_FRACTION`.
    pub last_attack_seek: f32,
    /// Target snapshot to release on the next shot-cycle end; `None` when there
    /// is nothing to shoot at (or the archer isn't yet aimed).
    pub pending_shot: Option<PendingShot>,
}

/// Indices (and precomputed playback speeds) of the archer's clips inside its
/// shared `AnimationGraph`.
#[derive(Clone, Copy)]
pub struct ArcherAnimNodes {
    pub walk: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub hurts: [AnimationNodeIndex; 2],
    pub death: AnimationNodeIndex,
    /// Speed that makes one loop of the shot clip last `ARCHER_COOLDOWN`.
    pub attack_speed: f32,
    /// Clip-local duration (s) of the shot clip, so `animate_archer` can release
    /// the arrow at `ARCHER_SHOT_RELEASE_FRACTION` of the way through.
    pub attack_len: f32,
    /// Speed that makes the fall clip finish within `ARCHER_DEATH_DURATION`.
    pub death_speed: f32,
}

/// Handles for the shared archer model: the scene (mesh + skeleton) plus one
/// `AnimationClip` per action loaded from its own file. `graph`/`nodes` stay
/// `None` until `build_archer_graph` has seen all clips finish loading.
#[derive(Resource, Default)]
pub struct ArcherAssets {
    pub scene: Handle<Scene>,
    pub walk: Handle<AnimationClip>,
    pub attack: Handle<AnimationClip>,
    pub hurts: [Handle<AnimationClip>; 2],
    pub death: Handle<AnimationClip>,
    pub graph: Option<Handle<AnimationGraph>>,
    pub nodes: Option<ArcherAnimNodes>,
    pub bow: Handle<Scene>,
}

/// Marker on the bow scene entity parented to the archer's `LeftHand` bone.
#[derive(Component)]
pub struct ArcherBow;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Menu,
    Settings,
    SideSelect,
    Playing,
    Paused,
    Ended(Side),
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
    pub eye_mat: Handle<StandardMaterial>,
    pub ground: Handle<StandardMaterial>,
    pub wood_mat: Handle<StandardMaterial>,
    pub metal_mat: Handle<StandardMaterial>,
    // Character meshes
    pub body_mesh: Handle<Mesh>,
    pub head_mesh: Handle<Mesh>,
    pub limb_mesh: Handle<Mesh>,
    pub eye_mesh: Handle<Mesh>,
    // Weapons
    pub spear_shaft: Handle<Mesh>,
    pub spear_tip: Handle<Mesh>,
    pub pickaxe_handle: Handle<Mesh>,
    pub pickaxe_head: Handle<Mesh>,
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
    // Zone boundary marker
    pub zone_marker_mesh: Handle<Mesh>,
    pub zone_marker_mat: Handle<StandardMaterial>,
}

/// Handles for the glTF environment scenes (buildings + desert props), loaded
/// once at startup by `load_env_assets`. Each Meshy export is normalized to a
/// ~1.9-unit box centered at the origin, so a per-kind scale + ground-lift
/// (see the `*_SCALE` / `*_LIFT` consts) sizes and seats them on the ground.
#[derive(Resource, Default)]
pub struct EnvAssets {
    pub base: Handle<Scene>,
    pub tower: Handle<Scene>,
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
    fn lane_half_width_tighter_in_2v2() {
        assert!(GameMode::OneVsOne.lane_half_width() > GameMode::TwoVsTwo.lane_half_width());
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
