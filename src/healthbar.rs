use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::common::*;

const BAR_WIDTH: f32 = 0.9;
const BAR_HEIGHT: f32 = 0.10;
const BAR_DEPTH: f32 = 0.02;
const BAR_BG_COLOR: Color = Color::srgba(0.05, 0.05, 0.07, 0.9);
const BAR_FILL_FULL: Color = Color::srgb(0.30, 0.85, 0.30);
const BAR_FILL_LOW: Color = Color::srgb(0.95, 0.30, 0.25);
const TOWER_BAR_WIDTH: f32 = 1.2;

/// Vertical offset above each owner kind. Tuned to clear the tallest mesh
/// without floating too far overhead.
const UNIT_BAR_HEIGHT: f32 = 1.25;
const TOWER_BAR_HEIGHT: f32 = 3.30;
const BASE_BAR_HEIGHT: f32 = 3.20;

/// Number of pre-built fill tints along the green→red ramp. Bars swap between
/// these shared material handles as HP drops, instead of mutating a per-bar
/// material asset every frame (which forced a GPU re-upload per bar per frame).
const FILL_STEPS: usize = 9;

/// Width variants of the shared bar meshes (`HealthBarAssets` indices).
const VARIANT_UNIT: usize = 0;
const VARIANT_STRUCT: usize = 1;

/// Shared health-bar meshes/materials, built once at startup by
/// [`init_health_bar_assets`]. Every spawned bar clones these handles, so
/// buying a unit allocates no new assets.
#[derive(Resource, Default)]
pub struct HealthBarAssets {
    /// Background / fill meshes, indexed by `VARIANT_UNIT` / `VARIANT_STRUCT`.
    bg_mesh: [Handle<Mesh>; 2],
    fill_mesh: [Handle<Mesh>; 2],
    bg_mat: Handle<StandardMaterial>,
    /// Green→red tint ramp; index `FILL_STEPS - 1` = full HP.
    fill_mats: [Handle<StandardMaterial>; FILL_STEPS],
}

/// Linear green→red fill color for `step` in `0..FILL_STEPS` (high = healthy).
fn fill_color(step: usize) -> Color {
    let frac = step as f32 / (FILL_STEPS - 1) as f32;
    let full = BAR_FILL_FULL.to_srgba();
    let low = BAR_FILL_LOW.to_srgba();
    Color::srgb(
        low.red * (1.0 - frac) + full.red * frac,
        low.green * (1.0 - frac) + full.green * frac,
        low.blue * (1.0 - frac) + full.blue * frac,
    )
}

pub fn init_health_bar_assets(
    mut assets: ResMut<HealthBarAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (variant, width) in [(VARIANT_UNIT, BAR_WIDTH), (VARIANT_STRUCT, TOWER_BAR_WIDTH)] {
        assets.bg_mesh[variant] = meshes.add(Cuboid::new(width, BAR_HEIGHT, BAR_DEPTH));
        // Slightly thinner fill so the dark frame stays visible on edges.
        assets.fill_mesh[variant] =
            meshes.add(Cuboid::new(width * 0.96, BAR_HEIGHT * 0.7, BAR_DEPTH * 1.2));
    }
    assets.bg_mat = materials.add(StandardMaterial {
        base_color: BAR_BG_COLOR,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    for step in 0..FILL_STEPS {
        assets.fill_mats[step] = materials.add(StandardMaterial {
            base_color: fill_color(step),
            unlit: true,
            ..default()
        });
    }
}

/// Component on the health bar root. Drives placement, billboarding and
/// visibility each frame; despawned automatically when its owner disappears.
#[derive(Component)]
pub struct HealthBar {
    pub owner: Entity,
    pub height: f32,
    pub width: f32,
    /// Buildings (towers, bases) keep the bar visible at full HP; mobile
    /// units only show it once they've taken damage.
    pub always_visible: bool,
    /// Child entity holding the colored fill cuboid. Its X scale is set to
    /// the current HP fraction whenever the fraction changes.
    pub fill: Entity,
    /// HP fraction last applied to the fill (scale + tint), so unchanged HP
    /// costs nothing per frame. Starts negative to force the first update.
    pub last_frac: f32,
}

pub fn spawn_health_bar_for_unit(commands: &mut Commands, owner: Entity) {
    spawn_bar(
        commands,
        owner,
        UNIT_BAR_HEIGHT,
        VARIANT_UNIT,
        BAR_WIDTH,
        false,
    );
}

pub fn spawn_health_bar_for_tower(commands: &mut Commands, owner: Entity) {
    spawn_bar(
        commands,
        owner,
        TOWER_BAR_HEIGHT,
        VARIANT_STRUCT,
        TOWER_BAR_WIDTH,
        true,
    );
}

pub fn spawn_health_bar_for_base(commands: &mut Commands, owner: Entity) {
    spawn_bar(
        commands,
        owner,
        BASE_BAR_HEIGHT,
        VARIANT_STRUCT,
        TOWER_BAR_WIDTH,
        true,
    );
}

fn spawn_bar(
    commands: &mut Commands,
    owner: Entity,
    height: f32,
    variant: usize,
    width: f32,
    always: bool,
) {
    // Queued so the shared handles can be read from `HealthBarAssets` (the
    // spawn_* helpers only carry `&mut Commands`).
    commands.queue(move |world: &mut World| {
        let assets = world.resource::<HealthBarAssets>();
        let bg_mesh = assets.bg_mesh[variant].clone();
        let fill_mesh = assets.fill_mesh[variant].clone();
        let bg_mat = assets.bg_mat.clone();
        // Spawn at the full-HP tint; update_health_bars retunes on HP change.
        let fill_mat = assets.fill_mats[FILL_STEPS - 1].clone();

        let fill = world
            .spawn((
                Mesh3d(fill_mesh),
                MeshMaterial3d(fill_mat),
                Transform::default(),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();

        world
            .spawn((
                Transform::from_xyz(0.0, height, 0.0),
                Visibility::Hidden,
                Mesh3d(bg_mesh),
                MeshMaterial3d(bg_mat),
                NotShadowCaster,
                NotShadowReceiver,
                HealthBar {
                    owner,
                    height,
                    width,
                    always_visible: always,
                    fill,
                    last_frac: -1.0,
                },
            ))
            .add_child(fill);
    });
}

/// Updates every health bar each frame: position above its owner and billboard
/// toward the camera (Y axis only) always; fill scale + green→red tint and
/// visibility only when the HP fraction actually changed (the tint is a swap
/// between the shared `HealthBarAssets` ramp materials, never an asset
/// mutation). Orphan bars (owner despawned) are cleaned up.
pub fn update_health_bars(
    mut commands: Commands,
    assets: Res<HealthBarAssets>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    healths: Query<(&GlobalTransform, &Health)>,
    mut bars: Query<(Entity, &mut HealthBar, &mut Transform, &mut Visibility)>,
    mut fills: Query<(&mut Transform, &mut MeshMaterial3d<StandardMaterial>), Without<HealthBar>>,
) {
    let Some(cam_t) = cameras.iter().next() else {
        return;
    };
    let cam_pos = cam_t.translation();
    for (entity, mut bar, mut transform, mut vis) in bars.iter_mut() {
        let Ok((owner_t, hp)) = healths.get(bar.owner) else {
            // Owner gone — drop the bar AND its `fill` child. Bevy 0.18
            // `despawn()` is recursive over the relationship graph, so the
            // fill (added via `add_child` in `spawn_bar`) goes with it.
            commands.entity(entity).despawn();
            continue;
        };
        let owner_pos = owner_t.translation();
        transform.translation = owner_pos + Vec3::new(0.0, bar.height, 0.0);

        // Y-only billboard: face the camera horizontally.
        let to_cam = cam_pos - transform.translation;
        let yaw = to_cam.x.atan2(to_cam.z);
        transform.rotation = Quat::from_rotation_y(yaw);

        let max = hp.max.max(1) as f32;
        let cur = hp.current.max(0) as f32;
        let frac = (cur / max).clamp(0.0, 1.0);

        // Everything below only matters when the HP fraction moved.
        if (frac - bar.last_frac).abs() < f32::EPSILON {
            continue;
        }
        bar.last_frac = frac;

        let new_vis = if bar.always_visible || frac < 0.999 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != new_vis {
            *vis = new_vis;
        }

        if let Ok((mut t, mut mat)) = fills.get_mut(bar.fill) {
            let inner_w = bar.width * 0.96;
            let scaled = inner_w * frac;
            t.scale = Vec3::new(frac.max(0.0001), 1.0, 1.0);
            // Anchor the fill at the left edge so it shrinks from the right.
            t.translation = Vec3::new(-(inner_w - scaled) * 0.5, 0.0, BAR_DEPTH * 0.6);

            // Re-tint by swapping to the nearest ramp material.
            let step = (frac * (FILL_STEPS - 1) as f32).round() as usize;
            let target = &assets.fill_mats[step.min(FILL_STEPS - 1)];
            if mat.0 != *target {
                mat.0 = target.clone();
            }
        }
    }
}
