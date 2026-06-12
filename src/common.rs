use avian3d::prelude::*;
use bevy::prelude::*;

// All tunable constants live in `config/` (split by domain: units, weapons,
// arena, world) and are re-exported flat here, so `use crate::common::*`
// keeps bringing every constant into scope.
pub use crate::config::*;

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

    /// World X of this side's base line (single source for the four spawners
    /// that used to re-match on `Side`).
    pub fn base_x(self) -> f32 {
        match self {
            Side::Left => LEFT_BASE_X,
            Side::Right => RIGHT_BASE_X,
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

/// Gameplay stats of one unit kind — the single source of truth shared by
/// `spawn_unit` (combat components), the HUD panel (button labels + stats
/// card) and the buy actions (cost). Add a field here rather than matching on
/// `UnitKind` at a call site.
#[derive(Clone, Copy)]
pub struct UnitStats {
    pub hp: i32,
    /// Per-hit damage. 0 for the non-fighting kinds (miner, priest).
    pub damage: i32,
    pub cost: u32,
    pub speed: f32,
    pub cooldown: f32,
    /// Signed X offset from the base along the side's forward axis at spawn —
    /// negative for the miner, which exits BEHIND the base (rock side).
    pub spawn_offset: f32,
}

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

    pub fn label(self) -> &'static str {
        match self {
            UnitKind::Soldier => "Soldier",
            UnitKind::Miner => "Miner",
            UnitKind::Archer => "Archer",
            UnitKind::Priest => "Priest",
        }
    }

    pub const fn stats(self) -> UnitStats {
        match self {
            UnitKind::Soldier => UnitStats {
                hp: SOLDIER_HP,
                damage: SOLDIER_DAMAGE,
                cost: SOLDIER_COST,
                speed: SOLDIER_SPEED,
                cooldown: SOLDIER_COOLDOWN,
                spawn_offset: SOLDIER_SPAWN_OFFSET,
            },
            UnitKind::Miner => UnitStats {
                hp: MINER_HP,
                damage: 0,
                cost: MINER_COST,
                speed: MINER_SPEED,
                cooldown: MINER_COOLDOWN,
                spawn_offset: -MINER_SPAWN_OFFSET,
            },
            UnitKind::Archer => UnitStats {
                hp: ARCHER_HP,
                damage: ARCHER_DAMAGE,
                cost: ARCHER_COST,
                speed: ARCHER_SPEED,
                cooldown: ARCHER_COOLDOWN,
                spawn_offset: ARCHER_SPAWN_OFFSET,
            },
            UnitKind::Priest => UnitStats {
                hp: PRIEST_HP,
                damage: 0,
                cost: PRIEST_COST,
                speed: PRIEST_SPEED,
                cooldown: PRIEST_COOLDOWN,
                spawn_offset: PRIEST_SPAWN_OFFSET,
            },
        }
    }
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

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct TorchLight;

#[derive(Component)]
pub struct TorchFlame;

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
    /// losses (see `ATTACK_HOLD`).
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

/// Tiny deterministic xorshift RNG — the project's single random source (no
/// `rand` dependency). Seeded per call site: the scenery/mountain scatter uses
/// fixed seeds for a reproducible layout; `units::rand_jitter` feeds it from an
/// atomic counter for cheap spawn jitter.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / (u32::MAX as f32 + 1.0)
    }
    /// Uniform in [a, b).
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }
}

/// Top-level app state, driven by Bevy `States` (`init_state` in `GamePlugin`).
/// Transitions are requested through `NextState<GameState>` and applied by the
/// `StateTransition` schedule between frames, so the input system of the new
/// state never sees the button press that caused the transition (this replaced
/// the old per-system `!state.is_changed()` guards). The winning side of a
/// finished match lives in the separate [`Winner`] resource, set by
/// `check_winner` right before entering `Ended`.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Settings,
    SideSelect,
    Playing,
    Paused,
    Ended,
}

/// Computed state active while a match exists on the battlefield: `Playing` or
/// `Paused` — NOT `Ended`, where the battlefield lingers behind the endgame
/// overlay until the player returns to the menu. `OnEnter` builds the arena,
/// HUD and player focus; `OnExit` tears down focus and tower placement.
///
/// CAREFUL: `Paused → Settings → Paused` leaves and re-enters this state, so
/// every `OnEnter(InMatch)` system must tolerate an already-built match
/// (`spawn_arena`, `spawn_initial_miners` and `grant_player_focus` keep
/// existence guards for exactly this).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InMatch;

impl ComputedStates for InMatch {
    type SourceStates = GameState;

    fn compute(source: GameState) -> Option<InMatch> {
        matches!(source, GameState::Playing | GameState::Paused).then_some(InMatch)
    }
}

/// Side that won the last finished match (`None` while no match has ended).
/// Set by `check_winner` before the `Ended` transition, read by the endgame
/// overlay, cleared by `reset_match`.
#[derive(Resource, Default, Clone, Copy)]
pub struct Winner(pub Option<Side>);

/// Despawn every entity carrying `T`. Shared `OnExit` teardown for the state
/// overlays (despawn is recursive over Bevy 0.18 relationships, so children go
/// with the root).
pub fn despawn_all<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// Top-level phases of the `Update` schedule, chained in `main.rs`. Every
/// plugin hangs its systems on one of these so cross-module ordering lives in
/// a single place.
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum AppSet {
    Input,
    World,
    React,
    Visual,
    FrameLimit,
}

/// Phases of the per-frame gameplay tick inside [`AppSet::World`], chained in
/// `main.rs` (damage → death state → animation → despawn must propagate within
/// one frame). Lets the units and towers plugins join the same chain without
/// referencing each other's systems.
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum CombatSet {
    Attack,
    ApplyDamage,
    Animate,
    Cleanup,
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
        // Units exit clustered, not fanned across the whole field.
        assert!(SPAWN_LANE_HALF_WIDTH_1V1 < LANE_HALF_WIDTH_1V1);
        assert!(SPAWN_LANE_HALF_WIDTH_2V2 < LANE_HALF_WIDTH_2V2);
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
