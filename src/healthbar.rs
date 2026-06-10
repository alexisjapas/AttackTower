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
/// Fill width as a fraction of the bar width, so the dark frame stays visible
/// on the edges. Shared by the mesh construction and the fill-anchoring math.
const FILL_WIDTH_RATIO: f32 = 0.96;

/// Vertical offset above each owner kind. Tuned to clear the tallest mesh
/// without floating too far overhead.
const UNIT_BAR_HEIGHT: f32 = 1.25;
const TOWER_BAR_HEIGHT: f32 = 3.30;
const BASE_BAR_HEIGHT: f32 = 3.20;

/// Shared health-bar assets, created once at startup. Every bar reuses these
/// meshes and the background material; only the fill material is per-bar
/// (it is re-tinted green→red as the owner's HP drops).
#[derive(Resource, Default)]
pub struct HealthBarAssets {
    pub bg_mesh_unit: Handle<Mesh>,
    pub fill_mesh_unit: Handle<Mesh>,
    pub bg_mesh_struct: Handle<Mesh>,
    pub fill_mesh_struct: Handle<Mesh>,
    pub bg_mat: Handle<StandardMaterial>,
}

pub fn init_health_bar_assets(
    mut assets: ResMut<HealthBarAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bg = |w: f32| Cuboid::new(w, BAR_HEIGHT, BAR_DEPTH);
    let fill = |w: f32| Cuboid::new(w * FILL_WIDTH_RATIO, BAR_HEIGHT * 0.7, BAR_DEPTH * 1.2);
    assets.bg_mesh_unit = meshes.add(bg(BAR_WIDTH));
    assets.fill_mesh_unit = meshes.add(fill(BAR_WIDTH));
    assets.bg_mesh_struct = meshes.add(bg(TOWER_BAR_WIDTH));
    assets.fill_mesh_struct = meshes.add(fill(TOWER_BAR_WIDTH));
    assets.bg_mat = materials.add(StandardMaterial {
        base_color: BAR_BG_COLOR,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
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
    /// the current HP fraction whenever the owner's HP changes.
    pub fill: Entity,
}

pub fn spawn_health_bar_for_unit(commands: &mut Commands, owner: Entity) {
    spawn_bar(commands, owner, UNIT_BAR_HEIGHT, BAR_WIDTH, false);
}

pub fn spawn_health_bar_for_tower(commands: &mut Commands, owner: Entity) {
    spawn_bar(commands, owner, TOWER_BAR_HEIGHT, TOWER_BAR_WIDTH, true);
}

pub fn spawn_health_bar_for_base(commands: &mut Commands, owner: Entity) {
    spawn_bar(commands, owner, BASE_BAR_HEIGHT, TOWER_BAR_WIDTH, true);
}

fn spawn_bar(commands: &mut Commands, owner: Entity, height: f32, width: f32, always: bool) {
    // Deferred: the shared handles live in HealthBarAssets and the per-bar
    // fill material needs Assets access, so the spawn happens through a queued
    // command with world access.
    commands.queue(move |world: &mut World| {
        let (bg_mesh, fill_mesh, bg_mat) = {
            let assets = world.resource::<HealthBarAssets>();
            if always {
                // Structures (towers/bases) use the wide pair.
                (
                    assets.bg_mesh_struct.clone(),
                    assets.fill_mesh_struct.clone(),
                    assets.bg_mat.clone(),
                )
            } else {
                (
                    assets.bg_mesh_unit.clone(),
                    assets.fill_mesh_unit.clone(),
                    assets.bg_mat.clone(),
                )
            }
        };
        // Per-bar: the fill material is mutated (re-tinted) as HP drops.
        let fill_mat = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: BAR_FILL_FULL,
                unlit: true,
                ..default()
            });

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
                },
            ))
            .add_child(fill);
    });
}

/// Updates every health bar: position above its owner and Y-only billboarding
/// run each frame, but the fill scale / tint / visibility are only rewritten
/// when the owner's HP actually changed (or the bar was just spawned) —
/// mutating `Assets<StandardMaterial>` every frame for every bar would
/// invalidate material caching for no visual gain. Orphan bars (owner
/// despawned) clean themselves up.
pub fn update_health_bars(
    mut commands: Commands,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    healths: Query<(&GlobalTransform, Ref<Health>)>,
    mut bars: Query<(Entity, Ref<HealthBar>, &mut Transform, &mut Visibility)>,
    fill_mats: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fill_t: Query<&mut Transform, Without<HealthBar>>,
) {
    let Some(cam_t) = cameras.iter().next() else {
        return;
    };
    let cam_pos = cam_t.translation();
    for (entity, bar, mut transform, mut vis) in bars.iter_mut() {
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

        // Fill/tint/visibility only when the HP moved or the bar is new (a bar
        // spawns one flush after its owner, so it may miss the owner's
        // initial Changed<Health> tick — `is_added` covers that frame).
        if !hp.is_changed() && !bar.is_added() {
            continue;
        }

        let max = hp.max.max(1) as f32;
        let cur = hp.current.max(0) as f32;
        let frac = (cur / max).clamp(0.0, 1.0);

        *vis = if bar.always_visible || frac < 0.999 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        if let Ok(mut t) = fill_t.get_mut(bar.fill) {
            let inner_w = bar.width * FILL_WIDTH_RATIO;
            let scaled = inner_w * frac;
            t.scale = Vec3::new(frac.max(0.0001), 1.0, 1.0);
            // Anchor the fill at the left edge so it shrinks from the right.
            t.translation = Vec3::new(-(inner_w - scaled) * 0.5, 0.0, BAR_DEPTH * 0.6);
        }

        // Re-tint the fill green → red as HP drops.
        if let Ok(mat_handle) = fill_mats.get(bar.fill)
            && let Some(mat) = materials.get_mut(&mat_handle.0)
        {
            let full = BAR_FILL_FULL.to_srgba();
            let low = BAR_FILL_LOW.to_srgba();
            mat.base_color = Color::srgb(
                low.red * (1.0 - frac) + full.red * frac,
                low.green * (1.0 - frac) + full.green * frac,
                low.blue * (1.0 - frac) + full.blue * frac,
            );
        }
    }
}
