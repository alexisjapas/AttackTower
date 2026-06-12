//! Unit tunables: per-kind stats and glTF model paths, combat AI ranges,
//! battalion formation, spawning geometry, physics capsule and animation
//! timing. Values are re-exported through `common` (`use crate::common::*`).

// Soldier
pub const SOLDIER_HP: i32 = 10;
pub const SOLDIER_DAMAGE: i32 = 3;
pub const SOLDIER_COST: u32 = 1;
pub const SOLDIER_SPEED: f32 = 1.8;
pub const SOLDIER_COOLDOWN: f32 = 1.0;
/// Fraction through the `Left_Slash` clip at which the blade connects and the
/// damage lands (just past the windup) — instead of at the start of the clip.
pub const SOLDIER_HIT_FRACTION: f32 = 0.35;

// Miner
pub const MINER_HP: i32 = 8;
pub const MINER_COST: u32 = 4;
pub const MINER_SPEED: f32 = 1.4;
pub const MINER_COOLDOWN: f32 = 1.1;
pub const MINER_GOLD_PER_HIT: u32 = 1;
pub const MAX_MINERS_PER_PLAYER: usize = 5;
/// Gold a miner deposits per round-trip: it stacks this much across several
/// swings (`MINER_GOLD_PER_HIT` each) before walking it back to the base, so
/// most of its time is spent mining rather than commuting.
pub const MINER_CAPACITY: u32 = 5;
pub const MINER_RING_RADIUS: f32 = 1.6;
pub const MINER_DEPOSIT_RANGE: f32 = 1.4;
/// How close (XZ) a velocity-driven miner must get to its mining slot before it
/// stops and starts swinging. Replaces the old exact transform snap-to-slot.
pub const MINER_ARRIVE_RANGE: f32 = 0.25;
/// Fraction through the `Heavy_Hammer_Swing` clip at which the pick bites the
/// rock and the ore is gained — near the end of the swing, so a miner finishes
/// its last swing before turning back to the base instead of leaving mid-air.
pub const MINER_COLLECT_FRACTION: f32 = 0.9;

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
/// Fraction through the `mage_spell_cast` clip at which the heal/armor lands
/// (the spell's visual release), not at the start of the cast.
pub const PRIEST_CAST_FRACTION: f32 = 0.6;
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
/// How fast (rad/s) an aiming kind (`uses_face_yaw`: archer + priest) pivots
/// toward `UnitAnim.face_yaw` (its target, or back toward the advance
/// direction when the target departs).
pub const FACE_TURN_SPEED: f32 = 6.0;
/// Below this facing error (rad) an aiming kind is considered aimed: it stops
/// turning and may shoot / cast / walk.
pub const FACE_TURN_EPS: f32 = 0.06;
/// How long (s) any unit keeps playing its attack animation after its target
/// briefly leaves range, so the pose doesn't flicker to idle between strikes
/// (originally tuned for the archer's volleys).
pub const ATTACK_HOLD: f32 = 0.6;
/// Fraction through the `Archery_Shot` clip at which the arrow leaves the bow.
/// The clip ends with the archer lowering the bow arm, so releasing slightly
/// before the end (rather than at the cycle boundary) reads as the actual loose.
pub const ARCHER_SHOT_RELEASE_FRACTION: f32 = 0.78;
/// Extra lead (real seconds) before the release point, so the arrow leaves a
/// touch earlier than the pose would suggest. Converted into clip-time with the
/// clip's playback speed where it is applied (`animate_unit_model`).
pub const ARCHER_SHOT_RELEASE_LEAD: f32 = 0.1;

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
