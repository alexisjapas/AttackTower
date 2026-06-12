#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod common;
mod config;
mod game;
mod graphics;
mod healthbar;
mod music;
mod placement;
mod setup;
mod towers;
mod ui;
mod units;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::common::{AppSet, CombatSet, RaytracingAvailable};
use crate::game::GamePlugin;
use crate::graphics::{GraphicsSettingsPlugin, load_settings, sanitize_settings};
use crate::healthbar::HealthBarPlugin;
use crate::music::MusicPlugin;
use crate::setup::{SetupPlugin, probe_raytracing_support};
use crate::towers::TowersPlugin;
use crate::ui::UiPlugin;
use crate::units::UnitsPlugin;

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
        .add_plugins(PhysicsPlugins::default())
        // Everything is ground-bound (units lock their Y axis), so global gravity
        // would only add solver work — disable it.
        .insert_resource(Gravity(Vec3::ZERO));
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
        .insert_resource(settings)
        // Cross-module schedule skeleton: the five chained Update phases, and
        // the combat sub-phases within World (damage → death state → animation
        // → despawn must land in one frame). Plugins hang their systems on
        // these sets; ordering decisions all live here.
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
        .configure_sets(
            Update,
            (
                CombatSet::Attack,
                CombatSet::ApplyDamage,
                CombatSet::Animate,
                CombatSet::Cleanup,
            )
                .chain()
                .in_set(AppSet::World),
        )
        .add_plugins((
            GamePlugin,
            SetupPlugin,
            UnitsPlugin,
            TowersPlugin,
            HealthBarPlugin,
            MusicPlugin,
            GraphicsSettingsPlugin,
            UiPlugin,
        ))
        .run();
}
