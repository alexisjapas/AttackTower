//! Battlefield & structures tunables: base/tower/arrow stats, arena geometry
//! (base positions, neutral zone, lanes battlefront, tower placement zone),
//! structure colliders and the gamepad cursor constants.

use super::units::{LANE_HALF_WIDTH_1V1, LANE_HALF_WIDTH_2V2};

pub const BASE_HP: i32 = 40;

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

pub const TOWER_DEATH_DURATION: f32 = 0.45;
