use avian3d::prelude::*;
use bevy::prelude::*;

use crate::common::*;
use crate::units::spawn_arrow;

/// Tower aiming/firing and the collapse-on-death animation. Construction
/// (`spawn_tower`) is driven by the placement flow in `ui.rs`.
pub struct TowersPlugin;

impl Plugin for TowersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tower_attack_tick
                    .run_if(in_state(GameState::Playing))
                    .in_set(CombatSet::Attack),
                cleanup_dead_towers.in_set(CombatSet::Cleanup),
            ),
        );
    }
}

pub fn spawn_tower(
    commands: &mut Commands,
    lib: &MatLibrary,
    env: &EnvAssets,
    slot: PlayerSlot,
    position: Vec3,
) {
    let side = slot.side();

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
            // Static obstacle: units flow around towers (replaces the old manual
            // sidestep). The entity stays unrotated so aiming/arrows are unaffected.
            RigidBody::Static,
            Collider::cylinder(TOWER_RADIUS, TOWER_HEIGHT),
            side.structure_layers(),
        ))
        .with_children(|p| {
            // The tower building model. Faces the center: side mirroring +
            // model yaw offset (the entity itself stays unrotated so aiming and
            // arrow spawns are unaffected).
            p.spawn((
                SceneRoot(env.tower.clone()),
                Transform::from_xyz(0.0, TOWER_MODEL_LIFT, 0.0)
                    .with_rotation(
                        side.base_rotation() * Quat::from_rotation_y(TOWER_MODEL_YAW_OFFSET),
                    )
                    .with_scale(Vec3::splat(TOWER_MODEL_SCALE)),
            ));
            // One torch on the battlement; lit only at night by `update_torches`.
            crate::setup::spawn_torch(
                p,
                lib,
                Vec3::new(0.0, TOWER_TORCH_POLE_Y, TOWER_TORCH_FORWARD),
            );
        })
        .id();
    crate::healthbar::spawn_health_bar_for_tower(commands, tower_entity);
}

pub fn tower_attack_tick(
    mut commands: Commands,
    time: Res<Time>,
    lib: Res<MatLibrary>,
    mut towers: Query<
        (Entity, &Side, &Transform, &Damage, &mut AttackCooldown),
        (With<Tower>, Without<TowerDying>),
    >,
    units: Query<(&Side, &Transform), (With<Unit>, Without<Tower>)>,
    bases: Query<(&Side, &Transform), (With<Base>, Without<Tower>, Without<BaseDestroyed>)>,
) {
    for (tower_entity, side, transform, damage, mut cooldown) in towers.iter_mut() {
        let pos = transform.translation;
        // Aim at the nearest enemy's position; the arrow then damages whatever
        // enemy it flies through (or plants in the ground on a miss).
        let mut nearest: Option<(Vec3, f32)> = None;
        let mut consider = |target_pos: Vec3| {
            let d = (target_pos.x - pos.x).hypot(target_pos.z - pos.z);
            if nearest.is_none_or(|(_, nd)| d < nd) {
                nearest = Some((target_pos, d));
            }
        };
        for (s, t) in units.iter() {
            if s != side {
                consider(t.translation);
            }
        }
        for (s, t) in bases.iter() {
            if s != side {
                consider(t.translation);
            }
        }

        if let Some((target_pos, dist)) = nearest
            && dist <= TOWER_RANGE
        {
            cooldown.0.tick(time.delta());
            if cooldown.0.just_finished() {
                let start = pos + Vec3::new(0.0, TOWER_ARROW_HEIGHT, 0.0);
                let aim = Vec3::new(target_pos.x, 0.0, target_pos.z);
                spawn_arrow(
                    &mut commands,
                    &lib,
                    *side,
                    start,
                    aim,
                    tower_entity,
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
