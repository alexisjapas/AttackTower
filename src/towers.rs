use bevy::prelude::*;

use crate::common::*;
use crate::units::spawn_arrow;

pub fn spawn_tower(commands: &mut Commands, lib: &MatLibrary, slot: PlayerSlot, position: Vec3) {
    let side = slot.side();
    let main_mat = match side {
        Side::Left => lib.left.clone(),
        Side::Right => lib.right.clone(),
    };
    let flag_mesh_size = 0.30;

    let tower_entity = commands
        .spawn((
            Transform::from_xyz(position.x, 0.0, position.z),
            Visibility::default(),
            Tower,
            side,
            slot,
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
            // Torch on the battlement (one). Lit only at night by update_torches.
            p.spawn((
                Mesh3d(lib.torch_pole_mesh.clone()),
                MeshMaterial3d(lib.wood_mat.clone()),
                Transform::from_xyz(0.0, 2.40, 0.42),
            ));
            p.spawn((
                Mesh3d(lib.flame_mesh.clone()),
                MeshMaterial3d(lib.flame_mat.clone()),
                Transform::from_xyz(0.0, 2.62, 0.42),
                Visibility::Hidden,
                TorchFlame,
            ));
            p.spawn((
                PointLight {
                    color: TORCH_COLOR,
                    intensity: 0.0,
                    range: TORCH_RANGE,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(0.0, 2.65, 0.42),
                TorchLight,
            ));
        })
        .id();
    crate::healthbar::spawn_health_bar_for_tower(commands, tower_entity);
}

pub fn tower_attack_tick(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    lib: Res<MatLibrary>,
    mut towers: Query<
        (&Side, &Transform, &Damage, &mut AttackCooldown),
        (With<Tower>, Without<TowerDying>),
    >,
    units: Query<(Entity, &Side, &Transform), (With<Unit>, Without<Tower>)>,
    bases: Query<(Entity, &Side, &Transform), (With<Base>, Without<Tower>, Without<BaseDestroyed>)>,
) {
    if *state != GameState::Playing {
        return;
    }

    for (side, transform, damage, mut cooldown) in towers.iter_mut() {
        let pos = transform.translation;
        let mut nearest: Option<(Entity, Vec3, f32)> = None;
        let mut consider = |entity: Entity, target_pos: Vec3| {
            let d = (target_pos.x - pos.x).hypot(target_pos.z - pos.z);
            if nearest.is_none_or(|(_, _, nd)| d < nd) {
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

        if let Some((target_entity, target_pos, dist)) = nearest
            && dist <= TOWER_RANGE
        {
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

pub fn cleanup_dead_towers(
    mut commands: Commands,
    time: Res<Time>,
    mut towers: Query<(Entity, &Health, &mut Transform, Option<&mut TowerDying>), With<Tower>>,
) {
    let dt = time.delta_secs();
    for (entity, hp, mut transform, dying) in towers.iter_mut() {
        if hp.current > 0 {
            continue;
        }
        match dying {
            None => {
                commands.entity(entity).insert(TowerDying::default());
            }
            Some(mut d) => {
                d.t += dt;
                let progress = (d.t / TOWER_DEATH_DURATION).clamp(0.0, 1.0);
                // Tilt forward + sink: dramatic enough to read at a glance,
                // small enough not to clip into neighbours.
                transform.rotation = Quat::from_rotation_z(-progress * 0.9);
                transform.translation.y = -progress * 0.4;
                if progress >= 1.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn is_valid_tower_zone(side: Side, pos: Vec3, mode: GameMode) -> bool {
    let z_ok = pos.z.abs() <= mode.tower_z_limit();
    if !z_ok {
        return false;
    }
    match side {
        Side::Left => pos.x >= LEFT_BASE_X + TOWER_PLACEMENT_MARGIN && pos.x <= -ZONE_BOUNDARY,
        Side::Right => pos.x >= ZONE_BOUNDARY && pos.x <= RIGHT_BASE_X - TOWER_PLACEMENT_MARGIN,
    }
}

pub fn collides_with_existing_tower(pos: Vec3, towers: &[Vec3]) -> bool {
    towers.iter().any(|t| {
        let dx = t.x - pos.x;
        let dz = t.z - pos.z;
        (dx * dx + dz * dz).sqrt() < TOWER_MIN_SEPARATION
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f32, z: f32) -> Vec3 {
        Vec3::new(x, 0.0, z)
    }

    #[test]
    fn placement_inside_left_zone_accepted() {
        let pos = p(-7.0, 0.0);
        assert!(is_valid_tower_zone(Side::Left, pos, GameMode::OneVsOne));
    }

    #[test]
    fn placement_outside_zone_rejected() {
        // Past the neutral strip in the enemy half.
        let pos = p(5.0, 0.0);
        assert!(!is_valid_tower_zone(Side::Left, pos, GameMode::OneVsOne));
    }

    #[test]
    fn placement_too_far_on_z_rejected_in_1v1() {
        // In 1v1 the Z limit is tight, so a tower at large |z| should be refused.
        let limit = GameMode::OneVsOne.tower_z_limit();
        let pos = p(-7.0, limit + 1.0);
        assert!(!is_valid_tower_zone(Side::Left, pos, GameMode::OneVsOne));
    }

    #[test]
    fn placement_at_large_z_accepted_in_2v2() {
        // The same Z that fails in 1v1 must succeed in 2v2 (where the second
        // base is offset and the lane Z range extends further).
        let pos = p(-7.0, GameMode::OneVsOne.tower_z_limit() + 0.5);
        assert!(is_valid_tower_zone(Side::Left, pos, GameMode::TwoVsTwo));
    }

    #[test]
    fn collision_detects_overlap() {
        let existing = [p(0.0, 0.0)];
        assert!(collides_with_existing_tower(
            p(TOWER_MIN_SEPARATION * 0.5, 0.0),
            &existing
        ));
        assert!(!collides_with_existing_tower(
            p(TOWER_MIN_SEPARATION * 2.0, 0.0),
            &existing
        ));
    }
}
