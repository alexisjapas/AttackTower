mod common;
mod game;
mod graphics;
mod healthbar;
mod music;
mod setup;
mod towers;
mod ui;
mod units;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::common::*;
use crate::game::*;
use crate::graphics::*;
use crate::healthbar::*;
use crate::music::*;
use crate::setup::*;
use crate::towers::*;
use crate::ui::*;
use crate::units::*;

fn main() {
    let mut app = App::new();
    #[cfg(all(feature = "dlss", not(feature = "force_disable_dlss")))]
    app.insert_resource(bevy::anti_alias::dlss::DlssProjectId(
        bevy::asset::uuid::uuid!("a4c91e02-d6fe-4b30-9277-91e8c6f4a9d3"),
    ));
    // Decided once, before the renderer is built: if the adapter can't actually
    // service Solari, we skip both the plugin and its feature request so the
    // renderer never tries to allocate BLAS/TLAS on hardware that reports
    // limits of 0.
    let raytracing_supported = probe_raytracing_support();
    let mut settings = load_settings();
    // Fix any invariant violation that may have been persisted (e.g. RT on
    // while HDR off → wgpu storage-binding panic on launch). DLSS support
    // isn't probed yet at this point, so pass `true` and let the runtime
    // system reconcile once DlssAvailable is detected.
    sanitize_settings(&mut settings, true, raytracing_supported);
    let default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "AttackTower".into(),
            ..default()
        }),
        ..default()
    });
    #[cfg(feature = "raytracing")]
    let default_plugins = if raytracing_supported {
        default_plugins.set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::WgpuSettings {
                features: bevy::solari::prelude::SolariPlugins::required_wgpu_features(),
                ..default()
            }
            .into(),
            ..default()
        })
    } else {
        default_plugins
    };
    app.add_plugins(default_plugins)
        .add_plugins(PhysicsPlugins::default());
    #[cfg(feature = "raytracing")]
    if raytracing_supported {
        app.add_plugins(bevy::solari::prelude::SolariPlugins);
    }
    app.insert_resource(RaytracingAvailable(raytracing_supported))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.10)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 60.0,
            affects_lightmapped_meshes: true,
        })
        .init_resource::<Gold>()
        .init_resource::<MatLibrary>()
        .init_resource::<GameState>()
        .init_resource::<PlacementMode>()
        .init_resource::<PlayerControllers>()
        .init_resource::<MenuFocus>()
        .insert_resource(settings)
        .init_resource::<GraphicsPreset>()
        .init_resource::<SettingsTab>()
        .init_resource::<SettingsOrigin>()
        .init_resource::<TimeOfDay>()
        .init_resource::<DlssAvailable>()
        .init_resource::<GameTime>()
        .init_resource::<GameMode>()
        .add_systems(
            Startup,
            (init_mat_library, setup_world, setup_ui, setup_music).chain(),
        )
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
                    advance_game_time,
                    animate_sun,
                    spawn_arena,
                    spawn_initial_miners,
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
                    (
                        check_winner,
                        manage_input_components,
                        update_menu_overlay,
                        update_graphics_preset,
                        update_settings_overlay,
                        update_pause_overlay,
                        update_sideselect_overlay,
                        update_endgame_overlay,
                        update_game_hud_visibility,
                        update_torches,
                        sync_raytracing_meshes,
                    )
                        .chain(),
                    (
                        enforce_settings_invariants,
                        apply_raytracing_setting,
                        detect_dlss_support,
                        update_sideselect_cards,
                        update_settings_toggle_texts,
                        update_settings_description,
                        scroll_focused_into_view,
                        apply_menu_focus_visual,
                        apply_player_focus_visual,
                        apply_graphics_settings,
                        apply_dlss_setting,
                        persist_settings,
                        sync_music_playback,
                        update_gold_text,
                        update_base_hp_text,
                        update_focus_stats_text,
                        update_clock_text,
                        update_health_bars,
                        limit_fps,
                    )
                        .chain(),
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}
