#![allow(clippy::type_complexity, clippy::too_many_arguments)]

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
        .init_resource::<PlayerNations>()
        .init_resource::<MenuFocus>()
        .insert_resource(settings)
        .init_resource::<GraphicsPreset>()
        .init_resource::<SettingsTab>()
        .init_resource::<SettingsOrigin>()
        .init_resource::<TimeOfDay>()
        .init_resource::<DlssAvailable>()
        .init_resource::<GameTime>()
        .init_resource::<GameMode>()
        .init_resource::<ArcherAssets>()
        .init_resource::<EnvAssets>()
        .add_systems(
            Startup,
            (
                init_mat_library,
                load_env_assets,
                setup_world,
                setup_ui,
                setup_music,
                load_archer_assets,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (
                AppSet::Input,
                AppSet::World,
                AppSet::React,
                AppSet::Visual,
                AppSet::FrameLimit,
            )
                .chain(),
        )
        // Input: each system gates on a single GameState and may mutate it,
        // so a global chain is needed inside the set.
        .add_systems(
            Update,
            (
                menu_input_system,
                settings_input_system,
                pause_input_system,
                sideselect_input_system,
                gameplay_input_system,
                placement_system,
            )
                .chain()
                .in_set(AppSet::Input),
        )
        // World: gameplay tick. combat → damage → animate → cleanup forms a
        // dependency chain (death state propagation). spawn/time/animate_sun
        // are independent — Bevy will parallelise them.
        .add_systems(
            Update,
            (
                advance_game_time,
                animate_sun,
                spawn_arena,
                spawn_initial_miners,
                build_archer_graph,
                bind_archer_animation_player,
                bind_archer_bow_hand,
                (
                    (combat_tick, tower_attack_tick, arrow_flight_system),
                    process_damage_effects,
                    (animate_units, animate_archer),
                    (cleanup_dead_units, cleanup_dead_towers),
                )
                    .chain(),
            )
                .in_set(AppSet::World),
        )
        // React: systems that read `state.is_changed()` to rebuild overlays
        // and react to mutations made by Input + World. `check_winner` and
        // `detect_pad_disconnect` mutate state, so they run first; the
        // overlays then rebuild in parallel since they touch independent
        // marker components.
        .add_systems(
            Update,
            (
                (check_winner, detect_pad_disconnect),
                manage_input_components,
                (
                    update_menu_overlay,
                    update_graphics_preset,
                    update_settings_overlay,
                    update_pause_overlay,
                    update_sideselect_overlay,
                    update_endgame_overlay,
                    update_game_hud_visibility,
                    update_torches,
                    sync_raytracing_meshes,
                ),
            )
                .chain()
                .in_set(AppSet::React),
        )
        // Visual: text refreshes, healthbar billboarding, settings application
        // — most run in parallel. The chain is only used where a system
        // observes the result of another (focus visuals need overlays settled,
        // settings description reads the active preset, etc.).
        .add_systems(
            Update,
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
                apply_colorblind_palette,
                apply_dlss_setting,
                persist_settings,
                sync_music_playback,
                update_gold_text,
                update_base_hp_text,
                update_focus_stats_text,
                update_clock_text,
                update_health_bars,
                debug_camera_control,
            )
                .in_set(AppSet::Visual),
        )
        // Last: throttle the frame so VSync-off configs hit the FPS cap. Must
        // run after every other system or its sleep gets paid before work.
        .add_systems(Update, limit_fps.in_set(AppSet::FrameLimit))
        .run();
}

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug, Copy)]
enum AppSet {
    Input,
    World,
    React,
    Visual,
    FrameLimit,
}
