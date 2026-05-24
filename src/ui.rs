use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::VolumetricFog;
use bevy::pbr::{Atmosphere, AtmosphereSettings, DistanceFog, FogFalloff};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};

use crate::common::*;
use crate::graphics::{
    description_for, param_label, slot_count, tab_slots, DescriptionKind, GraphicsPreset, Impact,
    MenuSlot, ParamDescription, ParamId,
};
use crate::towers::{collides_with_existing_tower, is_valid_tower_zone, spawn_tower};
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
pub struct ClockText;

#[derive(Component)]
pub struct GameHud;

#[derive(Component)]
pub struct MenuOverlay;

#[derive(Component)]
pub struct EndgameOverlay;

#[derive(Component)]
pub struct SettingsOverlay;

#[derive(Component)]
pub struct PauseOverlay;

#[derive(Component, Clone, Copy)]
pub struct SettingsToggleText(pub ParamId);

#[derive(Component)]
pub struct PresetText;

/// Marker on a tab toggle in the settings overlay. The overlay is rebuilt
/// when the active tab changes, so the highlight stays implicit (colours are
/// set at spawn time).
#[derive(Component, Clone, Copy)]
pub struct SettingsTabButton;

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
    let hud_bg = Color::srgba(0.0, 0.0, 0.0, 0.65);
    let hud_border = Color::srgb(0.85, 0.85, 0.9);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                right: Val::Px(12.0),
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(hud_bg),
            BorderColor::all(hud_border),
            Visibility::Hidden,
            GameHud,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Left Base: 20/20"),
                TextFont::from_font_size(22.0),
                TextColor(Side::Left.color()),
                BaseHpText(Side::Left),
            ));
            parent.spawn((
                Text::new("06:00"),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgb(0.95, 0.93, 0.78)),
                ClockText,
            ));
            parent.spawn((
                Text::new("Right Base: 20/20"),
                TextFont::from_font_size(22.0),
                TextColor(Side::Right.color()),
                BaseHpText(Side::Right),
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                right: Val::Px(12.0),
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(hud_bg),
            BorderColor::all(hud_border),
            Visibility::Hidden,
            GameHud,
        ))
        .with_children(|parent| {
            spawn_player_panel(parent, Side::Left);
            spawn_player_panel(parent, Side::Right);
        });
}

pub fn update_game_hud_visibility(
    state: Res<GameState>,
    mut hud: Query<&mut Visibility, With<GameHud>>,
) {
    if !state.is_changed() {
        return;
    }
    let visible = matches!(*state, GameState::Playing | GameState::Paused);
    for mut vis in &mut hud {
        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
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

pub fn update_clock_text(gtime: Res<GameTime>, mut q: Query<&mut Text, With<ClockText>>) {
    let hours_f = (gtime.0 / SUN_DAY_PERIOD * 24.0 + 6.0).rem_euclid(24.0);
    let h = hours_f.floor() as u32;
    let m = ((hours_f - h as f32) * 60.0).floor() as u32;
    for mut text in &mut q {
        text.0 = format!("{:02}:{:02}", h, m);
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
            spawn_menu_button(parent, 0, "Play", Side::Left.color());
            spawn_menu_button(parent, 1, "Settings", Color::srgb(0.7, 0.7, 0.75));
            spawn_menu_button(parent, 2, "Quit", Side::Right.color());
            parent.spawn((
                Text::new("D-pad: navigate   A: select"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });
}

pub fn update_settings_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    preset: Res<GraphicsPreset>,
    tab: Res<SettingsTab>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<SettingsOverlay>>,
) {
    // Rebuild on state change OR on tab change (so switching tab refreshes
    // the listed parameters).
    let rebuild = state.is_changed() || (tab.is_changed() && *state == GameState::Settings);
    if !rebuild {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if *state != GameState::Settings {
        return;
    }
    menu_focus.index = 0;
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
                row_gap: Val::Px(20.0),
                padding: UiRect::axes(Val::Px(24.0), Val::Px(24.0)),
                ..default()
            },
            // Translucent so the user can see live changes behind the menu.
            BackgroundColor(Color::srgba(0.05, 0.06, 0.10, 0.65)),
            SettingsOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont::from_font_size(52.0),
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
                    spawn_settings_menu_column(row, tab, &settings, dlss_avail.0, preset);
                    spawn_description_card(row, tab, preset);
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
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
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
                    SettingsTabButton,
                ))
                .with_child((
                    Text::new(tab.label()),
                    TextFont::from_font_size(20.0),
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
    preset: GraphicsPreset,
) {
    row.spawn((Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: Val::Px(8.0),
        min_width: Val::Px(420.0),
        ..default()
    },))
    .with_children(|col| {
        for (i, slot) in tab_slots(tab).iter().enumerate() {
            match slot {
                MenuSlot::Preset => spawn_preset_button(col, i, preset),
                MenuSlot::Param(id) => {
                    let label = param_label(*id, settings, dlss_supported);
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
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(Color::srgb(0.85, 0.78, 0.30)),
            MenuButton(index),
        ))
        .with_child((
            Text::new(format!("Preset: {}", preset.label())),
            TextFont::from_font_size(22.0),
            TextColor(Color::srgb(0.95, 0.90, 0.55)),
            PresetText,
        ));
}

fn spawn_description_card(row: &mut ChildSpawnerCommands, tab: SettingsTab, preset: GraphicsPreset) {
    let card_bg = Color::srgba(0.10, 0.11, 0.15, 0.90);
    let card_border = Color::srgb(0.32, 0.34, 0.42);
    row.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(10.0),
            width: Val::Px(480.0),
            min_height: Val::Px(420.0),
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(card_bg),
        BorderColor::all(card_border),
    ))
    .with_children(|card| {
        let (title, functional, technical, impacts) = describe_for_layout(tab, 0, preset);

        card.spawn((
            Text::new(title),
            TextFont::from_font_size(24.0),
            TextColor(Color::srgb(0.95, 0.95, 0.98)),
            DescField::Title,
        ));
        card.spawn((
            Text::new(functional),
            TextFont::from_font_size(15.0),
            TextColor(Color::srgb(0.85, 0.88, 0.92)),
            DescField::Functional,
        ));
        card.spawn((
            Text::new(technical),
            TextFont::from_font_size(15.0),
            TextColor(Color::srgb(0.70, 0.76, 0.85)),
            DescField::Technical,
        ));

        card.spawn((
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
            Text::new(if impacts.is_some() { "Performance impact" } else { "" }),
            TextFont::from_font_size(16.0),
            TextColor(Color::srgb(0.95, 0.95, 0.55)),
            DescField::ImpactHeading,
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
    card.spawn((Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        ..default()
    },))
    .with_children(|row| {
        row.spawn((
            Text::new(format!("{:<5}: ", label)),
            TextFont::from_font_size(16.0),
            TextColor(Color::srgb(0.80, 0.82, 0.88)),
        ));
        let (value_text, color) = match impact {
            Some(i) => (i.label().to_string(), i.color()),
            None => (String::new(), Color::WHITE),
        };
        row.spawn((
            Text::new(value_text),
            TextFont::from_font_size(16.0),
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
) -> (String, String, String, Option<(Impact, Impact, Impact, Impact)>) {
    match description_for(tab, menu_idx, preset) {
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
    state: Res<GameState>,
    focus: Res<MenuFocus>,
    preset: Res<GraphicsPreset>,
    tab: Res<SettingsTab>,
    mut q: Query<(&DescField, &mut Text, &mut TextColor)>,
) {
    if *state != GameState::Settings {
        return;
    }
    if !focus.is_changed() && !preset.is_changed() && !state.is_changed() && !tab.is_changed() {
        return;
    }
    let (title, functional, technical, impacts) = describe_for_layout(*tab, focus.index, *preset);
    for (field, mut text, mut color) in &mut q {
        match field {
            DescField::Title => text.0 = title.clone(),
            DescField::Functional => text.0 = functional.clone(),
            DescField::Technical => text.0 = technical.clone(),
            DescField::ImpactHeading => {
                text.0 = if impacts.is_some() {
                    "Performance impact".into()
                } else {
                    String::new()
                };
            }
            DescField::ImpactCpu => apply_impact(&mut text, &mut color, impacts.map(|i| i.0)),
            DescField::ImpactGpu => apply_impact(&mut text, &mut color, impacts.map(|i| i.1)),
            DescField::ImpactRam => apply_impact(&mut text, &mut color, impacts.map(|i| i.2)),
            DescField::ImpactVram => apply_impact(&mut text, &mut color, impacts.map(|i| i.3)),
        }
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
                padding: UiRect::axes(Val::Px(36.0), Val::Px(14.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(Color::srgb(0.7, 0.7, 0.75)),
            MenuButton(index),
        ))
        .with_child((
            Text::new(label),
            TextFont::from_font_size(24.0),
            TextColor(Color::WHITE),
            marker,
        ));
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

pub fn update_settings_toggle_texts(
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    preset: Res<GraphicsPreset>,
    mut toggles: Query<(&SettingsToggleText, &mut Text), Without<PresetText>>,
    mut preset_texts: Query<&mut Text, With<PresetText>>,
) {
    let changed = settings.is_changed() || dlss_avail.is_changed() || preset.is_changed();
    if !changed {
        return;
    }
    for (tag, mut text) in &mut toggles {
        text.0 = param_label(tag.0, &settings, dlss_avail.0);
    }
    let preset_label = format!("Preset: {}", preset.label());
    for mut text in &mut preset_texts {
        text.0 = preset_label.clone();
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
                    Text::new("A: back to menu"),
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
                Text::new("Choose a side"),
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
                    "D-pad left/right: choose   A: confirm   B: cancel   Start: launch",
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
                    Side::Left => "Left Player",
                    Side::Right => "Right Player",
                }),
                TextFont::from_font_size(26.0),
                TextColor(side.color()),
            ));
            card.spawn((
                Text::new("Available"),
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
    units: Query<(&Side, &UnitKind), With<Unit>>,
    mut slots: Query<(&PanelSlot, &mut Node, &mut BackgroundColor, &mut BorderColor)>,
) {
    let active = matches!(*state, GameState::Playing | GameState::Paused);
    let miners_left = units
        .iter()
        .filter(|(s, k)| **s == Side::Left && **k == UnitKind::Miner)
        .count();
    let miners_right = units
        .iter()
        .filter(|(s, k)| **s == Side::Right && **k == UnitKind::Miner)
        .count();
    for (slot, mut node, mut bg, mut border) in &mut slots {
        let hidden = slot.index == 1
            && match slot.side {
                Side::Left => miners_left >= MAX_MINERS_PER_SIDE,
                Side::Right => miners_right >= MAX_MINERS_PER_SIDE,
            };
        let new_display = if hidden { Display::None } else { Display::Flex };
        if node.display != new_display {
            node.display = new_display;
        }
        if hidden {
            continue;
        }
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
            text.0 = "Locked in".to_string();
            color.0 = slot.0.color();
        } else if hovered > 0 {
            text.0 = format!("Selected ({})", hovered);
            color.0 = Color::WHITE;
        } else {
            text.0 = "Available".to_string();
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

pub fn spawn_initial_miners(
    state: Res<GameState>,
    mut commands: Commands,
    lib: Res<MatLibrary>,
    units: Query<Entity, With<Unit>>,
) {
    if !state.is_changed() || *state != GameState::Playing {
        return;
    }
    // Skip if units already exist (e.g. resuming from Paused).
    if units.iter().next().is_some() {
        return;
    }
    spawn_miner(&mut commands, &lib, Side::Left, 0);
    spawn_miner(&mut commands, &lib, Side::Right, 0);
}

pub fn gameplay_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    lib: Res<MatLibrary>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut focuses: Query<(Entity, &mut PlayerFocus)>,
    gamepads: Query<&Gamepad>,
    units: Query<(&Side, &UnitKind), With<Unit>>,
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

        // While this side is placing a tower, let placement_system claim all inputs
        // (D-pad, South, West). Otherwise re-arming would swallow the confirm press.
        if placement.get(focus.side).is_some() {
            continue;
        }

        let miner_count = units
            .iter()
            .filter(|(s, k)| **s == focus.side && **k == UnitKind::Miner)
            .count();
        let slot_hidden = |slot: usize| slot == 1 && miner_count >= MAX_MINERS_PER_SIDE;
        // If we're currently parked on a hidden slot (cap just reached), nudge
        // off it so visuals and input stay coherent.
        if slot_hidden(focus.index) {
            focus.index = next_visible_slot(focus.index, 1, &slot_hidden);
        }

        if pad.just_pressed(GamepadButton::DPadLeft) {
            focus.index = next_visible_slot(focus.index, -1, &slot_hidden);
        } else if pad.just_pressed(GamepadButton::DPadRight) {
            focus.index = next_visible_slot(focus.index, 1, &slot_hidden);
        }

        if pad.just_pressed(GamepadButton::West) {
            arm_placement(&mut placement, focus.side);
        }

        if pad.just_pressed(GamepadButton::South) {
            match focus.index {
                0 => {
                    if gold.try_spend(focus.side, SOLDIER_COST) {
                        let count = units
                            .iter()
                            .filter(|(s, k)| **s == focus.side && **k == UnitKind::Soldier)
                            .count();
                        spawn_soldier(&mut commands, &lib, focus.side, count % LANE_COUNT);
                    }
                }
                1 => {
                    let miner_count = units
                        .iter()
                        .filter(|(s, k)| **s == focus.side && **k == UnitKind::Miner)
                        .count();
                    if miner_count < MAX_MINERS_PER_SIDE
                        && gold.try_spend(focus.side, MINER_COST)
                    {
                        spawn_miner(&mut commands, &lib, focus.side, miner_count);
                    }
                }
                2 => {
                    if gold.try_spend(focus.side, ARCHER_COST) {
                        let count = units
                            .iter()
                            .filter(|(s, k)| **s == focus.side && **k == UnitKind::Archer)
                            .count();
                        spawn_archer(&mut commands, &lib, focus.side, count % LANE_COUNT);
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
    existing_towers: Query<&Transform, With<Tower>>,
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

        let tower_positions: Vec<Vec3> = existing_towers.iter().map(|t| t.translation).collect();
        let in_zone = is_valid_tower_zone(side, pos);
        let no_overlap = !collides_with_existing_tower(pos, &tower_positions);
        let can_afford = gold.get(side) >= TOWER_COST;
        let valid = in_zone && no_overlap && can_afford;

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
    dlss_avail: Res<DlssAvailable>,
    preset: Res<GraphicsPreset>,
    mut tab: ResMut<SettingsTab>,
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

    let slots = slot_count(*tab);
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

    if back {
        *state = origin.to_state();
        return;
    }

    if !activate {
        return;
    }
    let slot = tab_slots(*tab).get(menu_focus.index).copied();
    match slot {
        Some(MenuSlot::Preset) => {
            let next = preset.cycle();
            next.apply(&mut settings, dlss_avail.0);
        }
        Some(MenuSlot::Param(id)) => match id {
            ParamId::Fullscreen => settings.fullscreen = !settings.fullscreen,
            ParamId::VSync => settings.vsync = !settings.vsync,
            ParamId::Hdr => settings.hdr = !settings.hdr,
            ParamId::Tonemapping => settings.tonemapping = (settings.tonemapping + 1) % 4,
            ParamId::Raytracing => {
                if cfg!(feature = "raytracing") {
                    settings.raytracing = !settings.raytracing;
                }
            }
            ParamId::Dlss => {
                if cfg!(feature = "dlss") && dlss_avail.0 {
                    settings.dlss = !settings.dlss;
                }
            }
            ParamId::Taa => settings.taa = !settings.taa,
            ParamId::Bloom => settings.bloom = !settings.bloom,
            ParamId::Atmosphere => settings.atmosphere = !settings.atmosphere,
            ParamId::VolumetricFog => settings.volumetric_fog = !settings.volumetric_fog,
            ParamId::DistanceFog => settings.distance_fog = !settings.distance_fog,
        },
        Some(MenuSlot::Back) | None => *state = origin.to_state(),
    }
}

pub fn apply_graphics_settings(
    settings: Res<GameSettings>,
    atmo: Res<AtmosphereHandle>,
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    mut tonemap: Query<&mut Tonemapping>,
    mut windows: Query<&mut Window>,
) {
    if !settings.is_changed() {
        return;
    }
    // Window mode + vsync.
    let mode = if settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    let present = if settings.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    for mut window in &mut windows {
        if window.mode != mode {
            window.mode = mode;
        }
        if window.present_mode != present {
            window.present_mode = present;
        }
    }
    // Per-camera components.
    for cam in &cameras {
        let mut e = commands.entity(cam);
        // HDR is a marker component on the camera in Bevy 0.18.
        if settings.hdr {
            e.insert(bevy::render::view::Hdr);
        } else {
            e.remove::<bevy::render::view::Hdr>();
        }
        if settings.bloom {
            e.insert(Bloom::NATURAL);
        } else {
            e.remove::<Bloom>();
        }
        if settings.atmosphere {
            e.insert((
                Atmosphere::earthlike(atmo.0.clone()),
                AtmosphereSettings::default(),
            ));
        } else {
            e.remove::<Atmosphere>().remove::<AtmosphereSettings>();
        }
        if settings.volumetric_fog {
            e.insert(VolumetricFog {
                ambient_intensity: 0.05,
                ..default()
            });
        } else {
            e.remove::<VolumetricFog>();
        }
        if settings.distance_fog {
            e.insert(DistanceFog {
                color: Color::srgba(0.55, 0.70, 0.85, 1.0),
                falloff: FogFalloff::ExponentialSquared { density: 0.012 },
                ..default()
            });
        } else {
            e.remove::<DistanceFog>();
        }
        if settings.taa {
            e.insert(TemporalAntiAliasing::default());
        } else {
            e.remove::<TemporalAntiAliasing>();
        }
    }
    // Tonemapping (mutates existing component on the camera).
    for mut t in &mut tonemap {
        *t = match settings.tonemapping {
            0 => Tonemapping::AcesFitted,
            1 => Tonemapping::TonyMcMapface,
            2 => Tonemapping::Reinhard,
            _ => Tonemapping::None,
        };
    }
}
