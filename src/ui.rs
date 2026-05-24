use bevy::prelude::*;
use bevy::window::WindowMode;

use crate::common::*;
use crate::towers::{is_valid_tower_zone, spawn_tower};
use crate::units::{spawn_archer, spawn_miner, spawn_soldier};

#[derive(Component, Clone, Copy)]
pub struct PanelSlot {
    pub side: Side,
    pub index: usize,
}

#[derive(Component)]
pub struct GoldText(pub Side);

#[derive(Component)]
pub struct BaseHpText(pub Side);

#[derive(Component)]
pub struct MenuOverlay;

#[derive(Component)]
pub struct EndgameOverlay;

#[derive(Component)]
pub struct SettingsOverlay;

#[derive(Component)]
pub struct PauseOverlay;

#[derive(Component)]
pub struct FullscreenToggleText;

#[derive(Component)]
pub struct SideSelectOverlay;

#[derive(Component, Clone, Copy)]
pub struct SideCard(pub Side);

#[derive(Component, Clone, Copy)]
pub struct SideCardStatus(pub Side);

#[derive(Component, Clone, Copy)]
pub struct MenuButton(pub usize);

const BTN_NORMAL: Color = Color::srgb(0.16, 0.16, 0.20);
const BTN_FOCUSED: Color = Color::srgb(0.32, 0.32, 0.40);
const CARD_NORMAL: Color = Color::srgb(0.12, 0.13, 0.18);
const CARD_HOVERED: Color = Color::srgb(0.22, 0.23, 0.30);

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(Val::Px(24.0)),
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Left Base: 20/20"),
                TextFont::from_font_size(22.0),
                TextColor(Side::Left.color()),
                BaseHpText(Side::Left),
            ));
            parent.spawn((
                Text::new("Right Base: 20/20"),
                TextFont::from_font_size(22.0),
                TextColor(Side::Right.color()),
                BaseHpText(Side::Right),
            ));
        });

    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(18.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(Val::Px(24.0)),
            ..default()
        },))
        .with_children(|parent| {
            spawn_player_panel(parent, Side::Left);
            spawn_player_panel(parent, Side::Right);
        });
}

fn spawn_player_panel(parent: &mut ChildSpawnerCommands, side: Side) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },))
        .with_children(|panel| {
            spawn_slot(panel, side, 0, &format!("Soldier ({}g)", SOLDIER_COST));
            spawn_slot(panel, side, 1, &format!("Miner ({}g)", MINER_COST));
            spawn_slot(panel, side, 2, &format!("Archer ({}g)", ARCHER_COST));
            spawn_slot(panel, side, 3, &format!("Tower ({}g)", TOWER_COST));
            panel.spawn((
                Text::new("Gold: 10"),
                TextFont::from_font_size(18.0),
                TextColor(side.color()),
                GoldText(side),
            ));
        });
}

fn spawn_slot(panel: &mut ChildSpawnerCommands, side: Side, index: usize, label: &str) {
    panel
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(side.color()),
            PanelSlot { side, index },
        ))
        .with_child((
            Text::new(label),
            TextFont::from_font_size(15.0),
            TextColor(Color::WHITE),
        ));
}

pub fn update_gold_text(gold: Res<Gold>, mut texts: Query<(&GoldText, &mut Text)>) {
    if !gold.is_changed() {
        return;
    }
    for (slot, mut text) in &mut texts {
        text.0 = format!("Gold: {}", gold.get(slot.0));
    }
}

pub fn update_base_hp_text(
    bases: Query<(&Side, &Health), With<Base>>,
    mut texts: Query<(&BaseHpText, &mut Text)>,
) {
    for (slot, mut text) in &mut texts {
        if let Some((_, hp)) = bases.iter().find(|(s, _)| **s == slot.0) {
            text.0 = format!("{} Base: {}/{}", slot.0.label(), hp.current.max(0), hp.max);
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Overlays
// ────────────────────────────────────────────────────────────────────────────

pub fn update_menu_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<MenuOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if *state != GameState::Menu {
        return;
    }
    menu_focus.index = 0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.06, 0.10)),
            MenuOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("AttackTower"),
                TextFont::from_font_size(72.0),
                TextColor(Color::WHITE),
            ));
            spawn_menu_button(parent, 0, "Jouer", Side::Left.color());
            spawn_menu_button(parent, 1, "Parametres", Color::srgb(0.7, 0.7, 0.75));
            spawn_menu_button(parent, 2, "Quitter", Side::Right.color());
            parent.spawn((
                Text::new("D-pad : naviguer   A : selectionner"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });
}

pub fn update_settings_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    settings: Res<GameSettings>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<SettingsOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if *state != GameState::Settings {
        return;
    }
    menu_focus.index = 0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.06, 0.10)),
            SettingsOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Parametres"),
                TextFont::from_font_size(56.0),
                TextColor(Color::WHITE),
            ));
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(36.0), Val::Px(14.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(360.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(BTN_NORMAL),
                    BorderColor::all(Color::srgb(0.7, 0.7, 0.75)),
                    MenuButton(0),
                ))
                .with_child((
                    Text::new(fullscreen_label(settings.fullscreen)),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::WHITE),
                    FullscreenToggleText,
                ));
            spawn_menu_button(parent, 1, "Retour", Color::WHITE);
            parent.spawn((
                Text::new("D-pad : naviguer   A : modifier / valider   B : retour"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.75, 0.75, 0.8)),
            ));
        });
}

fn fullscreen_label(on: bool) -> String {
    format!("Plein ecran : {}", if on { "OUI" } else { "NON" })
}

pub fn update_pause_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<PauseOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if *state != GameState::Paused {
        return;
    }
    menu_focus.index = 0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            PauseOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Pause"),
                TextFont::from_font_size(56.0),
                TextColor(Color::WHITE),
            ));
            spawn_menu_button(parent, 0, "Reprendre", Side::Left.color());
            spawn_menu_button(parent, 1, "Parametres", Color::srgb(0.7, 0.7, 0.75));
            spawn_menu_button(parent, 2, "Menu principal", Side::Right.color());
            parent.spawn((
                Text::new("D-pad : naviguer   A : selectionner   Start/B : reprendre"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
            ));
        });
}

pub fn pause_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut menu_focus: ResMut<MenuFocus>,
    mut origin: ResMut<SettingsOrigin>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut players: ResMut<PlayerControllers>,
    mut bases: Query<&mut Health, With<Base>>,
    units: Query<Entity, With<Unit>>,
    arrows: Query<Entity, With<Arrow>>,
    towers: Query<Entity, With<Tower>>,
    ghosts: Query<Entity, With<TowerGhost>>,
    gamepads: Query<&Gamepad>,
) {
    if *state != GameState::Paused {
        return;
    }
    if state.is_changed() {
        return;
    }

    const SLOTS: usize = 3;
    if menu_focus.index >= SLOTS {
        menu_focus.index = 0;
    }

    let mut up = false;
    let mut down = false;
    let mut activate = false;
    let mut resume = false;
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

    if resume {
        *state = GameState::Playing;
        return;
    }

    if activate {
        match menu_focus.index {
            0 => *state = GameState::Playing,
            1 => {
                *origin = SettingsOrigin::Paused;
                *state = GameState::Settings;
            }
            2 => {
                for e in &units {
                    commands.entity(e).despawn();
                }
                for e in &arrows {
                    commands.entity(e).despawn();
                }
                for e in &towers {
                    commands.entity(e).despawn();
                }
                for e in &ghosts {
                    commands.entity(e).despawn();
                }
                for mut hp in bases.iter_mut() {
                    hp.current = hp.max;
                }
                *gold = Gold::default();
                *placement = PlacementMode::default();
                *players = PlayerControllers::default();
                *state = GameState::Menu;
            }
            _ => {}
        }
    }
}

pub fn update_fullscreen_toggle_text(
    settings: Res<GameSettings>,
    mut texts: Query<&mut Text, With<FullscreenToggleText>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut text in &mut texts {
        text.0 = fullscreen_label(settings.fullscreen);
    }
}

pub fn update_endgame_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<EndgameOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if let GameState::Ended(winner) = *state {
        menu_focus.index = 0;
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                EndgameOverlay,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("Player {} wins", winner.label())),
                    TextFont::from_font_size(54.0),
                    TextColor(winner.color()),
                ));
                spawn_menu_button(parent, 0, "Restart", Color::WHITE);
                parent.spawn((
                    Text::new("A : retour menu"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.8, 0.8, 0.85)),
                ));
            });
    }
}

pub fn update_sideselect_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    overlay: Query<Entity, With<SideSelectOverlay>>,
    seats: Query<Entity, With<SeatSelection>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    // Clear any leftover seat selections when entering/leaving SideSelect.
    for entity in &seats {
        commands.entity(entity).remove::<SeatSelection>();
    }
    if *state != GameState::SideSelect {
        return;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(28.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.06, 0.10)),
            SideSelectOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Choisir un cote"),
                TextFont::from_font_size(48.0),
                TextColor(Color::WHITE),
            ));
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(48.0),
                    ..default()
                },))
                .with_children(|row| {
                    spawn_side_card(row, Side::Left);
                    spawn_side_card(row, Side::Right);
                });
            parent.spawn((
                Text::new(
                    "D-pad gauche/droite : choisir   A : confirmer   B : annuler   Start : demarrer",
                ),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.75, 0.75, 0.8)),
            ));
        });
}

fn spawn_side_card(parent: &mut ChildSpawnerCommands, side: Side) {
    parent
        .spawn((
            Node {
                width: Val::Px(260.0),
                height: Val::Px(180.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            BackgroundColor(CARD_NORMAL),
            BorderColor::all(side.color()),
            SideCard(side),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(match side {
                    Side::Left => "Joueur Gauche",
                    Side::Right => "Joueur Droit",
                }),
                TextFont::from_font_size(26.0),
                TextColor(side.color()),
            ));
            card.spawn((
                Text::new("Libre"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
                SideCardStatus(side),
            ));
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    label: &str,
    border: Color,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(36.0), Val::Px(14.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(220.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(border),
            MenuButton(index),
        ))
        .with_child((
            Text::new(label),
            TextFont::from_font_size(26.0),
            TextColor(Color::WHITE),
        ));
}

// ────────────────────────────────────────────────────────────────────────────
// Focus visuals (refresh each frame from focus resources/components)
// ────────────────────────────────────────────────────────────────────────────

pub fn apply_menu_focus_visual(
    state: Res<GameState>,
    focus: Res<MenuFocus>,
    mut buttons: Query<(&MenuButton, &mut BackgroundColor)>,
) {
    let active = matches!(
        *state,
        GameState::Menu | GameState::Settings | GameState::Paused | GameState::Ended(_)
    );
    for (btn, mut bg) in &mut buttons {
        bg.0 = if active && btn.0 == focus.index {
            BTN_FOCUSED
        } else {
            BTN_NORMAL
        };
    }
}

pub fn apply_player_focus_visual(
    state: Res<GameState>,
    focuses: Query<&PlayerFocus>,
    mut slots: Query<(&PanelSlot, &mut BackgroundColor, &mut BorderColor)>,
) {
    let active = matches!(*state, GameState::Playing | GameState::Paused);
    for (slot, mut bg, mut border) in &mut slots {
        let focused = active && focuses.iter().any(|f| f.side == slot.side && f.index == slot.index);
        bg.0 = if focused { BTN_FOCUSED } else { BTN_NORMAL };
        *border = BorderColor::all(if focused { Color::WHITE } else { slot.side.color() });
    }
}

pub fn update_sideselect_cards(
    state: Res<GameState>,
    seats: Query<&SeatSelection>,
    mut texts: Query<(&SideCardStatus, &mut Text, &mut TextColor)>,
    mut cards: Query<(&SideCard, &mut BackgroundColor, &mut BorderColor)>,
) {
    if *state != GameState::SideSelect {
        return;
    }
    for (slot, mut text, mut color) in &mut texts {
        let confirmed = seats
            .iter()
            .any(|s| s.confirmed && s.hovered == slot.0);
        let hovered = seats
            .iter()
            .filter(|s| !s.confirmed && s.hovered == slot.0)
            .count();
        if confirmed {
            text.0 = "Verrouille".to_string();
            color.0 = slot.0.color();
        } else if hovered > 0 {
            text.0 = format!("Selectionne ({})", hovered);
            color.0 = Color::WHITE;
        } else {
            text.0 = "Libre".to_string();
            color.0 = Color::srgb(0.7, 0.7, 0.75);
        }
    }
    for (card, mut bg, mut border) in &mut cards {
        let confirmed = seats.iter().any(|s| s.confirmed && s.hovered == card.0);
        let hovered = seats.iter().any(|s| !s.confirmed && s.hovered == card.0);
        bg.0 = if confirmed || hovered {
            CARD_HOVERED
        } else {
            CARD_NORMAL
        };
        let border_color = if confirmed {
            Color::WHITE
        } else {
            card.0.color()
        };
        *border = BorderColor::all(border_color);
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Lifecycle helpers (run on state change)
// ────────────────────────────────────────────────────────────────────────────

pub fn manage_input_components(
    mut commands: Commands,
    state: Res<GameState>,
    players: Res<PlayerControllers>,
    gamepads: Query<&Gamepad>,
    focuses: Query<Entity, With<PlayerFocus>>,
) {
    if !state.is_changed() {
        return;
    }
    let active = matches!(*state, GameState::Playing | GameState::Paused);
    if !active {
        for entity in &focuses {
            commands.entity(entity).remove::<PlayerFocus>();
        }
        return;
    }
    // Keep focus if already set (e.g. Pause→Playing).
    if focuses.iter().next().is_some() {
        return;
    }
    for (side, opt) in [(Side::Left, players.left), (Side::Right, players.right)] {
        if let Some(pad) = opt {
            if gamepads.get(pad).is_ok() {
                commands
                    .entity(pad)
                    .insert(PlayerFocus { side, index: 0 });
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Input systems (gamepad-only)
// ────────────────────────────────────────────────────────────────────────────

pub fn menu_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut menu_focus: ResMut<MenuFocus>,
    mut origin: ResMut<SettingsOrigin>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut players: ResMut<PlayerControllers>,
    mut bases: Query<&mut Health, With<Base>>,
    units: Query<Entity, With<Unit>>,
    arrows: Query<Entity, With<Arrow>>,
    towers: Query<Entity, With<Tower>>,
    ghosts: Query<Entity, With<TowerGhost>>,
    mut exit: MessageWriter<AppExit>,
    gamepads: Query<&Gamepad>,
) {
    let in_menu = *state == GameState::Menu;
    let in_endgame = matches!(*state, GameState::Ended(_));
    if !in_menu && !in_endgame {
        return;
    }
    if state.is_changed() {
        return;
    }

    let slot_count = if in_menu { 3 } else { 1 };

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

    if menu_focus.index >= slot_count {
        menu_focus.index = 0;
    }
    if up {
        menu_focus.index = (menu_focus.index + slot_count - 1) % slot_count;
    }
    if down {
        menu_focus.index = (menu_focus.index + 1) % slot_count;
    }

    if !activate {
        return;
    }

    if in_menu {
        match menu_focus.index {
            0 => {
                if pad_count > 0 {
                    *state = GameState::SideSelect;
                }
            }
            1 => {
                *origin = SettingsOrigin::Menu;
                *state = GameState::Settings;
            }
            2 => {
                exit.write(AppExit::Success);
            }
            _ => {}
        }
    } else if in_endgame {
        for e in &units {
            commands.entity(e).despawn();
        }
        for e in &arrows {
            commands.entity(e).despawn();
        }
        for e in &towers {
            commands.entity(e).despawn();
        }
        for e in &ghosts {
            commands.entity(e).despawn();
        }
        for mut hp in bases.iter_mut() {
            hp.current = hp.max;
        }
        *gold = Gold::default();
        *placement = PlacementMode::default();
        *players = PlayerControllers::default();
        *state = GameState::Menu;
    }
}

pub fn sideselect_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut players: ResMut<PlayerControllers>,
    mut seats: Query<(Entity, &Gamepad, Option<&mut SeatSelection>)>,
) {
    if *state != GameState::SideSelect {
        return;
    }
    if state.is_changed() {
        return;
    }

    // Snapshot existing confirmations so we can reject same-frame conflicts.
    let confirmations: Vec<(Entity, Side, bool)> = seats
        .iter()
        .map(|(e, _, s)| {
            s.as_ref()
                .map_or((e, Side::Left, false), |s| (e, s.hovered, s.confirmed))
        })
        .collect();
    let confirmed_left = confirmations
        .iter()
        .find(|(_, h, c)| *c && *h == Side::Left)
        .map(|x| x.0);
    let confirmed_right = confirmations
        .iter()
        .find(|(_, h, c)| *c && *h == Side::Right)
        .map(|x| x.0);

    let mut start_pressed = false;

    for (pad_entity, pad, seat_opt) in seats.iter_mut() {
        if pad.just_pressed(GamepadButton::Start) {
            start_pressed = true;
        }

        match seat_opt {
            None => {
                // Lazily create a seat on first input from this gamepad.
                if pad.just_pressed(GamepadButton::DPadLeft)
                    || pad.just_pressed(GamepadButton::DPadRight)
                    || pad.just_pressed(GamepadButton::South)
                {
                    let hovered = if pad.just_pressed(GamepadButton::DPadRight) {
                        Side::Right
                    } else {
                        Side::Left
                    };
                    commands.entity(pad_entity).insert(SeatSelection {
                        hovered,
                        confirmed: false,
                    });
                }
            }
            Some(mut seat) => {
                if seat.confirmed {
                    if pad.just_pressed(GamepadButton::East) {
                        seat.confirmed = false;
                    }
                    continue;
                }
                if pad.just_pressed(GamepadButton::DPadLeft) {
                    seat.hovered = Side::Left;
                }
                if pad.just_pressed(GamepadButton::DPadRight) {
                    seat.hovered = Side::Right;
                }
                if pad.just_pressed(GamepadButton::South) {
                    let already_taken = match seat.hovered {
                        Side::Left => confirmed_left.is_some_and(|e| e != pad_entity),
                        Side::Right => confirmed_right.is_some_and(|e| e != pad_entity),
                    };
                    if !already_taken {
                        seat.confirmed = true;
                    }
                }
            }
        }
    }

    if start_pressed && (confirmed_left.is_some() || confirmed_right.is_some()) {
        players.left = confirmed_left;
        players.right = confirmed_right;
        *state = GameState::Playing;
    }
}

pub fn gameplay_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    lib: Res<MatLibrary>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut focuses: Query<(Entity, &mut PlayerFocus)>,
    gamepads: Query<&Gamepad>,
) {
    if *state != GameState::Playing {
        return;
    }
    if state.is_changed() {
        return;
    }

    let mut pause = false;

    for (pad_entity, mut focus) in focuses.iter_mut() {
        let Ok(pad) = gamepads.get(pad_entity) else {
            continue;
        };

        if pad.just_pressed(GamepadButton::Start) {
            pause = true;
            continue;
        }

        if pad.just_pressed(GamepadButton::DPadLeft) {
            focus.index = (focus.index + PLAYER_PANEL_SLOTS - 1) % PLAYER_PANEL_SLOTS;
        } else if pad.just_pressed(GamepadButton::DPadRight) {
            focus.index = (focus.index + 1) % PLAYER_PANEL_SLOTS;
        }

        if pad.just_pressed(GamepadButton::West) {
            arm_placement(&mut placement, focus.side);
        }

        if pad.just_pressed(GamepadButton::South) {
            match focus.index {
                0 => {
                    if gold.try_spend(focus.side, SOLDIER_COST) {
                        spawn_soldier(&mut commands, &lib, focus.side);
                    }
                }
                1 => {
                    if gold.try_spend(focus.side, MINER_COST) {
                        spawn_miner(&mut commands, &lib, focus.side);
                    }
                }
                2 => {
                    if gold.try_spend(focus.side, ARCHER_COST) {
                        spawn_archer(&mut commands, &lib, focus.side);
                    }
                }
                3 => arm_placement(&mut placement, focus.side),
                _ => {}
            }
        }
    }

    if pause {
        *state = GameState::Paused;
    }
}

pub fn placement_system(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    lib: Res<MatLibrary>,
    mut placement: ResMut<PlacementMode>,
    mut gold: ResMut<Gold>,
    players: Res<PlayerControllers>,
    gamepads: Query<&Gamepad>,
    ghosts: Query<(Entity, &Side), With<TowerGhost>>,
) {
    // While paused, keep the ghosts visible in place but skip input handling.
    if *state == GameState::Paused {
        return;
    }
    // Outside Playing: wipe everything.
    if *state != GameState::Playing {
        *placement = PlacementMode::default();
        for (e, _) in &ghosts {
            commands.entity(e).despawn();
        }
        return;
    }

    for side in [Side::Left, Side::Right] {
        // Despawn any existing ghost for this side (will respawn below if still placing).
        for (e, ghost_side) in &ghosts {
            if *ghost_side == side {
                commands.entity(e).despawn();
            }
        }

        let Some(seat) = placement.get(side) else {
            continue;
        };
        let Some(pad_entity) = players.get(side) else {
            placement.clear(side);
            continue;
        };
        let Ok(pad) = gamepads.get(pad_entity) else {
            placement.clear(side);
            continue;
        };

        // Swallow the press that activated placement.
        if !seat.armed {
            placement.set(
                side,
                PlacementSeat {
                    world_pos: seat.world_pos,
                    armed: true,
                },
            );
            continue;
        }

        if pad.just_pressed(GamepadButton::East) {
            placement.clear(side);
            continue;
        }

        // Move virtual cursor by left stick.
        let stick = pad.left_stick();
        let dt = time.delta_secs();
        let dx = if stick.x.abs() > GAMEPAD_STICK_DEADZONE {
            stick.x
        } else {
            0.0
        };
        let dz = if stick.y.abs() > GAMEPAD_STICK_DEADZONE {
            stick.y
        } else {
            0.0
        };
        let mut pos = seat.world_pos;
        pos.x += dx * GAMEPAD_CURSOR_SPEED * dt;
        // Stick Y positive = up on screen → -Z in world (closer to camera-far).
        pos.z -= dz * GAMEPAD_CURSOR_SPEED * dt;

        let in_zone = is_valid_tower_zone(side, pos);
        let can_afford = gold.get(side) >= TOWER_COST;
        let valid = in_zone && can_afford;

        // Place on A.
        if pad.just_pressed(GamepadButton::South) && valid && gold.try_spend(side, TOWER_COST) {
            spawn_tower(
                &mut commands,
                &lib,
                side,
                Vec3::new(pos.x, 0.0, pos.z),
            );
            placement.clear(side);
            continue;
        }

        // Update seat and spawn ghost.
        placement.set(
            side,
            PlacementSeat {
                world_pos: pos,
                armed: true,
            },
        );
        let mat = if valid {
            lib.ghost_valid_mat.clone()
        } else {
            lib.ghost_invalid_mat.clone()
        };
        commands.spawn((
            Mesh3d(lib.tower_ghost_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(pos.x, TOWER_HEIGHT * 0.5, pos.z),
            TowerGhost,
            side,
        ));
    }
}

fn arm_placement(placement: &mut PlacementMode, side: Side) {
    placement.set(
        side,
        PlacementSeat {
            world_pos: default_placement_pos(side),
            armed: false,
        },
    );
}

fn default_placement_pos(side: Side) -> Vec3 {
    let x = match side {
        Side::Left => (LEFT_BASE_X + TOWER_PLACEMENT_MARGIN - ZONE_BOUNDARY) * 0.5,
        Side::Right => (ZONE_BOUNDARY + RIGHT_BASE_X - TOWER_PLACEMENT_MARGIN) * 0.5,
    };
    Vec3::new(x, 0.0, 0.0)
}

pub fn settings_input_system(
    mut state: ResMut<GameState>,
    mut settings: ResMut<GameSettings>,
    mut menu_focus: ResMut<MenuFocus>,
    origin: Res<SettingsOrigin>,
    gamepads: Query<&Gamepad>,
) {
    if *state != GameState::Settings {
        return;
    }
    if state.is_changed() {
        return;
    }

    const SLOTS: usize = 2;
    if menu_focus.index >= SLOTS {
        menu_focus.index = 0;
    }

    let mut up = false;
    let mut down = false;
    let mut activate = false;
    let mut back = false;
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
    }

    if up {
        menu_focus.index = (menu_focus.index + SLOTS - 1) % SLOTS;
    }
    if down {
        menu_focus.index = (menu_focus.index + 1) % SLOTS;
    }

    if back {
        *state = origin.to_state();
        return;
    }

    if activate {
        match menu_focus.index {
            0 => settings.fullscreen = !settings.fullscreen,
            1 => *state = origin.to_state(),
            _ => {}
        }
    }
}

pub fn apply_window_settings(
    settings: Res<GameSettings>,
    mut windows: Query<&mut Window>,
) {
    if !settings.is_changed() {
        return;
    }
    let mode = if settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    for mut window in &mut windows {
        if window.mode != mode {
            window.mode = mode;
        }
    }
}
