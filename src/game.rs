use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::common::*;

pub fn check_winner(
    mut commands: Commands,
    bases: Query<(Entity, &Side, &Health, Option<&BaseDestroyed>), With<Base>>,
    mut state: ResMut<GameState>,
) {
    if *state != GameState::Playing {
        return;
    }
    // Mark freshly-destroyed bases so combat/HUD systems treat them as gone.
    // The collider goes with the marker: a ruin neither blocks enemy units nor
    // soaks stray arrows (relevant in 2v2, where play continues around it).
    for (entity, _, hp, destroyed) in &bases {
        if hp.current <= 0 && destroyed.is_none() {
            commands
                .entity(entity)
                .insert(BaseDestroyed::default())
                .remove::<Collider>();
        }
    }
    // A side is defeated only when ALL of its bases are at 0 HP. In 1v1 that's
    // a single base; in 2v2 both allied bases must fall.
    let mut left_alive = false;
    let mut right_alive = false;
    let mut left_seen = false;
    let mut right_seen = false;
    for (_, side, hp, _) in &bases {
        match side {
            Side::Left => {
                left_seen = true;
                if hp.current > 0 {
                    left_alive = true;
                }
            }
            Side::Right => {
                right_seen = true;
                if hp.current > 0 {
                    right_alive = true;
                }
            }
        }
    }
    let left_defeated = left_seen && !left_alive;
    let right_defeated = right_seen && !right_alive;
    match (left_defeated, right_defeated) {
        // Both sides' last bases fell on the same frame: a draw, not an
        // arbitrary left-checked-first win.
        (true, true) => *state = GameState::Ended(None),
        (true, false) => *state = GameState::Ended(Some(Side::Right)),
        (false, true) => *state = GameState::Ended(Some(Side::Left)),
        (false, false) => {}
    }
}

/// Sink a destroyed base into the ground over `BASE_COLLAPSE_DURATION` so the
/// destruction reads on screen (the model otherwise stands untouched). Also
/// runs during `Ended` so the killing blow's collapse is visible behind the
/// endgame overlay; frozen while paused like the rest of the gameplay.
pub fn collapse_destroyed_bases(
    time: Res<Time>,
    state: Res<GameState>,
    mut bases: Query<(&mut Transform, &mut BaseDestroyed)>,
) {
    if !matches!(*state, GameState::Playing | GameState::Ended(_)) {
        return;
    }
    let dt = time.delta_secs();
    for (mut transform, mut destroyed) in &mut bases {
        if destroyed.t >= BASE_COLLAPSE_DURATION {
            continue;
        }
        destroyed.t += dt;
        let p = (destroyed.t / BASE_COLLAPSE_DURATION).clamp(0.0, 1.0);
        // Smoothstep ease so the sink starts and ends gently.
        let ease = p * p * (3.0 - 2.0 * p);
        transform.translation.y = -BASE_COLLAPSE_SINK * ease;
    }
}
