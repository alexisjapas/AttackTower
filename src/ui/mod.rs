use bevy::prelude::*;

use crate::common::*;
use crate::placement::{clear_placement, placement_system};

mod hud;
mod input;
mod overlays;

pub use hud::*;
pub use input::*;
pub use overlays::*;

// Shared button/panel palette used across the HUD and the overlays.
pub(crate) const BTN_NORMAL: Color = Color::srgb(0.16, 0.16, 0.20);
pub(crate) const BTN_FOCUSED: Color = Color::srgb(0.32, 0.32, 0.40);
pub(crate) const BTN_DISABLED: Color = Color::srgb(0.10, 0.10, 0.12);
pub(crate) const BORDER_DISABLED: Color = Color::srgb(0.30, 0.30, 0.34);
pub(crate) const HUD_BG_DISABLED: Color = Color::srgba(0.0, 0.0, 0.0, 0.40);
pub(crate) const HUD_BORDER_DISABLED: Color = Color::srgb(0.40, 0.40, 0.44);
pub(crate) const CARD_NORMAL: Color = Color::srgb(0.12, 0.13, 0.18);
pub(crate) const CARD_HOVERED: Color = Color::srgb(0.22, 0.23, 0.30);

/// Every Bevy UI node (HUD + state overlays) and the per-state input systems.
/// Overlays are spawned/torn down by `OnEnter`/`OnExit` transitions; input
/// systems are gated by `run_if(in_state(..))`, and since `NextState` applies
/// between frames, the system of the new state never sees the press that
/// caused the transition.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseUi>()
            .init_resource::<MenuFocus>()
            .init_resource::<PlacementMode>()
            .init_resource::<PlayerControllers>()
            .init_resource::<PlayerNations>()
            .add_systems(Startup, setup_ui)
            // Input: read_mouse_ui first (the others consume its snapshot);
            // the per-state systems are mutually exclusive via run_if but stay
            // chained so the mouse snapshot is always fresh.
            .add_systems(
                Update,
                (
                    read_mouse_ui,
                    menu_input_system.run_if(in_state(GameState::Menu)),
                    endgame_input_system.run_if(in_state(GameState::Ended)),
                    settings_input_system.run_if(in_state(GameState::Settings)),
                    pause_input_system.run_if(in_state(GameState::Paused)),
                    sideselect_input_system.run_if(in_state(GameState::SideSelect)),
                    gameplay_input_system.run_if(in_state(GameState::Playing)),
                    placement_system.run_if(in_state(GameState::Playing)),
                )
                    .chain()
                    .in_set(AppSet::Input),
            )
            // Overlays: built on state entry, torn down on exit. The pause and
            // settings overlays keep an Update refresher for in-state rebuilds
            // (pad disconnect warning / structural slot changes).
            .add_systems(OnEnter(GameState::Menu), spawn_menu_overlay)
            .add_systems(OnExit(GameState::Menu), despawn_all::<MenuOverlay>)
            .add_systems(OnExit(GameState::Settings), despawn_all::<SettingsOverlay>)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_overlay)
            .add_systems(OnExit(GameState::Paused), despawn_all::<PauseOverlay>)
            .add_systems(OnEnter(GameState::SideSelect), spawn_sideselect_overlay)
            .add_systems(
                OnExit(GameState::SideSelect),
                (despawn_all::<SideSelectOverlay>, clear_seat_selections),
            )
            .add_systems(OnEnter(GameState::Ended), spawn_endgame_overlay)
            .add_systems(OnExit(GameState::Ended), despawn_all::<EndgameOverlay>)
            // Match lifecycle: HUD, per-pad focus and tower placement follow
            // the InMatch computed state (Playing | Paused).
            .add_systems(OnEnter(InMatch), (show_game_hud, grant_player_focus))
            .add_systems(
                OnExit(InMatch),
                (hide_game_hud, revoke_player_focus, clear_placement),
            )
            .add_systems(
                Update,
                (
                    detect_pad_disconnect.run_if(in_state(InMatch)),
                    refresh_pause_overlay.run_if(in_state(GameState::Paused)),
                    update_settings_overlay.run_if(in_state(GameState::Settings)),
                )
                    .in_set(AppSet::React),
            )
            .add_systems(
                Update,
                (
                    update_sideselect_cards.run_if(in_state(GameState::SideSelect)),
                    (update_settings_toggle_texts, update_settings_description)
                        .run_if(in_state(GameState::Settings)),
                    scroll_focused_into_view.run_if(in_state(GameState::Settings)),
                    apply_menu_focus_visual,
                    apply_player_focus_visual,
                    update_gold_text,
                    update_cell_costs,
                    update_hint_text,
                    update_base_hp_text,
                    update_focus_stats_text,
                    update_clock_text,
                )
                    .in_set(AppSet::Visual),
            );
    }
}
