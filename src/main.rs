mod common;
mod game;
mod setup;
mod towers;
mod ui;
mod units;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::common::*;
use crate::game::*;
use crate::setup::*;
use crate::towers::*;
use crate::ui::*;
use crate::units::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AttackTower".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.10)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 280.0,
            affects_lightmapped_meshes: true,
        })
        .init_resource::<Gold>()
        .init_resource::<MatLibrary>()
        .init_resource::<GameState>()
        .init_resource::<PlacementMode>()
        .init_resource::<PlayerControllers>()
        .init_resource::<MenuFocus>()
        .init_resource::<GameSettings>()
        .init_resource::<SettingsOrigin>()
        .add_systems(Startup, (init_mat_library, setup_world, setup_ui).chain())
        .add_systems(
            Update,
            (
                (
                    menu_input_system,
                    settings_input_system,
                    pause_input_system,
                    sideselect_input_system,
                    gameplay_input_system,
                    placement_system,
                    combat_tick,
                    tower_attack_tick,
                    arrow_flight_system,
                    process_damage_effects,
                    animate_units,
                    cleanup_dead_units,
                    cleanup_dead_towers,
                )
                    .chain(),
                (
                    check_winner,
                    manage_input_components,
                    update_menu_overlay,
                    update_settings_overlay,
                    update_pause_overlay,
                    update_sideselect_overlay,
                    update_endgame_overlay,
                    update_sideselect_cards,
                    update_fullscreen_toggle_text,
                    apply_menu_focus_visual,
                    apply_player_focus_visual,
                    apply_window_settings,
                    update_gold_text,
                    update_base_hp_text,
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}
