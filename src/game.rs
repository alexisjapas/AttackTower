use bevy::prelude::*;

use crate::common::*;

pub fn check_winner(bases: Query<(&Side, &Health), With<Base>>, mut state: ResMut<GameState>) {
    if *state != GameState::Playing {
        return;
    }
    for (side, hp) in &bases {
        if hp.current <= 0 {
            *state = GameState::Ended(side.opposite());
            return;
        }
    }
}
