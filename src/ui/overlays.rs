//! State overlays (menu, settings, pause, side-select, endgame): their
//! marker components, the OnEnter spawners, the in-state refreshers and the
//! focus highlight for menu buttons. Torn down by despawn_all on OnExit.

use bevy::prelude::*;

use crate::graphics::{
    DescriptionKind, GraphicsPreset, Impact, MenuSlot, ParamDescription, ParamId, description_for,
    param_label, tab_slots,
};

// `super::*` also surfaces mod.rs's own imports (notably `crate::common::*`),
// hence no explicit common import here.
use super::*;

#[derive(Component)]
pub struct MenuOverlay;

#[derive(Component)]
pub struct EndgameOverlay;

#[derive(Component)]
pub struct SettingsOverlay;

#[derive(Component)]
pub struct PauseOverlay;

/// Marker on the scrollable column that lists the settings parameters. Used by
/// [`scroll_focused_into_view`] to find the column and update its
/// [`ScrollPosition`] when the focused row would fall outside the viewport.
#[derive(Component)]
pub struct SettingsMenuColumn;

#[derive(Component, Clone, Copy)]
pub struct SettingsToggleText(pub ParamId);

#[derive(Component)]
pub struct PresetText;

/// Marker on a tab toggle in the settings overlay. The overlay is rebuilt
/// when the active tab changes, so the highlight stays implicit (colours are
/// set at spawn time). Carries the tab it selects so a mouse click can target
/// it directly (see [`read_mouse_ui`]).
#[derive(Component, Clone, Copy)]
pub struct SettingsTabButton(pub SettingsTab);

/// Marker for one of the four impact rows in the description card. Carries
/// the channel name so the value can be re-colored on focus change without
/// duplicate components.
#[derive(Component, Clone, Copy)]
pub enum DescField {
    Title,
    Functional,
    Technical,
    ImpactHeading,
    ImpactCpu,
    ImpactGpu,
    ImpactRam,
    ImpactVram,
}

/// Marker on the heading + each row Node of the impact section. Their
/// `Node.display` is toggled together when the focused slot has no impacts
/// (Preset selector, Back button) so the labels disappear entirely.
#[derive(Component)]
pub struct ImpactRowNode;

#[derive(Component)]
pub struct SideSelectOverlay;

#[derive(Component, Clone, Copy)]
pub struct SideCard(pub PlayerSlot);

/// Which text line of a SideSelect card this entity is. One component type for
/// all three lines so `update_sideselect_cards` can drive them from a single
/// `Query<&mut Text>` without `&mut Text` aliasing across markers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CardLine {
    Controller,
    Status,
    Nation,
}

#[derive(Component, Clone, Copy)]
pub struct SideCardLine {
    pub slot: PlayerSlot,
    pub line: CardLine,
}

#[derive(Component, Clone, Copy)]
pub struct MenuButton(pub usize);

// ────────────────────────────────────────────────────────────────────────────
// Overlays
// ────────────────────────────────────────────────────────────────────────────

/// OnEnter(Menu): build the main menu (torn down by `despawn_all` on exit).
pub fn spawn_menu_overlay(mut commands: Commands, mut menu_focus: ResMut<MenuFocus>) {
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
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.65)),
            MenuOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("AttackTower"),
                TextFont::from_font_size(56.0),
                TextColor(Color::WHITE),
            ));
            spawn_menu_button(parent, 0, "Play 1v1", Side::Left.color());
            spawn_menu_button(parent, 1, "Play 2v2", Side::Left.color());
            spawn_menu_button(parent, 2, "Settings", Color::srgb(0.7, 0.7, 0.75));
            spawn_menu_button(parent, 3, "Quit", Side::Right.color());
            parent.spawn((
                Text::new("D-pad: navigate   A: select"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });
}

/// Runs only in Settings (run condition); teardown on exit is `despawn_all`.
/// Builds on entry (`State` change) and rebuilds in place on tab change or a
/// STRUCTURAL settings change (a sub-parameter row appearing/disappearing).
/// Plain toggles are already reflected in place by
/// `update_settings_toggle_texts` / `update_settings_description`, so tearing
/// the whole tree down for them would be pure churn.
pub fn update_settings_overlay(
    mut commands: Commands,
    state: Res<State<GameState>>,
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    rt_avail: Res<RaytracingAvailable>,
    preset: Res<GraphicsPreset>,
    tab: Res<SettingsTab>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<SettingsOverlay>>,
    // Slot list of the last build, compared for the structural trigger.
    mut last_slots: Local<Vec<MenuSlot>>,
) {
    let slots = tab_slots(*tab, &settings);
    let rebuild = state.is_changed() || tab.is_changed() || *last_slots != slots;
    if !rebuild {
        return;
    }
    *last_slots = slots;
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    // Reset focus only when entering Settings or switching tab. Structural
    // rebuilds (sub-parameter toggles) keep focus where the user just acted.
    if state.is_changed() || tab.is_changed() {
        menu_focus.index = 0;
    }
    let slots_after = last_slots.len();
    if menu_focus.index >= slots_after {
        menu_focus.index = slots_after.saturating_sub(1);
    }
    let preset = *preset;
    let tab = *tab;
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
                row_gap: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(18.0)),
                ..default()
            },
            // Translucent so the user can see live changes behind the menu.
            BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.65)),
            SettingsOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont::from_font_size(32.0),
                TextColor(Color::WHITE),
            ));

            spawn_tab_selector(parent, tab);

            // Two-column row: menu on the left, description card on the right.
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: Val::Px(36.0),
                    ..default()
                },))
                .with_children(|row| {
                    spawn_settings_menu_column(
                        row,
                        tab,
                        &settings,
                        dlss_avail.0,
                        rt_avail.0,
                        preset,
                    );
                    spawn_description_card(row, tab, preset, &settings);
                });

            parent.spawn((
                Text::new("D-pad: navigate   A: toggle/confirm   B: back   LB/RB: switch tab"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.75, 0.75, 0.8)),
            ));
        });
}

fn spawn_tab_selector(parent: &mut ChildSpawnerCommands, active: SettingsTab) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        },))
        .with_children(|row| {
            for tab in [SettingsTab::Video, SettingsTab::Graphics] {
                let selected = tab == active;
                row.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(6.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(140.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(if selected { BTN_FOCUSED } else { BTN_NORMAL }),
                    BorderColor::all(if selected {
                        Color::WHITE
                    } else {
                        Color::srgb(0.45, 0.46, 0.55)
                    }),
                    SettingsTabButton(tab),
                    Button,
                ))
                .with_child((
                    Text::new(tab.label()),
                    TextFont::from_font_size(18.0),
                    TextColor(if selected {
                        Color::WHITE
                    } else {
                        Color::srgb(0.78, 0.80, 0.86)
                    }),
                ));
            }
        });
}

fn spawn_settings_menu_column(
    row: &mut ChildSpawnerCommands,
    tab: SettingsTab,
    settings: &GameSettings,
    dlss_supported: bool,
    rt_supported: bool,
    preset: GraphicsPreset,
) {
    row.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(4.0),
            min_width: Val::Px(380.0),
            max_height: Val::Vh(72.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        ScrollPosition::default(),
        SettingsMenuColumn,
    ))
    .with_children(|col| {
        for (i, slot) in tab_slots(tab, settings).iter().enumerate() {
            match slot {
                MenuSlot::Preset => spawn_preset_button(col, i, preset),
                MenuSlot::Param(id) => {
                    let label = param_label(*id, settings, dlss_supported, rt_supported);
                    spawn_toggle_button(col, i, label, SettingsToggleText(*id));
                }
                MenuSlot::Back => spawn_menu_button(col, i, "Back", Color::WHITE),
            }
        }
    });
}

fn spawn_preset_button(parent: &mut ChildSpawnerCommands, index: usize, preset: GraphicsPreset) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(Color::srgb(0.85, 0.78, 0.30)),
            MenuButton(index),
            Button,
        ))
        .with_child((
            Text::new(format!("Preset: {}", preset.label())),
            TextFont::from_font_size(20.0),
            TextColor(Color::srgb(0.95, 0.90, 0.55)),
            PresetText,
        ));
}

fn spawn_description_card(
    row: &mut ChildSpawnerCommands,
    tab: SettingsTab,
    preset: GraphicsPreset,
    settings: &GameSettings,
) {
    let card_bg = Color::srgba(0.10, 0.11, 0.15, 0.90);
    let card_border = Color::srgb(0.32, 0.34, 0.42);
    row.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(8.0),
            width: Val::Px(420.0),
            min_height: Val::Px(340.0),
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(card_bg),
        BorderColor::all(card_border),
    ))
    .with_children(|card| {
        let (title, functional, technical, impacts) = describe_for_layout(tab, 0, preset, settings);

        card.spawn((
            Text::new(title),
            TextFont::from_font_size(20.0),
            TextColor(Color::srgb(0.95, 0.95, 0.98)),
            DescField::Title,
        ));
        card.spawn((
            Text::new(functional),
            TextFont::from_font_size(13.0),
            TextColor(Color::srgb(0.85, 0.88, 0.92)),
            DescField::Functional,
        ));
        card.spawn((
            Text::new(technical),
            TextFont::from_font_size(13.0),
            TextColor(Color::srgb(0.70, 0.76, 0.85)),
            DescField::Technical,
        ));

        card.spawn((
            Node {
                margin: UiRect::top(Val::Px(6.0)),
                display: if impacts.is_some() {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
            Text::new("Performance impact"),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgb(0.95, 0.95, 0.55)),
            DescField::ImpactHeading,
            ImpactRowNode,
        ));

        spawn_impact_row(card, "CPU", DescField::ImpactCpu, impacts.map(|i| i.0));
        spawn_impact_row(card, "GPU", DescField::ImpactGpu, impacts.map(|i| i.1));
        spawn_impact_row(card, "RAM", DescField::ImpactRam, impacts.map(|i| i.2));
        spawn_impact_row(card, "VRAM", DescField::ImpactVram, impacts.map(|i| i.3));
    });
}

fn spawn_impact_row(
    card: &mut ChildSpawnerCommands,
    label: &str,
    field: DescField,
    impact: Option<Impact>,
) {
    card.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            display: if impact.is_some() {
                Display::Flex
            } else {
                Display::None
            },
            ..default()
        },
        ImpactRowNode,
    ))
    .with_children(|row| {
        row.spawn((
            Text::new(format!("{:<5}: ", label)),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgb(0.80, 0.82, 0.88)),
        ));
        let (value_text, color) = match impact {
            Some(i) => (i.label().to_string(), i.color()),
            None => (String::new(), Color::WHITE),
        };
        row.spawn((
            Text::new(value_text),
            TextFont::from_font_size(14.0),
            TextColor(color),
            field,
        ));
    });
}

/// Returns (title, functional, technical, optional impacts (cpu, gpu, ram, vram)).
fn describe_for_layout(
    tab: SettingsTab,
    menu_idx: usize,
    preset: GraphicsPreset,
    settings: &GameSettings,
) -> (
    String,
    String,
    String,
    Option<(Impact, Impact, Impact, Impact)>,
) {
    match description_for(tab, menu_idx, preset, settings) {
        DescriptionKind::Param(ParamDescription {
            title,
            functional,
            technical,
            cpu,
            gpu,
            ram,
            vram,
        }) => (
            title.into(),
            functional.into(),
            technical.into(),
            Some((cpu, gpu, ram, vram)),
        ),
        DescriptionKind::Preset {
            title,
            functional,
            technical,
        } => (title.into(), functional.into(), technical.into(), None),
        DescriptionKind::None => (
            "Back".into(),
            "Return to the previous screen without changing the configuration.".into(),
            String::new(),
            None,
        ),
    }
}

pub fn update_settings_description(
    focus: Res<MenuFocus>,
    preset: Res<GraphicsPreset>,
    tab: Res<SettingsTab>,
    settings: Res<GameSettings>,
    mut q: Query<(&DescField, &mut Text, &mut TextColor)>,
    mut rows: Query<&mut Node, With<ImpactRowNode>>,
) {
    // The card spawns populated for focus index 0 and entering Settings resets
    // MenuFocus (a change), so these triggers also cover the entry frame.
    if !focus.is_changed() && !preset.is_changed() && !tab.is_changed() && !settings.is_changed() {
        return;
    }
    let (title, functional, technical, impacts) =
        describe_for_layout(*tab, focus.index, *preset, &settings);
    for (field, mut text, mut color) in &mut q {
        match field {
            DescField::Title => text.0 = title.clone(),
            DescField::Functional => text.0 = functional.clone(),
            DescField::Technical => text.0 = technical.clone(),
            DescField::ImpactHeading => text.0 = "Performance impact".into(),
            DescField::ImpactCpu => apply_impact(&mut text, &mut color, impacts.map(|i| i.0)),
            DescField::ImpactGpu => apply_impact(&mut text, &mut color, impacts.map(|i| i.1)),
            DescField::ImpactRam => apply_impact(&mut text, &mut color, impacts.map(|i| i.2)),
            DescField::ImpactVram => apply_impact(&mut text, &mut color, impacts.map(|i| i.3)),
        }
    }
    let display = if impacts.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut rows {
        if node.display != display {
            node.display = display;
        }
    }
}

/// Adjust the settings menu column's [`ScrollPosition`] so the currently
/// focused [`MenuButton`] is fully visible. The column itself has a capped
/// `max_height` + `Overflow::scroll_y`, so this is what makes D-pad navigation
/// past the visible area actually scroll into view on small screens.
pub fn scroll_focused_into_view(
    focus: Res<MenuFocus>,
    tab: Res<SettingsTab>,
    settings: Res<GameSettings>,
    mut columns: Query<(&ComputedNode, &Children, &mut ScrollPosition), With<SettingsMenuColumn>>,
    buttons: Query<(&ComputedNode, &MenuButton)>,
) {
    if !focus.is_changed() && !tab.is_changed() && !settings.is_changed() {
        return;
    }
    let Ok((column_node, children, mut scroll)) = columns.single_mut() else {
        return;
    };
    let viewport_height = column_node.size().y;
    // Walk the column's direct children in order, accumulating heights to
    // compute each button's top offset in the (unscrolled) content space.
    // `row_gap` matches the column's `Node.row_gap` above.
    let row_gap = 4.0_f32;
    let mut y_offset = 0.0_f32;
    let mut focused: Option<(f32, f32)> = None; // (top, height)
    for child in children.iter() {
        let Ok((child_node, btn)) = buttons.get(child) else {
            continue;
        };
        let h = child_node.size().y;
        if btn.0 == focus.index {
            focused = Some((y_offset, h));
            break;
        }
        y_offset += h + row_gap;
    }
    let Some((top, height)) = focused else {
        return;
    };
    let bottom = top + height;
    let mut s = scroll.y;
    if top < s {
        s = top;
    } else if bottom > s + viewport_height {
        s = bottom - viewport_height;
    }
    s = s.max(0.0);
    if (scroll.y - s).abs() > f32::EPSILON {
        scroll.y = s;
    }
}

fn apply_impact(text: &mut Text, color: &mut TextColor, impact: Option<Impact>) {
    match impact {
        Some(i) => {
            text.0 = i.label().to_string();
            color.0 = i.color();
        }
        None => {
            text.0.clear();
            color.0 = Color::WHITE;
        }
    }
}

fn spawn_toggle_button<M: Component>(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    label: String,
    marker: M,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(28.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(Color::srgb(0.7, 0.7, 0.75)),
            MenuButton(index),
            Button,
        ))
        .with_child((
            Text::new(label),
            TextFont::from_font_size(20.0),
            TextColor(Color::WHITE),
            marker,
        ));
}

/// OnEnter(Paused): build the pause overlay (torn down by `despawn_all`).
pub fn spawn_pause_overlay(
    mut commands: Commands,
    mode: Res<GameMode>,
    players: Res<PlayerControllers>,
    mut menu_focus: ResMut<MenuFocus>,
) {
    menu_focus.index = 0;
    build_pause_overlay(&mut commands, &mode, &players);
}

/// While paused: rebuild the overlay when the controller set changes, so a
/// pad disconnect mid-pause refreshes the "Pad disconnected" warning lines.
pub fn refresh_pause_overlay(
    mut commands: Commands,
    mode: Res<GameMode>,
    players: Res<PlayerControllers>,
    overlay: Query<Entity, With<PauseOverlay>>,
) {
    if !players.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    build_pause_overlay(&mut commands, &mode, &players);
}

fn build_pause_overlay(commands: &mut Commands, mode: &GameMode, players: &PlayerControllers) {
    let missing: Vec<PlayerSlot> = mode
        .active_slots()
        .iter()
        .copied()
        .filter(|s| players.get(*s).is_none())
        .collect();
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
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            PauseOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Pause"),
                TextFont::from_font_size(40.0),
                TextColor(Color::WHITE),
            ));
            for slot in &missing {
                parent.spawn((
                    Text::new(format!("Pad disconnected: {}", slot.label())),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::srgb(1.0, 0.55, 0.30)),
                ));
            }
            spawn_menu_button(parent, 0, "Resume", Side::Left.color());
            spawn_menu_button(parent, 1, "Settings", Color::srgb(0.7, 0.7, 0.75));
            spawn_menu_button(parent, 2, "Main menu", Side::Right.color());
            parent.spawn((
                Text::new("D-pad: navigate   A: select   Start/B: resume"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
            ));
        });
}

pub fn update_settings_toggle_texts(
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    rt_avail: Res<RaytracingAvailable>,
    preset: Res<GraphicsPreset>,
    mut toggles: Query<(&SettingsToggleText, &mut Text), Without<PresetText>>,
    mut preset_texts: Query<&mut Text, With<PresetText>>,
) {
    let changed = settings.is_changed()
        || dlss_avail.is_changed()
        || rt_avail.is_changed()
        || preset.is_changed();
    if !changed {
        return;
    }
    for (tag, mut text) in &mut toggles {
        text.0 = param_label(tag.0, &settings, dlss_avail.0, rt_avail.0);
    }
    let preset_label = format!("Preset: {}", preset.label());
    for mut text in &mut preset_texts {
        text.0 = preset_label.clone();
    }
}

/// OnEnter(Ended): build the victory screen from the [`Winner`] resource (set
/// by `check_winner` right before the transition).
pub fn spawn_endgame_overlay(
    mut commands: Commands,
    winner: Res<Winner>,
    mut menu_focus: ResMut<MenuFocus>,
) {
    let Some(winner) = winner.0 else {
        return;
    };
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
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            EndgameOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Player {} wins", winner.label())),
                TextFont::from_font_size(40.0),
                TextColor(winner.color()),
            ));
            spawn_menu_button(parent, 0, "Main menu", Color::WHITE);
            parent.spawn((
                Text::new("A: back to menu"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
            ));
        });
}

/// OnExit(SideSelect): drop every pad's in-progress seat choice so the next
/// visit starts from a clean slate.
pub fn clear_seat_selections(mut commands: Commands, seats: Query<Entity, With<SeatSelection>>) {
    for entity in &seats {
        commands.entity(entity).remove::<SeatSelection>();
    }
}

/// OnEnter(SideSelect): build the seat-pick screen for the chosen GameMode.
pub fn spawn_sideselect_overlay(mut commands: Commands, mode: Res<GameMode>) {
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
            BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.65)),
            SideSelectOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Choose a side"),
                TextFont::from_font_size(36.0),
                TextColor(Color::WHITE),
            ));
            if *mode == GameMode::TwoVsTwo {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(20.0),
                        ..default()
                    },))
                    .with_children(|col| {
                        col.spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(48.0),
                            ..default()
                        },))
                            .with_children(|row| {
                                spawn_side_card(row, PlayerSlot::LeftTop);
                                spawn_side_card(row, PlayerSlot::RightTop);
                            });
                        col.spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(48.0),
                            ..default()
                        },))
                            .with_children(|row| {
                                spawn_side_card(row, PlayerSlot::LeftBottom);
                                spawn_side_card(row, PlayerSlot::RightBottom);
                            });
                    });
            } else {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(48.0),
                        ..default()
                    },))
                    .with_children(|row| {
                        spawn_side_card(row, PlayerSlot::LeftBottom);
                        spawn_side_card(row, PlayerSlot::RightBottom);
                    });
            }
            parent.spawn((
                Text::new("D-pad: choose seat / nation   A: confirm   B: back   Start: launch"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.75, 0.75, 0.8)),
            ));
        });
}

fn spawn_side_card(parent: &mut ChildSpawnerCommands, slot: PlayerSlot) {
    let title = match slot {
        PlayerSlot::LeftBottom => "Left Bottom",
        PlayerSlot::LeftTop => "Left Top",
        PlayerSlot::RightBottom => "Right Bottom",
        PlayerSlot::RightTop => "Right Top",
    };
    parent
        .spawn((
            Node {
                width: Val::Px(230.0),
                height: Val::Px(176.0),
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            BackgroundColor(CARD_NORMAL),
            BorderColor::all(slot.side().color()),
            SideCard(slot),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(title),
                TextFont::from_font_size(22.0),
                TextColor(slot.side().color()),
            ));
            // Controller name (item: show which pad holds the seat).
            card.spawn((
                Text::new("—"),
                TextFont::from_font_size(15.0),
                TextColor(Color::srgb(0.65, 0.65, 0.72)),
                SideCardLine {
                    slot,
                    line: CardLine::Controller,
                },
            ));
            card.spawn((
                Text::new("Available"),
                TextFont::from_font_size(18.0),
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
                SideCardLine {
                    slot,
                    line: CardLine::Status,
                },
            ));
            // Nation choice (shown once the seat is claimed).
            card.spawn((
                Text::new("—"),
                TextFont::from_font_size(17.0),
                TextColor(Color::srgb(0.8, 0.8, 0.85)),
                SideCardLine {
                    slot,
                    line: CardLine::Nation,
                },
            ));
        });
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, index: usize, label: &str, border: Color) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(32.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(220.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(border),
            MenuButton(index),
            Button,
        ))
        .with_child((
            Text::new(label),
            TextFont::from_font_size(22.0),
            TextColor(Color::WHITE),
        ));
}

// ────────────────────────────────────────────────────────────────────────────
// Focus visuals (refresh each frame from focus resources/components)
// ────────────────────────────────────────────────────────────────────────────

pub fn apply_menu_focus_visual(
    state: Res<State<GameState>>,
    focus: Res<MenuFocus>,
    mut buttons: Query<(&MenuButton, &mut BackgroundColor)>,
) {
    let active = matches!(
        *state.get(),
        GameState::Menu | GameState::Settings | GameState::Paused | GameState::Ended
    );
    for (btn, mut bg) in &mut buttons {
        let target = if active && btn.0 == focus.index {
            BTN_FOCUSED
        } else {
            BTN_NORMAL
        };
        // Compare before writing: an unconditional write would flag every
        // button changed every frame and re-extract the UI for nothing.
        if bg.0 != target {
            bg.0 = target;
        }
    }
}

/// Trim a gamepad's reported name so it fits inside a card.
fn pad_short_name(name: &str) -> String {
    const MAX: usize = 18;
    if name.chars().count() > MAX {
        let head: String = name.chars().take(MAX - 1).collect();
        format!("{head}…")
    } else {
        name.to_string()
    }
}

pub fn update_sideselect_cards(
    changed_seats: Query<(), Changed<SeatSelection>>,
    mut removed_seats: RemovedComponents<SeatSelection>,
    seats: Query<(&SeatSelection, Option<&Name>)>,
    mut texts: Query<(&SideCardLine, &mut Text, &mut TextColor)>,
    mut cards: Query<(&SideCard, &mut BackgroundColor, &mut BorderColor)>,
) {
    // Repaint only when a seat selection actually moved (insert/mutation/
    // removal) — every `Text` write below re-uploads glyphs. The cards spawn
    // with correct "no seats" defaults, so the entry frame needs no repaint.
    if changed_seats.is_empty() && removed_seats.read().next().is_none() {
        return;
    }

    // Aggregate per slot: who claimed it (at most one), and who is just hovering.
    let mut claimant: [Option<(SeatPhase, usize, Option<String>)>; 4] = Default::default();
    let mut hover_count: [usize; 4] = [0; 4];
    let mut hover_name: [Option<String>; 4] = Default::default();
    for (sel, name) in &seats {
        let i = sel.hovered.index();
        let nm = name.map(|n| pad_short_name(n.as_str()));
        if sel.claims_seat() {
            claimant[i] = Some((sel.phase, sel.nation, nm));
        } else {
            if hover_count[i] == 0 {
                hover_name[i] = nm;
            }
            hover_count[i] += 1;
        }
    }

    let dim = Color::srgb(0.5, 0.5, 0.56);
    let n_nations = Nation::ALL.len();

    for (line, mut text, mut color) in &mut texts {
        let i = line.slot.index();
        let side_color = line.slot.side().color();
        match line.line {
            CardLine::Status => match &claimant[i] {
                Some((SeatPhase::Locked, _, _)) => {
                    text.0 = "Locked in".to_string();
                    color.0 = side_color;
                }
                Some((SeatPhase::PickingNation, _, _)) => {
                    text.0 = "Choosing nation".to_string();
                    color.0 = Color::WHITE;
                }
                _ if hover_count[i] > 0 => {
                    text.0 = format!("Selected ({})", hover_count[i]);
                    color.0 = Color::WHITE;
                }
                _ => {
                    text.0 = "Available".to_string();
                    color.0 = Color::srgb(0.7, 0.7, 0.75);
                }
            },
            CardLine::Controller => {
                if let Some((_, _, name)) = &claimant[i] {
                    text.0 = name.clone().unwrap_or_else(|| "Controller".to_string());
                    color.0 = Color::srgb(0.85, 0.85, 0.9);
                } else if hover_count[i] == 1 {
                    text.0 = hover_name[i]
                        .clone()
                        .unwrap_or_else(|| "Controller".to_string());
                    color.0 = Color::srgb(0.7, 0.7, 0.78);
                } else if hover_count[i] > 1 {
                    text.0 = format!("{} controllers", hover_count[i]);
                    color.0 = Color::srgb(0.7, 0.7, 0.78);
                } else {
                    text.0 = "—".to_string();
                    color.0 = dim;
                }
            }
            CardLine::Nation => match &claimant[i] {
                Some((phase, nation, _)) => {
                    let n = Nation::ALL[nation % n_nations];
                    text.0 = format!("▸ {}", n.label());
                    color.0 = if *phase == SeatPhase::Locked {
                        side_color
                    } else {
                        Color::WHITE
                    };
                }
                None => {
                    text.0 = "—".to_string();
                    color.0 = dim;
                }
            },
        }
    }

    for (card, mut bg, mut border) in &mut cards {
        let i = card.0.index();
        let locked = matches!(claimant[i], Some((SeatPhase::Locked, _, _)));
        let occupied = claimant[i].is_some() || hover_count[i] > 0;
        bg.0 = if occupied { CARD_HOVERED } else { CARD_NORMAL };
        let border_color = if locked {
            Color::WHITE
        } else {
            card.0.side().color()
        };
        *border = BorderColor::all(border_color);
    }
}
