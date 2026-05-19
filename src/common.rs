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

pub const STARTING_GOLD: u32 = 10;
pub const ENGAGE_RANGE: f32 = 1.4;
pub const MINE_RANGE: f32 = 1.4;

pub const LEFT_BASE_X: f32 = -8.0;
pub const RIGHT_BASE_X: f32 = 8.0;

pub const UNIT_RADIUS: f32 = 0.35;
pub const SOLDIER_SPAWN_OFFSET: f32 = 1.5;
pub const MINER_SPAWN_OFFSET: f32 = 1.0;
pub const ROCK_OFFSET: f32 = 3.0;
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
}

#[derive(Component)]
pub struct Base;

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Rock;

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
    Playing,
    Ended(Side),
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
}
