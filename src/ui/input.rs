//! Per-state gamepad/mouse input systems, the mouse-interaction snapshot
//! (MouseUi), seat navigation for the SideSelect screen, the HUD buy
//! actions and the pad-disconnect / player-focus lifecycle.

use bevy::prelude::*;

use crate::common::*;
use crate::graphics::{GraphicsPreset, MenuSlot, ParamId, slot_count, tab_slots};
use crate::placement::arm_placement;
use crate::units::{spawn_combat_unit, spawn_miner};

use super::*;

/// Per-frame snapshot of mouse interaction with the UI, populated by
/// [`read_mouse_ui`] at the head of the input chain and consumed by the
/// per-state input systems. This is what gives the otherwise gamepad-only game
/// clickable buttons for debugging: a hover moves the menu focus, a left-click
/// activates the focused item exactly as the gamepad's South button would.
///
/// Only the currently active overlay's buttons exist (the others are despawned
/// on state change), so the `MenuButton`-indexed fields are unambiguous across
/// the Menu / Pause / Settings / Endgame screens.
#[derive(Resource, Default, PartialEq, Eq)]
pub struct MouseUi {
    /// `MenuButton` index under the cursor (drives focus on hover).
    pub menu_hover: Option<usize>,
    /// `MenuButton` index left-clicked this frame (activate).
    pub menu_click: Option<usize>,
    /// Settings tab left-clicked this frame.
    pub tab_click: Option<SettingsTab>,
    /// HUD player-panel slot under the cursor (drives the in-game hover
    /// highlight, since a controller-less debug session has no `PlayerFocus`).
    pub panel_hover: Option<(PlayerSlot, usize)>,
    /// HUD player-panel slot left-clicked this frame (buy unit / arm tower).
    pub panel_click: Option<(PlayerSlot, usize)>,
}

/// Translate raw UI [`Interaction`] state into [`MouseUi`] intent. Runs first in
/// the input chain so the per-state systems see a fresh snapshot. A click is the
/// frame where the left mouse button goes down while a button reports `Pressed`
/// (Bevy's `ui_focus_system` sets `Pressed` only while the cursor is over the
/// node and the button is held).
pub fn read_mouse_ui(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse: ResMut<MouseUi>,
    menu_buttons: Query<(&MenuButton, &Interaction)>,
    tab_buttons: Query<(&SettingsTabButton, &Interaction)>,
    panel_buttons: Query<(&PanelSlot, &Interaction)>,
) {
    let mut next = MouseUi::default();
    let clicked = mouse_buttons.just_pressed(MouseButton::Left);
    for (btn, interaction) in &menu_buttons {
        match interaction {
            Interaction::Hovered => next.menu_hover = Some(btn.0),
            Interaction::Pressed => {
                next.menu_hover = Some(btn.0);
                if clicked {
                    next.menu_click = Some(btn.0);
                }
            }
            Interaction::None => {}
        }
    }
    for (slot, interaction) in &panel_buttons {
        match interaction {
            Interaction::Hovered => next.panel_hover = Some((slot.slot, slot.index)),
            Interaction::Pressed => {
                next.panel_hover = Some((slot.slot, slot.index));
                if clicked {
                    next.panel_click = Some((slot.slot, slot.index));
                }
            }
            Interaction::None => {}
        }
    }
    if clicked {
        for (tab, interaction) in &tab_buttons {
            if *interaction == Interaction::Pressed {
                next.tab_click = Some(tab.0);
            }
        }
    }
    // Write only when the snapshot moved, so `MouseUi.is_changed()` means
    // something (a steady hover or an idle mouse no longer dirties the
    // resource every frame). Clicks stay one-frame pulses either way.
    mouse.set_if_neq(next);
}

pub fn pause_input_system(
    mut next: ResMut<NextState<GameState>>,
    mut menu_focus: ResMut<MenuFocus>,
    mut origin: ResMut<SettingsOrigin>,
    gamepads: Query<&Gamepad>,
    mouse: Res<MouseUi>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    const SLOTS: usize = 3;
    if menu_focus.index >= SLOTS {
        menu_focus.index = 0;
    }

    let mut up = false;
    let mut down = false;
    let mut activate = false;
    // Escape mirrors gamepad Start/East to resume (keyboard debug fallback).
    let mut resume = keys.just_pressed(KeyCode::Escape);
    for pad in &gamepads {
        if pad.just_pressed(GamepadButton::DPadUp) {
            up = true;
        }
        if pad.just_pressed(GamepadButton::DPadDown) {
            down = true;
        }
        if pad.just_pressed(GamepadButton::South) {
            activate = true;
        }
        if pad.just_pressed(GamepadButton::Start) || pad.just_pressed(GamepadButton::East) {
            resume = true;
        }
    }

    if up {
        menu_focus.index = (menu_focus.index + SLOTS - 1) % SLOTS;
    }
    if down {
        menu_focus.index = (menu_focus.index + 1) % SLOTS;
    }

    // Mouse: hover moves focus, left-click activates the hovered item.
    if let Some(i) = mouse.menu_hover.filter(|i| *i < SLOTS) {
        menu_focus.index = i;
    }
    if let Some(i) = mouse.menu_click.filter(|i| *i < SLOTS) {
        menu_focus.index = i;
        activate = true;
    }

    if resume {
        next.set(GameState::Playing);
        return;
    }

    if activate {
        match menu_focus.index {
            0 => next.set(GameState::Playing),
            1 => {
                *origin = SettingsOrigin::Paused;
                next.set(GameState::Settings);
            }
            // Match teardown happens in game.rs::reset_match, OnEnter(Menu).
            2 => next.set(GameState::Menu),
            _ => {}
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Lifecycle helpers (run on state change)
// ────────────────────────────────────────────────────────────────────────────

/// Auto-pause the match if any active player's gamepad goes missing (the
/// entity disappears from the `Gamepad` query). Without this the abandoned
/// player would silently freeze in place while the other plays on, with no
/// way to recover except for the surviving pad to open the pause menu.
/// Runs only while `InMatch` (run condition).
pub fn detect_pad_disconnect(
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut players: ResMut<PlayerControllers>,
    mode: Res<GameMode>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    let mut any_lost = false;
    for &slot in mode.active_slots() {
        if let Some(entity) = players.get(slot)
            && gamepads.get(entity).is_err()
        {
            players.set(slot, None);
            any_lost = true;
        }
    }
    if any_lost && *state.get() == GameState::Playing {
        next.set(GameState::Paused);
    }
}

/// OnEnter(InMatch): give each active player's pad its HUD focus cursor.
pub fn grant_player_focus(
    mut commands: Commands,
    mode: Res<GameMode>,
    players: Res<PlayerControllers>,
    gamepads: Query<&Gamepad>,
    focuses: Query<Entity, With<PlayerFocus>>,
) {
    // `Paused → Settings → Paused` re-enters InMatch: keep the existing focus
    // (see the `InMatch` doc in common.rs)... except it was revoked on exit,
    // so this guard only matters if that ever changes. Cheap either way.
    if focuses.iter().next().is_some() {
        return;
    }
    for &slot in mode.active_slots() {
        if let Some(pad) = players.get(slot)
            && gamepads.get(pad).is_ok()
        {
            commands.entity(pad).insert(PlayerFocus { slot, index: 0 });
        }
    }
}

/// OnExit(InMatch): drop every pad's HUD focus cursor.
pub fn revoke_player_focus(mut commands: Commands, focuses: Query<Entity, With<PlayerFocus>>) {
    for entity in &focuses {
        commands.entity(entity).remove::<PlayerFocus>();
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Input systems (gamepad-only)
// ────────────────────────────────────────────────────────────────────────────

pub fn menu_input_system(
    mut next: ResMut<NextState<GameState>>,
    mut mode: ResMut<GameMode>,
    mut menu_focus: ResMut<MenuFocus>,
    mut origin: ResMut<SettingsOrigin>,
    mut exit: MessageWriter<AppExit>,
    gamepads: Query<&Gamepad>,
    mouse: Res<MouseUi>,
) {
    const SLOTS: usize = 4;

    let mut up = false;
    let mut down = false;
    let mut activate = false;
    let pad_count = gamepads.iter().count();
    for pad in &gamepads {
        if pad.just_pressed(GamepadButton::DPadUp) {
            up = true;
        }
        if pad.just_pressed(GamepadButton::DPadDown) {
            down = true;
        }
        if pad.just_pressed(GamepadButton::South) || pad.just_pressed(GamepadButton::Start) {
            activate = true;
        }
    }

    if menu_focus.index >= SLOTS {
        menu_focus.index = 0;
    }
    if up {
        menu_focus.index = (menu_focus.index + SLOTS - 1) % SLOTS;
    }
    if down {
        menu_focus.index = (menu_focus.index + 1) % SLOTS;
    }

    // Mouse: hover moves focus, left-click activates the hovered item.
    if let Some(i) = mouse.menu_hover.filter(|i| *i < SLOTS) {
        menu_focus.index = i;
    }
    if let Some(i) = mouse.menu_click.filter(|i| *i < SLOTS) {
        menu_focus.index = i;
        activate = true;
    }

    if !activate {
        return;
    }

    match menu_focus.index {
        0 if pad_count > 0 => {
            *mode = GameMode::OneVsOne;
            next.set(GameState::SideSelect);
        }
        1 if pad_count > 0 => {
            *mode = GameMode::TwoVsTwo;
            next.set(GameState::SideSelect);
        }
        // Debug launch: with no pad connected only the mouse can have fired
        // this activation, and SideSelect (which assigns pads to seats) would
        // be a dead end. Jump straight into a controller-less match so the
        // HUD buttons can be driven by mouse for debugging.
        0 => {
            *mode = GameMode::OneVsOne;
            next.set(GameState::Playing);
        }
        1 => {
            *mode = GameMode::TwoVsTwo;
            next.set(GameState::Playing);
        }
        2 => {
            *origin = SettingsOrigin::Menu;
            next.set(GameState::Settings);
        }
        3 => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

/// Victory screen input: the single "Main menu" button. The match teardown
/// itself happens in `game.rs::reset_match` on entering Menu.
pub fn endgame_input_system(
    mut next: ResMut<NextState<GameState>>,
    gamepads: Query<&Gamepad>,
    mouse: Res<MouseUi>,
) {
    let mut activate = mouse.menu_click == Some(0);
    for pad in &gamepads {
        if pad.just_pressed(GamepadButton::South) || pad.just_pressed(GamepadButton::Start) {
            activate = true;
        }
    }
    if activate {
        next.set(GameState::Menu);
    }
}

pub fn sideselect_input_system(
    mut commands: Commands,
    mut next: ResMut<NextState<GameState>>,
    mode: Res<GameMode>,
    mut players: ResMut<PlayerControllers>,
    mut nations: ResMut<PlayerNations>,
    mut seats: Query<(Entity, &Gamepad, Option<&mut SeatSelection>)>,
) {
    let two_v_two = *mode == GameMode::TwoVsTwo;
    let nation_count = Nation::ALL.len();

    // Snapshot which slots are *claimed* (a pad past seat selection) so others
    // can't hover/take them; reject same-frame conflicts.
    let mut claimed: [Option<Entity>; 4] = [None; 4];
    for (e, _, s) in seats.iter() {
        if let Some(sel) = s
            && sel.claims_seat()
        {
            claimed[sel.hovered.index()] = Some(e);
        }
    }

    let mut start_pressed = false;

    for (pad_entity, pad, seat_opt) in seats.iter_mut() {
        if pad.just_pressed(GamepadButton::Start) {
            start_pressed = true;
        }

        let locked_by_other = |pad: Entity| {
            let mut out = [false; 4];
            for (i, e) in claimed.iter().enumerate() {
                if let Some(owner) = e
                    && *owner != pad
                {
                    out[i] = true;
                }
            }
            out
        };

        match seat_opt {
            None => {
                if pad.just_pressed(GamepadButton::DPadLeft)
                    || pad.just_pressed(GamepadButton::DPadRight)
                    || pad.just_pressed(GamepadButton::DPadUp)
                    || pad.just_pressed(GamepadButton::DPadDown)
                    || pad.just_pressed(GamepadButton::South)
                {
                    let locked = locked_by_other(pad_entity);
                    let preferred = if pad.just_pressed(GamepadButton::DPadRight) {
                        PlayerSlot::RightBottom
                    } else {
                        PlayerSlot::LeftBottom
                    };
                    let hovered = if locked[preferred.index()] {
                        first_free_default(locked)
                    } else {
                        preferred
                    };
                    commands.entity(pad_entity).insert(SeatSelection {
                        hovered,
                        phase: SeatPhase::PickingSeat,
                        nation: 0,
                    });
                }
            }
            Some(mut seat) => match seat.phase {
                SeatPhase::PickingSeat => {
                    let locked = locked_by_other(pad_entity);
                    if pad.just_pressed(GamepadButton::DPadLeft) {
                        seat.hovered = move_seat(seat.hovered, SeatNav::Left, two_v_two, locked);
                    }
                    if pad.just_pressed(GamepadButton::DPadRight) {
                        seat.hovered = move_seat(seat.hovered, SeatNav::Right, two_v_two, locked);
                    }
                    if two_v_two {
                        if pad.just_pressed(GamepadButton::DPadUp) {
                            seat.hovered = move_seat(seat.hovered, SeatNav::Up, two_v_two, locked);
                        }
                        if pad.just_pressed(GamepadButton::DPadDown) {
                            seat.hovered =
                                move_seat(seat.hovered, SeatNav::Down, two_v_two, locked);
                        }
                    }
                    // Claim the seat and advance to nation pick (unless someone
                    // else grabbed it this frame).
                    if pad.just_pressed(GamepadButton::South) {
                        let taken = claimed[seat.hovered.index()].is_some_and(|e| e != pad_entity);
                        if !taken {
                            seat.phase = SeatPhase::PickingNation;
                        }
                    }
                    // Back out of the screen entirely.
                    if pad.just_pressed(GamepadButton::East) {
                        commands.entity(pad_entity).remove::<SeatSelection>();
                    }
                }
                SeatPhase::PickingNation => {
                    if pad.just_pressed(GamepadButton::DPadLeft) {
                        seat.nation = (seat.nation + nation_count - 1) % nation_count;
                    }
                    if pad.just_pressed(GamepadButton::DPadRight) {
                        seat.nation = (seat.nation + 1) % nation_count;
                    }
                    if pad.just_pressed(GamepadButton::South) {
                        seat.phase = SeatPhase::Locked;
                    }
                    // Release the seat, back to choosing position.
                    if pad.just_pressed(GamepadButton::East) {
                        seat.phase = SeatPhase::PickingSeat;
                    }
                }
                SeatPhase::Locked => {
                    // Reopen the nation choice.
                    if pad.just_pressed(GamepadButton::East) {
                        seat.phase = SeatPhase::PickingNation;
                    }
                }
            },
        }
    }

    // Launch once at least one pad is fully locked and none is still mid nation
    // pick (so every joined-and-committed player has a nation).
    if start_pressed {
        let mut locked_ctrl: [Option<Entity>; 4] = [None; 4];
        let mut locked_nat: [usize; 4] = [0; 4];
        let mut locked_any = false;
        let mut mid_nation = false;
        for (e, _, s) in seats.iter() {
            match s.map(|sel| (sel.phase, sel.hovered, sel.nation)) {
                Some((SeatPhase::Locked, slot, nation)) => {
                    locked_ctrl[slot.index()] = Some(e);
                    locked_nat[slot.index()] = nation;
                    locked_any = true;
                }
                Some((SeatPhase::PickingNation, _, _)) => mid_nation = true,
                _ => {}
            }
        }
        if locked_any && !mid_nation {
            let mut next_controllers = PlayerControllers::default();
            let mut next_nations = PlayerNations::default();
            for &slot in &PlayerSlot::ALL {
                next_controllers.set(slot, locked_ctrl[slot.index()]);
                next_nations.set(slot, Nation::ALL[locked_nat[slot.index()] % nation_count]);
            }
            *players = next_controllers;
            *nations = next_nations;
            next.set(GameState::Playing);
        }
    }
}

#[derive(Clone, Copy)]
enum SeatNav {
    Left,
    Right,
    Up,
    Down,
}

fn move_seat_step(current: PlayerSlot, nav: SeatNav, two_v_two: bool) -> PlayerSlot {
    if !two_v_two {
        return match nav {
            SeatNav::Left => PlayerSlot::LeftBottom,
            SeatNav::Right => PlayerSlot::RightBottom,
            _ => current,
        };
    }
    match (current, nav) {
        (PlayerSlot::LeftTop, SeatNav::Right) => PlayerSlot::RightTop,
        (PlayerSlot::RightTop, SeatNav::Left) => PlayerSlot::LeftTop,
        (PlayerSlot::LeftBottom, SeatNav::Right) => PlayerSlot::RightBottom,
        (PlayerSlot::RightBottom, SeatNav::Left) => PlayerSlot::LeftBottom,
        (PlayerSlot::LeftTop, SeatNav::Down) => PlayerSlot::LeftBottom,
        (PlayerSlot::LeftBottom, SeatNav::Up) => PlayerSlot::LeftTop,
        (PlayerSlot::RightTop, SeatNav::Down) => PlayerSlot::RightBottom,
        (PlayerSlot::RightBottom, SeatNav::Up) => PlayerSlot::RightTop,
        _ => current,
    }
}

/// Step in `nav` direction, skipping any slot locked by *another* player.
/// Bails out after a full loop of all 4 slots so we never spin.
fn move_seat(
    current: PlayerSlot,
    nav: SeatNav,
    two_v_two: bool,
    locked_by_other: [bool; 4],
) -> PlayerSlot {
    let mut next = move_seat_step(current, nav, two_v_two);
    for _ in 0..4 {
        if next == current || !locked_by_other[next.index()] {
            return next;
        }
        let after = move_seat_step(next, nav, two_v_two);
        if after == next {
            // Edge of the grid in this direction; nothing free that way.
            return current;
        }
        next = after;
    }
    current
}

fn first_free_default(locked_by_other: [bool; 4]) -> PlayerSlot {
    for &slot in &[
        PlayerSlot::LeftBottom,
        PlayerSlot::RightBottom,
        PlayerSlot::LeftTop,
        PlayerSlot::RightTop,
    ] {
        if !locked_by_other[slot.index()] {
            return slot;
        }
    }
    PlayerSlot::LeftBottom
}

fn next_visible_slot(start: usize, dir: i32, hidden: &impl Fn(usize) -> bool) -> usize {
    let n = PLAYER_PANEL_SLOTS;
    let mut idx = start;
    for _ in 0..n {
        idx = (idx as i32 + dir).rem_euclid(n as i32) as usize;
        if !hidden(idx) {
            return idx;
        }
    }
    start // all hidden — fall back
}

pub fn gameplay_input_system(
    mut commands: Commands,
    mut next: ResMut<NextState<GameState>>,
    mode: Res<GameMode>,
    models: Res<UnitModels>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut focuses: Query<(Entity, &mut PlayerFocus)>,
    gamepads: Query<&Gamepad>,
    units: Query<(&PlayerSlot, &UnitKind), With<Unit>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    mouse: Res<MouseUi>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut alive = [false; 4];
    for slot in &alive_bases {
        alive[slot.index()] = true;
    }

    // Escape mirrors the gamepad Start as a keyboard pause, so a controller-less
    // debug session (mouse-launched match) can still reach the pause menu.
    let mut pause = keys.just_pressed(KeyCode::Escape);

    for (pad_entity, mut focus) in focuses.iter_mut() {
        let Ok(pad) = gamepads.get(pad_entity) else {
            continue;
        };

        if pad.just_pressed(GamepadButton::Start) {
            pause = true;
            continue;
        }

        // Defeated player: cancel any pending placement and ignore inputs.
        if !alive[focus.slot.index()] {
            if placement.get(focus.slot).is_some() {
                placement.clear(focus.slot);
            }
            continue;
        }

        // While this player is placing a tower, let placement_system claim all inputs
        // (D-pad, South, West). Otherwise re-arming would swallow the confirm press.
        if placement.get(focus.slot).is_some() {
            continue;
        }

        let miner_count = units
            .iter()
            .filter(|(s, k)| **s == focus.slot && **k == UnitKind::Miner)
            .count();
        // Slot indices match the vertical HUD order: 0 Tower, 1 Soldier,
        // 2 Archer, 3 Priest, 4 Miner. Miner slot hides when the cap is reached.
        let slot_hidden = |idx: usize| idx == 4 && miner_count >= MAX_MINERS_PER_PLAYER;
        if slot_hidden(focus.index) {
            focus.index = next_visible_slot(focus.index, 1, &slot_hidden);
        }

        if pad.just_pressed(GamepadButton::DPadUp) || pad.just_pressed(GamepadButton::DPadLeft) {
            focus.index = next_visible_slot(focus.index, -1, &slot_hidden);
        } else if pad.just_pressed(GamepadButton::DPadDown)
            || pad.just_pressed(GamepadButton::DPadRight)
        {
            focus.index = next_visible_slot(focus.index, 1, &slot_hidden);
        }

        if pad.just_pressed(GamepadButton::West) {
            arm_placement(&mut placement, focus.slot, *mode);
        }

        if pad.just_pressed(GamepadButton::South) {
            buy_or_place_slot(
                &mut commands,
                &models,
                &mut gold,
                &mut placement,
                &units,
                focus.slot,
                focus.index,
                *mode,
            );
        }
    }

    // Mouse: left-clicking a HUD panel slot buys/places for that slot directly,
    // independent of any pad focus, honouring the same alive / placement-busy /
    // miner-cap guards as the gamepad path.
    if let Some((slot, index)) = mouse.panel_click
        && mode.active_slots().contains(&slot)
        && alive[slot.index()]
        && placement.get(slot).is_none()
    {
        let miner_count = units
            .iter()
            .filter(|(s, k)| **s == slot && **k == UnitKind::Miner)
            .count();
        let hidden = index == 4 && miner_count >= MAX_MINERS_PER_PLAYER;
        if !hidden {
            buy_or_place_slot(
                &mut commands,
                &models,
                &mut gold,
                &mut placement,
                &units,
                slot,
                index,
                *mode,
            );
        }
    }

    if pause {
        next.set(GameState::Paused);
    }
}

/// Perform the action bound to a HUD panel slot (Tower/Soldier/Archer/Priest/
/// Miner) for `slot`. Shared by the gamepad path (`focus.index` on South) and
/// the mouse path (`MouseUi::panel_click`) so both stay in lockstep.
fn buy_or_place_slot(
    commands: &mut Commands,
    models: &UnitModels,
    gold: &mut Gold,
    placement: &mut PlacementMode,
    units: &Query<(&PlayerSlot, &UnitKind), With<Unit>>,
    slot: PlayerSlot,
    index: usize,
    mode: GameMode,
) {
    let count_of = |kind: UnitKind| {
        units
            .iter()
            .filter(|(s, k)| **s == slot && **k == kind)
            .count()
    };
    match index {
        0 => arm_placement(placement, slot, mode),
        1..=3 => {
            let kind = match index {
                1 => UnitKind::Soldier,
                2 => UnitKind::Archer,
                _ => UnitKind::Priest,
            };
            if gold.try_spend(slot, kind.stats().cost) {
                // Cycle the spawn lane per kind so successive purchases don't
                // stack on the same spot.
                let lane = count_of(kind) % LANE_COUNT;
                spawn_combat_unit(commands, models, slot, mode, kind, lane);
            }
        }
        4 => {
            let miner_count = count_of(UnitKind::Miner);
            if miner_count < MAX_MINERS_PER_PLAYER
                && gold.try_spend(slot, UnitKind::Miner.stats().cost)
            {
                spawn_miner(commands, models, slot, mode, miner_count);
            }
        }
        _ => {}
    }
}

pub fn settings_input_system(
    mut next: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    rt_avail: Res<RaytracingAvailable>,
    preset: Res<GraphicsPreset>,
    mut tab: ResMut<SettingsTab>,
    mut menu_focus: ResMut<MenuFocus>,
    origin: Res<SettingsOrigin>,
    gamepads: Query<&Gamepad>,
    mouse: Res<MouseUi>,
) {
    let slots = slot_count(*tab, &settings);
    if menu_focus.index >= slots {
        menu_focus.index = 0;
    }

    let mut up = false;
    let mut down = false;
    let mut activate = false;
    let mut back = false;
    let mut switch_tab = false;
    for pad in &gamepads {
        if pad.just_pressed(GamepadButton::DPadUp) {
            up = true;
        }
        if pad.just_pressed(GamepadButton::DPadDown) {
            down = true;
        }
        if pad.just_pressed(GamepadButton::South) {
            activate = true;
        }
        if pad.just_pressed(GamepadButton::East) {
            back = true;
        }
        if pad.just_pressed(GamepadButton::LeftTrigger)
            || pad.just_pressed(GamepadButton::RightTrigger)
        {
            switch_tab = true;
        }
    }

    // Mouse-clicking a tab switches straight to it (a no-op if already active).
    if let Some(t) = mouse.tab_click {
        if *tab != t {
            *tab = t;
            menu_focus.index = 0;
        }
        return;
    }
    if switch_tab {
        *tab = tab.toggle();
        menu_focus.index = 0;
        return;
    }

    if up {
        menu_focus.index = (menu_focus.index + slots - 1) % slots;
    }
    if down {
        menu_focus.index = (menu_focus.index + 1) % slots;
    }

    // Mouse: hover moves focus, left-click activates the hovered row.
    if let Some(i) = mouse.menu_hover.filter(|i| *i < slots) {
        menu_focus.index = i;
    }
    if let Some(i) = mouse.menu_click.filter(|i| *i < slots) {
        menu_focus.index = i;
        activate = true;
    }

    if back {
        next.set(origin.to_state());
        return;
    }

    if !activate {
        return;
    }
    let slot = tab_slots(*tab, &settings).get(menu_focus.index).copied();
    match slot {
        Some(MenuSlot::Preset) => {
            let next = preset.cycle();
            next.apply(&mut settings, dlss_avail.0, rt_avail.0);
        }
        Some(MenuSlot::Param(id)) => match id {
            ParamId::Fullscreen => settings.fullscreen = !settings.fullscreen,
            ParamId::VSync => settings.vsync = !settings.vsync,
            ParamId::Msaa => {
                settings.msaa = match settings.msaa {
                    0 => 2,
                    2 => 4,
                    4 => 8,
                    _ => 0,
                };
            }
            ParamId::Hdr => settings.hdr = !settings.hdr,
            ParamId::Exposure => settings.exposure = (settings.exposure + 1) % 3,
            ParamId::Tonemapping => settings.tonemapping = (settings.tonemapping + 1) % 4,
            ParamId::FpsCap => settings.fps_cap = (settings.fps_cap + 1) % 6,
            ParamId::Colorblind => settings.colorblind = !settings.colorblind,
            ParamId::Raytracing => {
                if cfg!(feature = "raytracing") && rt_avail.0 {
                    settings.raytracing = !settings.raytracing;
                }
            }
            ParamId::Dlss => {
                if cfg!(feature = "dlss") && dlss_avail.0 {
                    settings.dlss = !settings.dlss;
                }
            }
            ParamId::DlssQuality => settings.dlss_quality = (settings.dlss_quality + 1) % 5,
            ParamId::Taa => settings.taa = !settings.taa,
            ParamId::Fxaa => settings.fxaa = !settings.fxaa,
            ParamId::Bloom => settings.bloom = !settings.bloom,
            ParamId::BloomIntensity => {
                settings.bloom_intensity = (settings.bloom_intensity + 1) % 3;
            }
            ParamId::Atmosphere => settings.atmosphere = !settings.atmosphere,
            ParamId::VolumetricFog => settings.volumetric_fog = !settings.volumetric_fog,
            ParamId::FogDensity => settings.fog_density = (settings.fog_density + 1) % 3,
            ParamId::DistanceFog => settings.distance_fog = !settings.distance_fog,
            ParamId::Ssao => settings.ssao = !settings.ssao,
            ParamId::SsaoQuality => settings.ssao_quality = (settings.ssao_quality + 1) % 4,
            ParamId::Shadows => settings.shadows = !settings.shadows,
            ParamId::MotionBlur => settings.motion_blur = !settings.motion_blur,
        },
        Some(MenuSlot::Back) | None => next.set(origin.to_state()),
    }
}

#[cfg(test)]
mod seat_tests {
    use super::*;

    #[test]
    fn move_seat_step_1v1_picks_side() {
        assert_eq!(
            move_seat_step(PlayerSlot::LeftBottom, SeatNav::Right, false),
            PlayerSlot::RightBottom
        );
        assert_eq!(
            move_seat_step(PlayerSlot::RightBottom, SeatNav::Left, false),
            PlayerSlot::LeftBottom
        );
        // Up/Down are no-ops in 1v1.
        assert_eq!(
            move_seat_step(PlayerSlot::LeftBottom, SeatNav::Up, false),
            PlayerSlot::LeftBottom
        );
    }

    #[test]
    fn move_seat_step_2v2_navigates_grid() {
        assert_eq!(
            move_seat_step(PlayerSlot::LeftBottom, SeatNav::Up, true),
            PlayerSlot::LeftTop
        );
        assert_eq!(
            move_seat_step(PlayerSlot::RightTop, SeatNav::Down, true),
            PlayerSlot::RightBottom
        );
        // No wrap-around on edges.
        assert_eq!(
            move_seat_step(PlayerSlot::LeftTop, SeatNav::Up, true),
            PlayerSlot::LeftTop
        );
    }

    #[test]
    fn move_seat_skips_locked_neighbour() {
        // 2v2: LeftBottom moves right, but RightBottom is taken — should land
        // on… current (nothing free that way after one step).
        let mut locked = [false; 4];
        locked[PlayerSlot::RightBottom.index()] = true;
        assert_eq!(
            move_seat(PlayerSlot::LeftBottom, SeatNav::Right, true, locked),
            PlayerSlot::LeftBottom
        );
    }
}
