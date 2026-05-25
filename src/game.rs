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
    for (entity, _, hp, destroyed) in &bases {
        if hp.current <= 0 && destroyed.is_none() {
            commands.entity(entity).insert(BaseDestroyed);
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
    if left_seen && !left_alive {
        *state = GameState::Ended(Side::Right);
    } else if right_seen && !right_alive {
        *state = GameState::Ended(Side::Left);
    }
}
