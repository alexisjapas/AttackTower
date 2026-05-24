use bevy::prelude::*;

use crate::common::*;
use crate::units::spawn_arrow;

pub fn spawn_tower(commands: &mut Commands, lib: &MatLibrary, side: Side, position: Vec3) {
    let main_mat = match side {
        Side::Left => lib.left.clone(),
        Side::Right => lib.right.clone(),
    };
    let flag_mesh_size = 0.30;

    commands
        .spawn((
            Transform::from_xyz(position.x, 0.0, position.z),
            Visibility::default(),
            Tower,
            side,
            Health::new(TOWER_HP),
            Damage(TOWER_DAMAGE),
            AttackCooldown::ready(TOWER_COOLDOWN),
        ))
        .with_children(|p| {
            // Foundation
            p.spawn((
                Mesh3d(lib.tower_foundation.clone()),
                MeshMaterial3d(lib.stone_dark.clone()),
                Transform::from_xyz(0.0, 0.15, 0.0),
            ));
            // Shaft
            p.spawn((
                Mesh3d(lib.tower_shaft.clone()),
                MeshMaterial3d(lib.stone_light.clone()),
                Transform::from_xyz(0.0, 1.1, 0.0),
            ));
            // Battlement slab
            p.spawn((
                Mesh3d(lib.tower_top_slab.clone()),
                MeshMaterial3d(lib.stone_dark.clone()),
                Transform::from_xyz(0.0, 1.98, 0.0),
            ));
            // Crenellations around the slab edge
            let crenel_y = 2.17;
            for &(cx, cz) in &[
                (0.48, 0.0),
                (-0.48, 0.0),
                (0.0, 0.48),
                (0.0, -0.48),
                (0.36, 0.36),
                (-0.36, 0.36),
                (0.36, -0.36),
                (-0.36, -0.36),
            ] {
                p.spawn((
                    Mesh3d(lib.tower_crenel.clone()),
                    MeshMaterial3d(lib.stone_light.clone()),
                    Transform::from_xyz(cx, crenel_y, cz),
                ));
            }
            // Colored conical roof on top
            p.spawn((
                Mesh3d(lib.tower_roof.clone()),
                MeshMaterial3d(main_mat.clone()),
                Transform::from_xyz(0.0, 2.55, 0.0),
            ));
            // Small flag at the very top
            p.spawn((
                Mesh3d(lib.tower_crenel.clone()),
                MeshMaterial3d(main_mat),
                Transform {
                    translation: Vec3::new(0.0, 2.95, 0.0),
                    scale: Vec3::new(flag_mesh_size, flag_mesh_size, flag_mesh_size),
                    ..default()
                },
            ));
        });
}

pub fn tower_attack_tick(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    lib: Res<MatLibrary>,
    mut towers: Query<(&Side, &Transform, &Damage, &mut AttackCooldown), With<Tower>>,
    units: Query<(Entity, &Side, &Transform), (With<Unit>, Without<Tower>)>,
    bases: Query<(Entity, &Side, &Transform), (With<Base>, Without<Tower>)>,
) {
    if *state != GameState::Playing {
        return;
    }

    for (side, transform, damage, mut cooldown) in towers.iter_mut() {
        let pos = transform.translation;
        let mut nearest: Option<(Entity, Vec3, f32)> = None;
        let mut consider = |entity: Entity, target_pos: Vec3| {
            let d = (target_pos.x - pos.x).hypot(target_pos.z - pos.z);
            if nearest.map_or(true, |(_, _, nd)| d < nd) {
                nearest = Some((entity, target_pos, d));
            }
        };
        for (entity, s, t) in units.iter() {
            if s != side {
                consider(entity, t.translation);
            }
        }
        for (entity, s, t) in bases.iter() {
            if s != side {
                consider(entity, t.translation);
            }
        }

        if let Some((target_entity, target_pos, dist)) = nearest {
            if dist <= TOWER_RANGE {
                cooldown.0.tick(time.delta());
                if cooldown.0.just_finished() {
                    let start = pos + Vec3::new(0.0, TOWER_ARROW_HEIGHT, 0.0);
                    spawn_arrow(
                        &mut commands,
                        &lib,
                        *side,
                        start,
                        target_entity,
                        target_pos,
                        damage.0,
                    );
                }
            }
        }
    }
}

pub fn cleanup_dead_towers(mut commands: Commands, towers: Query<(Entity, &Health), With<Tower>>) {
    for (entity, hp) in &towers {
        if hp.current <= 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn is_valid_tower_zone(side: Side, pos: Vec3) -> bool {
    let z_ok = pos.z.abs() <= TOWER_PLACEMENT_Z_LIMIT;
    if !z_ok {
        return false;
    }
    match side {
        Side::Left => pos.x >= LEFT_BASE_X + TOWER_PLACEMENT_MARGIN && pos.x <= -ZONE_BOUNDARY,
        Side::Right => pos.x >= ZONE_BOUNDARY && pos.x <= RIGHT_BASE_X - TOWER_PLACEMENT_MARGIN,
    }
}
