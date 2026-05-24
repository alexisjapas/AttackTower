use bevy::prelude::*;

pub const BASE_HP: i32 = 20;

// Soldier
pub const SOLDIER_HP: i32 = 10;
pub const SOLDIER_DAMAGE: i32 = 3;
pub const SOLDIER_COST: u32 = 1;
pub const SOLDIER_SPEED: f32 = 1.8;
pub const SOLDIER_COOLDOWN: f32 = 1.0;

// Miner
pub const MINER_HP: i32 = 8;
pub const MINER_COST: u32 = 5;
pub const MINER_SPEED: f32 = 1.4;
pub const MINER_COOLDOWN: f32 = 1.1;
pub const MINER_GOLD_PER_HIT: u32 = 1;
pub const MAX_MINERS_PER_SIDE: usize = 5;
pub const MINER_CAPACITY: u32 = 3;
pub const MINER_RING_RADIUS: f32 = 1.6;
pub const MINER_DEPOSIT_RANGE: f32 = 1.4;

// Archer
pub const ARCHER_HP: i32 = 7;
pub const ARCHER_DAMAGE: i32 = 2;
pub const ARCHER_COST: u32 = 3;
pub const ARCHER_SPEED: f32 = 1.5;
pub const ARCHER_COOLDOWN: f32 = 1.7;
pub const ARCHER_RANGE: f32 = 6.5;
pub const ARCHER_SPAWN_OFFSET: f32 = 1.5;

// Tower
pub const TOWER_HP: i32 = 30;
pub const TOWER_DAMAGE: i32 = 3;
pub const TOWER_COST: u32 = 8;
pub const TOWER_RANGE: f32 = 7.5;
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
pub const MINE_RANGE: f32 = 1.4;

pub const LEFT_BASE_X: f32 = -14.0;
pub const RIGHT_BASE_X: f32 = 14.0;
// Terrain between bases is split into three equal parts: left zone, neutral, right zone.
pub const ZONE_BOUNDARY: f32 = (RIGHT_BASE_X - LEFT_BASE_X) / 6.0;
pub const TOWER_PLACEMENT_MARGIN: f32 = 1.6;
pub const TOWER_PLACEMENT_Z_LIMIT: f32 = 4.0;

pub const GAMEPAD_STICK_DEADZONE: f32 = 0.25;
pub const GAMEPAD_CURSOR_SPEED: f32 = 6.0;
pub const PLAYER_PANEL_SLOTS: usize = 4;

pub const UNIT_RADIUS: f32 = 0.35;
pub const SOLDIER_SPAWN_OFFSET: f32 = 1.5;
pub const LANE_COUNT: usize = 5;
pub const LANE_HALF_WIDTH: f32 = 2.6;
pub const MINER_SPAWN_OFFSET: f32 = 1.0;
pub const ROCK_OFFSET: f32 = 5.5;
pub const SPAWN_Z_JITTER: f32 = 0.6;

pub const BOB_BASE_Y: f32 = 0.55;
pub const HIP_Y: f32 = 0.40;
pub const LEG_PIVOT_OFFSET: f32 = 0.18;
pub const LEG_SPREAD_Z: f32 = 0.13;
pub const ARM_PIVOT_OFFSET: f32 = 0.18;
pub const ARM_SPREAD_Z: f32 = 0.27;
pub const ARM_SHOULDER_Y: f32 = 0.10;

pub const WALK_FREQUENCY: f32 = 10.0;
pub const LEG_SWING: f32 = 0.55;
pub const ARM_SWING: f32 = 0.40;
pub const BOB_AMPLITUDE: f32 = 0.05;
pub const ATTACK_SWING_AMPLITUDE: f32 = 1.2;
pub const HURT_DURATION: f32 = 0.18;
pub const HURT_TILT: f32 = 0.28;
pub const DEATH_DURATION: f32 = 0.6;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    pub fn forward(self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Side::Left => Color::srgb(0.25, 0.55, 1.0),
            Side::Right => Color::srgb(1.0, 0.40, 0.35),
        }
    }

    pub fn color_dark(self) -> Color {
        match self {
            Side::Left => Color::srgb(0.14, 0.32, 0.70),
            Side::Right => Color::srgb(0.70, 0.24, 0.20),
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

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Rock;

#[derive(Component)]
pub struct Tower;

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct TorchLight;

#[derive(Component)]
pub struct TorchFlame;

pub const TORCH_INTENSITY: f32 = 250_000.0;
pub const TORCH_RANGE: f32 = 10.0;
pub const TORCH_COLOR: Color = Color::srgb(1.0, 0.65, 0.30);

pub const SUN_DAY_PERIOD: f32 = 240.0;
pub const SUN_DISTANCE: f32 = 55.0;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    #[default]
    Day,
    Night,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct DlssAvailable(pub bool);

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
    pub left: Option<PlacementSeat>,
    pub right: Option<PlacementSeat>,
}

impl PlacementMode {
    pub fn get(&self, side: Side) -> Option<PlacementSeat> {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }

    pub fn get_mut(&mut self, side: Side) -> &mut Option<PlacementSeat> {
        match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    pub fn set(&mut self, side: Side, seat: PlacementSeat) {
        *self.get_mut(side) = Some(seat);
    }

    pub fn clear(&mut self, side: Side) {
        *self.get_mut(side) = None;
    }
}

#[derive(Resource, Default)]
pub struct PlayerControllers {
    pub left: Option<Entity>,
    pub right: Option<Entity>,
}

impl PlayerControllers {
    pub fn get(&self, side: Side) -> Option<Entity> {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }
}

#[derive(Resource, Default)]
pub struct MenuFocus {
    pub index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct SeatSelection {
    pub hovered: Side,
    pub confirmed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct PlayerFocus {
    pub side: Side,
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
}

#[derive(Component)]
pub struct UnitRig {
    pub bob: Entity,
    pub leg_left: Entity,
    pub leg_right: Entity,
    pub arm_left: Entity,
    pub arm_right: Entity,
}

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
    pub fullscreen: bool,
    pub vsync: bool,
    pub hdr: bool,
    pub raytracing: bool,
    pub dlss: bool,
    pub taa: bool,
    pub bloom: bool,
    pub atmosphere: bool,
    pub volumetric_fog: bool,
    pub distance_fog: bool,
    pub tonemapping: u8, // 0=AcesFitted 1=TonyMcMapface 2=Reinhard 3=None
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fullscreen: true,
            vsync: true,
            hdr: true,
            raytracing: false,
            dlss: false,
            taa: false,
            bloom: true,
            atmosphere: true,
            volumetric_fog: true,
            distance_fog: true,
            tonemapping: 0,
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
    pub left: u32,
    pub right: u32,
}

impl Default for Gold {
    fn default() -> Self {
        Self {
            left: STARTING_GOLD,
            right: STARTING_GOLD,
        }
    }
}

impl Gold {
    pub fn get(&self, side: Side) -> u32 {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }

    pub fn add(&mut self, side: Side, amount: u32) {
        match side {
            Side::Left => self.left = self.left.saturating_add(amount),
            Side::Right => self.right = self.right.saturating_add(amount),
        }
    }

    pub fn try_spend(&mut self, side: Side, amount: u32) -> bool {
        if self.get(side) >= amount {
            match side {
                Side::Left => self.left -= amount,
                Side::Right => self.right -= amount,
            }
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
    pub stone_light: Handle<StandardMaterial>,
    pub stone_dark: Handle<StandardMaterial>,
    pub rock_mat: Handle<StandardMaterial>,
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
    pub bow_limb: Handle<Mesh>,
    pub bow_string: Handle<Mesh>,
    pub arrow_shaft: Handle<Mesh>,
    pub arrow_tip: Handle<Mesh>,
    pub arrow_fletch: Handle<Mesh>,
    // Scenery
    pub grass_blade: Handle<Mesh>,
    pub bush_mesh: Handle<Mesh>,
    pub plant_stem: Handle<Mesh>,
    pub plant_flower: Handle<Mesh>,
    pub grass_mat: Handle<StandardMaterial>,
    pub bush_mat: Handle<StandardMaterial>,
    pub flower_red_mat: Handle<StandardMaterial>,
    pub flower_yellow_mat: Handle<StandardMaterial>,
    pub flower_violet_mat: Handle<StandardMaterial>,
    pub flame_mat: Handle<StandardMaterial>,
    pub flame_mesh: Handle<Mesh>,
    pub torch_pole_mesh: Handle<Mesh>,
    // Tower meshes
    pub tower_foundation: Handle<Mesh>,
    pub tower_shaft: Handle<Mesh>,
    pub tower_top_slab: Handle<Mesh>,
    pub tower_crenel: Handle<Mesh>,
    pub tower_roof: Handle<Mesh>,
    // Tower ghost (placement preview)
    pub tower_ghost_mesh: Handle<Mesh>,
    pub ghost_valid_mat: Handle<StandardMaterial>,
    pub ghost_invalid_mat: Handle<StandardMaterial>,
    // Zone boundary marker
    pub zone_marker_mesh: Handle<Mesh>,
    pub zone_marker_mat: Handle<StandardMaterial>,
}
