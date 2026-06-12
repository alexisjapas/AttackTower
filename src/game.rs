use bevy::prelude::*;

use crate::common::*;

/// Match lifecycle: owns the `GameState` machine (and the derived `InMatch`
/// computed state), the per-match resources, the win condition and the
/// between-match cleanup.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_computed_state::<InMatch>()
            .init_resource::<Winner>()
            .init_resource::<Gold>()
            .init_resource::<GameMode>()
            // Fires on the initial Menu entry at boot too, where it is a no-op
            // over empty defaults.
            .add_systems(OnEnter(GameState::Menu), reset_match)
            .add_systems(
                Update,
                check_winner
                    .run_if(in_state(GameState::Playing))
                    .in_set(AppSet::React),
            );
    }
}

pub fn check_winner(
    mut commands: Commands,
    bases: Query<(Entity, &Side, &Health, Option<&BaseDestroyed>), With<Base>>,
    mut winner: ResMut<Winner>,
    mut next: ResMut<NextState<GameState>>,
) {
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
    let victor = if left_seen && !left_alive {
        Some(Side::Right)
    } else if right_seen && !right_alive {
        Some(Side::Left)
    } else {
        None
    };
    if let Some(side) = victor {
        winner.0 = Some(side);
        next.set(GameState::Ended);
    }
}

/// Query filter for everything that belongs to a live match and must be wiped
/// on reset (bases and rocks included, since GameMode may change before the
/// next match). Bundled into one filter so `reset_match` stays under Bevy's
/// system-parameter count limit.
type BattlefieldEntity = Or<(
    With<Base>,
    With<Rock>,
    With<Unit>,
    With<Arrow>,
    With<Tower>,
    With<TowerGhost>,
)>;

/// OnEnter(Menu): wipe any finished/abandoned match — despawn every
/// battlefield entity and reset gold, placement, player→pad mapping, winner
/// and the day/night clock (so a relaunched match opens at the same morning).
/// The arena is rebuilt by `spawn_arena` on the next `InMatch` entry. Both the
/// pause and endgame "Main menu" actions land here by simply switching state.
fn reset_match(
    mut commands: Commands,
    battlefield: Query<Entity, BattlefieldEntity>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut players: ResMut<PlayerControllers>,
    mut winner: ResMut<Winner>,
    mut gtime: ResMut<GameTime>,
    mut tod: ResMut<TimeOfDay>,
) {
    for e in &battlefield {
        commands.entity(e).despawn();
    }
    *gold = Gold::default();
    *placement = PlacementMode::default();
    *players = PlayerControllers::default();
    *winner = Winner::default();
    *gtime = GameTime::default();
    *tod = TimeOfDay::default();
}
