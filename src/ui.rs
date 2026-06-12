use bevy::anti_alias::fxaa::Fxaa;
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::VolumetricFog;
use bevy::pbr::{
    Atmosphere, AtmosphereSettings, DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion,
    ScreenSpaceAmbientOcclusionQualityLevel,
};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::motion_blur::MotionBlur;
use bevy::prelude::*;
use bevy::render::view::Msaa;
use bevy::window::{PresentMode, WindowMode};

use crate::common::*;
use crate::graphics::{
    DescriptionKind, GraphicsPreset, Impact, MenuSlot, ParamDescription, ParamId,
    bloom_intensity_value, description_for, exposure_ev100, fog_density_value, param_label,
    slot_count, tab_slots,
};
use crate::towers::{collides_with_existing_tower, is_valid_tower_zone, spawn_tower};
use crate::units::{spawn_archer, spawn_miner, spawn_priest, spawn_soldier};

#[derive(Component, Clone, Copy)]
pub struct PanelSlot {
    pub slot: PlayerSlot,
    pub index: usize,
}

#[derive(Component)]
pub struct GoldText(pub PlayerSlot);

/// Stats card text in a player's HUD: refreshes whenever that player's
/// focus index changes (showing Tower/Soldier/Archer/Miner specs).
#[derive(Component)]
pub struct FocusStatsText(pub PlayerSlot);

#[derive(Component)]
pub struct BaseHpText(pub PlayerSlot);

/// Marker for the top player HUD corners (LeftTop / RightTop). They are hidden
/// in 1v1 by [`update_game_hud_visibility`].
#[derive(Component)]
pub struct TopPlayerHud;

/// Marker on the root Node of each player corner; used by
/// [`apply_player_focus_visual`] to grey the whole corner when that slot's
/// base is destroyed.
#[derive(Component, Clone, Copy)]
pub struct PlayerCorner(pub PlayerSlot);

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

/// Per-frame snapshot of mouse interaction with the UI, populated by
/// [`read_mouse_ui`] at the head of the input chain and consumed by the
/// per-state input systems. This is what gives the otherwise gamepad-only game
/// clickable buttons for debugging: a hover moves the menu focus, a left-click
/// activates the focused item exactly as the gamepad's South button would.
///
/// Only the currently active overlay's buttons exist (the others are despawned
/// on state change), so the `MenuButton`-indexed fields are unambiguous across
/// the Menu / Pause / Settings / Endgame screens.
#[derive(Resource, Default)]
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
    *mouse = MouseUi::default();
    let clicked = mouse_buttons.just_pressed(MouseButton::Left);
    for (btn, interaction) in &menu_buttons {
        match interaction {
            Interaction::Hovered => mouse.menu_hover = Some(btn.0),
            Interaction::Pressed => {
                mouse.menu_hover = Some(btn.0);
                if clicked {
                    mouse.menu_click = Some(btn.0);
                }
            }
            Interaction::None => {}
        }
    }
    for (slot, interaction) in &panel_buttons {
        match interaction {
            Interaction::Hovered => mouse.panel_hover = Some((slot.slot, slot.index)),
            Interaction::Pressed => {
                mouse.panel_hover = Some((slot.slot, slot.index));
                if clicked {
                    mouse.panel_click = Some((slot.slot, slot.index));
                }
            }
            Interaction::None => {}
        }
    }
    if clicked {
        for (tab, interaction) in &tab_buttons {
            if *interaction == Interaction::Pressed {
                mouse.tab_click = Some(tab.0);
            }
        }
    }
}

const BTN_NORMAL: Color = Color::srgb(0.16, 0.16, 0.20);
const BTN_FOCUSED: Color = Color::srgb(0.32, 0.32, 0.40);
const BTN_DISABLED: Color = Color::srgb(0.10, 0.10, 0.12);
const BORDER_DISABLED: Color = Color::srgb(0.30, 0.30, 0.34);
const HUD_BG_DISABLED: Color = Color::srgba(0.0, 0.0, 0.0, 0.40);
const HUD_BORDER_DISABLED: Color = Color::srgb(0.40, 0.40, 0.44);
const CARD_NORMAL: Color = Color::srgb(0.12, 0.13, 0.18);
const CARD_HOVERED: Color = Color::srgb(0.22, 0.23, 0.30);

pub fn setup_ui(mut commands: Commands) {
    let hud_bg = Color::srgba(0.0, 0.0, 0.0, 0.65);
    let hud_border = Color::srgb(0.85, 0.85, 0.9);
    // Top global bar: clock only (base HP lives inside each player corner).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                right: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
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
                Text::new("06:00"),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgb(0.95, 0.93, 0.78)),
                ClockText,
            ));
        });

    spawn_player_corner(&mut commands, PlayerSlot::LeftBottom, hud_bg, hud_border);
    spawn_player_corner(&mut commands, PlayerSlot::RightBottom, hud_bg, hud_border);
    spawn_player_corner(&mut commands, PlayerSlot::LeftTop, hud_bg, hud_border);
    spawn_player_corner(&mut commands, PlayerSlot::RightTop, hud_bg, hud_border);
}

fn spawn_player_corner(commands: &mut Commands, slot: PlayerSlot, bg: Color, border: Color) {
    let (left, right) = match slot.side() {
        Side::Left => (Val::Px(12.0), Val::Auto),
        Side::Right => (Val::Auto, Val::Px(12.0)),
    };
    // Bottom corners hug the bottom edge; top corners sit just below the
    // global HP/clock bar (which lives at top: 12 and is ~60px tall).
    let (top_val, bottom_val) = if slot.is_top() {
        (Val::Px(80.0), Val::Auto)
    } else {
        (Val::Auto, Val::Px(12.0))
    };
    let mut entity = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: top_val,
            bottom: bottom_val,
            left,
            right,
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(bg),
        BorderColor::all(border),
        Visibility::Hidden,
        GameHud,
        PlayerCorner(slot),
    ));
    if slot.is_top() {
        entity.insert(TopPlayerHud);
    }
    entity.with_children(|parent| {
        spawn_player_panel(parent, slot);
    });
}

pub fn update_game_hud_visibility(
    state: Res<GameState>,
    mode: Res<GameMode>,
    mut hud: Query<(&mut Visibility, Option<&TopPlayerHud>), With<GameHud>>,
) {
    if !state.is_changed() && !mode.is_changed() {
        return;
    }
    let active = matches!(*state, GameState::Playing | GameState::Paused);
    let two_v_two = *mode == GameMode::TwoVsTwo;
    for (mut vis, top) in &mut hud {
        let show = active && (top.is_none() || two_v_two);
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn spawn_player_panel(parent: &mut ChildSpawnerCommands, slot: PlayerSlot) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(4.0),
            min_width: Val::Px(170.0),
            ..default()
        },))
        .with_children(|panel| {
            // Base HP header (specific to this player slot).
            panel.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                Text::new(format!("{}: {}/{}", slot.label(), BASE_HP, BASE_HP)),
                TextFont::from_font_size(18.0),
                TextColor(slot.side().color()),
                BaseHpText(slot),
            ));
            // Order matches navigation order (0 → 3, top to bottom).
            spawn_category_header(panel, "Buildings");
            spawn_slot(panel, slot, 0, &format!("Tower ({}g)", TOWER_COST));
            spawn_category_header(panel, "Combat");
            spawn_slot(panel, slot, 1, &format!("Soldier ({}g)", SOLDIER_COST));
            spawn_slot(panel, slot, 2, &format!("Archer ({}g)", ARCHER_COST));
            spawn_slot(panel, slot, 3, &format!("Priest ({}g)", PRIEST_COST));
            spawn_category_header(panel, "Resources");
            spawn_slot(panel, slot, 4, &format!("Miner ({}g)", MINER_COST));
            panel.spawn((
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                Text::new("Gold: 10"),
                TextFont::from_font_size(18.0),
                TextColor(slot.side().color()),
                GoldText(slot),
            ));
            // Hover stats card: lit by update_focus_stats_text.
            panel
                .spawn((
                    Node {
                        margin: UiRect::top(Val::Px(6.0)),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.10, 0.85)),
                    BorderColor::all(Color::srgba(0.5, 0.5, 0.55, 0.7)),
                ))
                .with_child((
                    Text::new(focus_stats_string(0)),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.92, 0.92, 0.96)),
                    FocusStatsText(slot),
                ));
        });
}

/// Stats text for one of the five slot indices, matching the panel button
/// order (0 Tower, 1 Soldier, 2 Archer, 3 Priest, 4 Miner).
fn focus_stats_string(index: usize) -> String {
    match index {
        0 => format!(
            "Tower\nHP {}  DMG {}  RNG {:.1}  CD {:.1}s",
            TOWER_HP, TOWER_DAMAGE, TOWER_RANGE, TOWER_COOLDOWN
        ),
        1 => format!(
            "Soldier\nHP {}  DMG {}  SPD {:.1}  CD {:.1}s",
            SOLDIER_HP, SOLDIER_DAMAGE, SOLDIER_SPEED, SOLDIER_COOLDOWN
        ),
        2 => format!(
            "Archer\nHP {}  DMG {}  RNG {:.1}  CD {:.1}s",
            ARCHER_HP, ARCHER_DAMAGE, ARCHER_RANGE, ARCHER_COOLDOWN
        ),
        3 => format!(
            "Priest\nHP {}  HEAL {}  RNG {:.1}  CD {:.1}s",
            PRIEST_HP, PRIEST_HEAL, PRIEST_RANGE, PRIEST_COOLDOWN
        ),
        4 => format!(
            "Miner\nHP {}  CAP {}  SPD {:.1}  CD {:.1}s",
            MINER_HP, MINER_CAPACITY, MINER_SPEED, MINER_COOLDOWN
        ),
        _ => String::new(),
    }
}

pub fn update_focus_stats_text(
    focuses: Query<&PlayerFocus, Changed<PlayerFocus>>,
    all_focuses: Query<&PlayerFocus>,
    mut texts: Query<(&FocusStatsText, &mut Text)>,
) {
    // Only refresh when a player's focus actually moved. The text is otherwise
    // static and re-formatting it every frame is wasted work.
    if focuses.is_empty() {
        return;
    }
    let mut focus_per_slot: [Option<usize>; 4] = [None; 4];
    for f in &all_focuses {
        focus_per_slot[f.slot.index()] = Some(f.index);
    }
    for (tag, mut text) in &mut texts {
        let idx = focus_per_slot[tag.0.index()].unwrap_or(0);
        text.0 = focus_stats_string(idx);
    }
}

fn spawn_category_header(panel: &mut ChildSpawnerCommands, label: &str) {
    panel.spawn((
        Node {
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        },
        Text::new(label),
        TextFont::from_font_size(13.0),
        TextColor(Color::srgba(0.78, 0.80, 0.86, 0.85)),
    ));
}

fn spawn_slot(panel: &mut ChildSpawnerCommands, slot: PlayerSlot, index: usize, label: &str) {
    panel
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(slot.side().color()),
            PanelSlot { slot, index },
            Button,
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
    for (tag, mut text) in &mut texts {
        text.0 = format!("Gold: {}", gold.get(tag.0));
    }
}

pub fn update_clock_text(
    gtime: Res<GameTime>,
    mut last_shown: Local<(u32, u32)>,
    mut q: Query<&mut Text, With<ClockText>>,
) {
    let hours_f = (gtime.0 / SUN_DAY_PERIOD * 24.0 + 6.0).rem_euclid(24.0);
    let h = hours_f.floor() as u32;
    let m = ((hours_f - h as f32) * 60.0).floor() as u32;
    // Only push to the Text components when the displayed minute changes.
    // Bevy's change-detection wraps each Text mutation in a Changed flag and
    // re-uploads the glyph mesh, so skipping idempotent writes is cheap.
    if (h, m) == *last_shown {
        return;
    }
    *last_shown = (h, m);
    for mut text in &mut q {
        text.0 = format!("{:02}:{:02}", h, m);
    }
}

pub fn update_base_hp_text(
    bases: Query<(&PlayerSlot, &Health), (With<Base>, Changed<Health>)>,
    all_bases: Query<(&PlayerSlot, &Health), With<Base>>,
    mut texts: Query<(&BaseHpText, &mut Text)>,
) {
    // Only refresh when a base's HP actually changed (or a base was spawned).
    if bases.is_empty() {
        return;
    }
    for (tag, mut text) in &mut texts {
        if let Some((_, hp)) = all_bases.iter().find(|(s, _)| **s == tag.0) {
            text.0 = format!("{}: {}/{}", tag.0.label(), hp.current.max(0), hp.max);
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

pub fn update_settings_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    settings: Res<GameSettings>,
    dlss_avail: Res<DlssAvailable>,
    rt_avail: Res<RaytracingAvailable>,
    preset: Res<GraphicsPreset>,
    tab: Res<SettingsTab>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<SettingsOverlay>>,
) {
    // Rebuild on state change, on tab change, OR on settings change (so
    // sub-parameter rows appear/disappear immediately when their parent
    // toggle flips).
    let in_settings = *state == GameState::Settings;
    let rebuild =
        state.is_changed() || (in_settings && (tab.is_changed() || settings.is_changed()));
    if !rebuild {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if !in_settings {
        return;
    }
    // Reset focus only when entering Settings or switching tab. Settings-only
    // rebuilds (parameter toggles) keep focus where the user just acted.
    if state.is_changed() || tab.is_changed() {
        menu_focus.index = 0;
    }
    let slots_after = slot_count(*tab, &settings);
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
    state: Res<GameState>,
    focus: Res<MenuFocus>,
    preset: Res<GraphicsPreset>,
    tab: Res<SettingsTab>,
    settings: Res<GameSettings>,
    mut q: Query<(&DescField, &mut Text, &mut TextColor)>,
    mut rows: Query<&mut Node, With<ImpactRowNode>>,
) {
    if *state != GameState::Settings {
        return;
    }
    if !focus.is_changed()
        && !preset.is_changed()
        && !state.is_changed()
        && !tab.is_changed()
        && !settings.is_changed()
    {
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
    state: Res<GameState>,
    focus: Res<MenuFocus>,
    tab: Res<SettingsTab>,
    settings: Res<GameSettings>,
    mut columns: Query<(&ComputedNode, &Children, &mut ScrollPosition), With<SettingsMenuColumn>>,
    buttons: Query<(&ComputedNode, &MenuButton)>,
) {
    if *state != GameState::Settings {
        return;
    }
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

pub fn update_pause_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    mode: Res<GameMode>,
    players: Res<PlayerControllers>,
    mut menu_focus: ResMut<MenuFocus>,
    overlay: Query<Entity, With<PauseOverlay>>,
) {
    // Also rebuild on PlayerControllers change so a pad disconnect during
    // pause refreshes the "Pad X disconnected" warning.
    if !state.is_changed() && !players.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if *state != GameState::Paused {
        return;
    }
    if state.is_changed() {
        menu_focus.index = 0;
    }
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

pub fn pause_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut menu_focus: ResMut<MenuFocus>,
    mut origin: ResMut<SettingsOrigin>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut players: ResMut<PlayerControllers>,
    mut gtime: ResMut<GameTime>,
    mut tod: ResMut<TimeOfDay>,
    battlefield: Query<Entity, BattlefieldEntity>,
    gamepads: Query<&Gamepad>,
    mouse: Res<MouseUi>,
    keys: Res<ButtonInput<KeyCode>>,
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
                reset_match(
                    &mut commands,
                    &battlefield,
                    &mut gold,
                    &mut placement,
                    &mut players,
                    &mut gtime,
                    &mut tod,
                );
                *state = GameState::Menu;
            }
            _ => {}
        }
    }
}

/// Query filter for everything that belongs to a live match and must be wiped
/// on reset (bases and rocks included, since GameMode may change before the next
/// match). Bundled into one filter so the reset systems stay under Bevy's
/// system-parameter count limit.
type BattlefieldEntity = Or<(
    With<Base>,
    With<Rock>,
    With<Unit>,
    With<Arrow>,
    With<Tower>,
    With<TowerGhost>,
)>;

/// Wipe a finished/abandoned match: despawn every battlefield entity, and reset
/// gold, placement, player→pad mapping and the day/night clock. The arena is
/// rebuilt by `spawn_arena` on the next `Playing` transition. Used by both the
/// pause "Main menu" action and the endgame "Main menu" action.
fn reset_match(
    commands: &mut Commands,
    battlefield: &Query<Entity, BattlefieldEntity>,
    gold: &mut Gold,
    placement: &mut PlacementMode,
    players: &mut PlayerControllers,
    gtime: &mut GameTime,
    tod: &mut TimeOfDay,
) {
    for e in battlefield {
        commands.entity(e).despawn();
    }
    *gold = Gold::default();
    *placement = PlacementMode::default();
    *players = PlayerControllers::default();
    // Restart the day/night clock so the new match opens at the same morning,
    // not wherever the abandoned one left off.
    *gtime = GameTime::default();
    *tod = TimeOfDay::default();
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
}

pub fn update_sideselect_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    mode: Res<GameMode>,
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
    mouse: Res<MouseUi>,
    units: Query<(&PlayerSlot, &UnitKind), With<Unit>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    mut panels: Query<
        (
            &PanelSlot,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Without<PlayerCorner>,
    >,
    mut corners: Query<(&PlayerCorner, &mut BackgroundColor, &mut BorderColor), Without<PanelSlot>>,
) {
    let active = matches!(*state, GameState::Playing | GameState::Paused);
    let mut miners_per_slot = [0usize; 4];
    for (s, k) in &units {
        if *k == UnitKind::Miner {
            miners_per_slot[s.index()] += 1;
        }
    }
    let mut alive = [false; 4];
    for slot in &alive_bases {
        alive[slot.index()] = true;
    }
    for (panel, mut node, mut bg, mut border) in &mut panels {
        let defeated = !alive[panel.slot.index()];
        let hidden =
            panel.index == 4 && miners_per_slot[panel.slot.index()] >= MAX_MINERS_PER_PLAYER;
        let new_display = if hidden { Display::None } else { Display::Flex };
        if node.display != new_display {
            node.display = new_display;
        }
        if hidden {
            continue;
        }
        if defeated {
            bg.0 = BTN_DISABLED;
            *border = BorderColor::all(BORDER_DISABLED);
            continue;
        }
        let focused = active
            && (focuses
                .iter()
                .any(|f| f.slot == panel.slot && f.index == panel.index)
                || mouse.panel_hover == Some((panel.slot, panel.index)));
        bg.0 = if focused { BTN_FOCUSED } else { BTN_NORMAL };
        *border = BorderColor::all(if focused {
            Color::WHITE
        } else {
            panel.slot.side().color()
        });
    }
    let hud_bg = Color::srgba(0.0, 0.0, 0.0, 0.65);
    let hud_border = Color::srgb(0.85, 0.85, 0.9);
    for (corner, mut bg, mut border) in &mut corners {
        if alive[corner.0.index()] {
            bg.0 = hud_bg;
            *border = BorderColor::all(hud_border);
        } else {
            bg.0 = HUD_BG_DISABLED;
            *border = BorderColor::all(HUD_BORDER_DISABLED);
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
    state: Res<GameState>,
    seats: Query<(&SeatSelection, Option<&Name>)>,
    mut texts: Query<(&SideCardLine, &mut Text, &mut TextColor)>,
    mut cards: Query<(&SideCard, &mut BackgroundColor, &mut BorderColor)>,
) {
    if *state != GameState::SideSelect {
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

// ────────────────────────────────────────────────────────────────────────────
// Lifecycle helpers (run on state change)
// ────────────────────────────────────────────────────────────────────────────

/// Auto-pause the match if any active player's gamepad goes missing (the
/// entity disappears from the `Gamepad` query). Without this the abandoned
/// player would silently freeze in place while the other plays on, with no
/// way to recover except for the surviving pad to open the pause menu.
pub fn detect_pad_disconnect(
    mut state: ResMut<GameState>,
    mut players: ResMut<PlayerControllers>,
    mode: Res<GameMode>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    if !matches!(*state, GameState::Playing | GameState::Paused) {
        return;
    }
    let mut any_lost = false;
    for &slot in mode.active_slots() {
        if let Some(entity) = players.get(slot)
            && gamepads.get(entity).is_err()
        {
            players.set(slot, None);
            any_lost = true;
        }
    }
    if any_lost && *state == GameState::Playing {
        *state = GameState::Paused;
    }
}

pub fn manage_input_components(
    mut commands: Commands,
    state: Res<GameState>,
    mode: Res<GameMode>,
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
    for &slot in mode.active_slots() {
        if let Some(pad) = players.get(slot)
            && gamepads.get(pad).is_ok()
        {
            commands.entity(pad).insert(PlayerFocus { slot, index: 0 });
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Input systems (gamepad-only)
// ────────────────────────────────────────────────────────────────────────────

pub fn menu_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut mode: ResMut<GameMode>,
    mut menu_focus: ResMut<MenuFocus>,
    mut origin: ResMut<SettingsOrigin>,
    mut gold: ResMut<Gold>,
    mut placement: ResMut<PlacementMode>,
    mut players: ResMut<PlayerControllers>,
    mut gtime: ResMut<GameTime>,
    mut tod: ResMut<TimeOfDay>,
    battlefield: Query<Entity, BattlefieldEntity>,
    mut exit: MessageWriter<AppExit>,
    gamepads: Query<&Gamepad>,
    mouse: Res<MouseUi>,
) {
    let in_menu = *state == GameState::Menu;
    let in_endgame = matches!(*state, GameState::Ended(_));
    if !in_menu && !in_endgame {
        return;
    }
    if state.is_changed() {
        return;
    }

    let slot_count = if in_menu { 4 } else { 1 };

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

    // Mouse: hover moves focus, left-click activates the hovered item.
    if let Some(i) = mouse.menu_hover.filter(|i| *i < slot_count) {
        menu_focus.index = i;
    }
    if let Some(i) = mouse.menu_click.filter(|i| *i < slot_count) {
        menu_focus.index = i;
        activate = true;
    }

    if !activate {
        return;
    }

    if in_menu {
        match menu_focus.index {
            0 if pad_count > 0 => {
                *mode = GameMode::OneVsOne;
                *state = GameState::SideSelect;
            }
            1 if pad_count > 0 => {
                *mode = GameMode::TwoVsTwo;
                *state = GameState::SideSelect;
            }
            // Debug launch: with no pad connected only the mouse can have fired
            // this activation, and SideSelect (which assigns pads to seats) would
            // be a dead end. Jump straight into a controller-less match so the
            // HUD buttons can be driven by mouse for debugging.
            0 => {
                *mode = GameMode::OneVsOne;
                *state = GameState::Playing;
            }
            1 => {
                *mode = GameMode::TwoVsTwo;
                *state = GameState::Playing;
            }
            2 => {
                *origin = SettingsOrigin::Menu;
                *state = GameState::Settings;
            }
            3 => {
                exit.write(AppExit::Success);
            }
            _ => {}
        }
    } else if in_endgame {
        reset_match(
            &mut commands,
            &battlefield,
            &mut gold,
            &mut placement,
            &mut players,
            &mut gtime,
            &mut tod,
        );
        *state = GameState::Menu;
    }
}

pub fn sideselect_input_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mode: Res<GameMode>,
    mut players: ResMut<PlayerControllers>,
    mut nations: ResMut<PlayerNations>,
    mut seats: Query<(Entity, &Gamepad, Option<&mut SeatSelection>)>,
) {
    if *state != GameState::SideSelect {
        return;
    }
    if state.is_changed() {
        return;
    }

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
            *state = GameState::Playing;
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
    mut state: ResMut<GameState>,
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
    if *state != GameState::Playing {
        return;
    }
    if state.is_changed() {
        return;
    }

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
        *state = GameState::Paused;
    }
}

pub fn placement_system(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    mode: Res<GameMode>,
    lib: Res<MatLibrary>,
    env: Res<EnvAssets>,
    mut placement: ResMut<PlacementMode>,
    mut gold: ResMut<Gold>,
    players: Res<PlayerControllers>,
    gamepads: Query<&Gamepad>,
    ghosts: Query<(Entity, &PlayerSlot), With<TowerGhost>>,
    existing_towers: Query<&Transform, With<Tower>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
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

    let mut alive = [false; 4];
    for slot in &alive_bases {
        alive[slot.index()] = true;
    }

    for &slot in mode.active_slots() {
        // Despawn any existing ghost for this slot (will respawn below if still placing).
        for (e, ghost_slot) in &ghosts {
            if *ghost_slot == slot {
                commands.entity(e).despawn();
            }
        }

        if !alive[slot.index()] {
            placement.clear(slot);
            continue;
        }

        let Some(seat) = placement.get(slot) else {
            continue;
        };

        // Swallow the frame that armed placement so the activating press/click
        // isn't also read as a confirm.
        if !seat.armed {
            placement.set(
                slot,
                PlacementSeat {
                    world_pos: seat.world_pos,
                    armed: true,
                },
            );
            continue;
        }

        let tower_positions: Vec<Vec3> = existing_towers.iter().map(|t| t.translation).collect();

        match players.get(slot) {
            // Gamepad-driven: left stick moves the cursor, A places, B cancels.
            Some(pad_entity) => {
                let Ok(pad) = gamepads.get(pad_entity) else {
                    placement.clear(slot);
                    continue;
                };
                if pad.just_pressed(GamepadButton::East) {
                    placement.clear(slot);
                    continue;
                }
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
                // Stick Y positive = up on screen → -Z in world.
                pos.z -= dz * GAMEPAD_CURSOR_SPEED * dt;
                let confirm = pad.just_pressed(GamepadButton::South);
                place_tower_at(
                    &mut commands,
                    &lib,
                    &env,
                    &mut gold,
                    &mut placement,
                    &tower_positions,
                    slot,
                    *mode,
                    pos,
                    confirm,
                );
            }
            // Mouse-driven (controller-less debug): the ghost tracks the cursor's
            // ground projection, left-click places, right-click cancels.
            None => {
                if mouse_buttons.just_pressed(MouseButton::Right) {
                    placement.clear(slot);
                    continue;
                }
                let pos = windows
                    .single()
                    .ok()
                    .and_then(|w| w.cursor_position())
                    .zip(camera.single().ok())
                    .and_then(|(cursor, (cam, cam_tf))| cursor_ground_pos(cam, cam_tf, cursor))
                    .map(|p| Vec3::new(p.x, 0.0, p.z))
                    .unwrap_or(seat.world_pos);
                let confirm = mouse_buttons.just_pressed(MouseButton::Left);
                place_tower_at(
                    &mut commands,
                    &lib,
                    &env,
                    &mut gold,
                    &mut placement,
                    &tower_positions,
                    slot,
                    *mode,
                    pos,
                    confirm,
                );
            }
        }
    }
}

/// Shared tail of tower placement: validate `pos`, and either spend gold + spawn
/// the tower (when `confirm` and the spot is valid) or (re)spawn the placement
/// ghost tinted by validity. Returns true if a tower was placed. Used by both
/// the gamepad and mouse placement paths in [`placement_system`].
fn place_tower_at(
    commands: &mut Commands,
    lib: &MatLibrary,
    env: &EnvAssets,
    gold: &mut Gold,
    placement: &mut PlacementMode,
    tower_positions: &[Vec3],
    slot: PlayerSlot,
    mode: GameMode,
    pos: Vec3,
    confirm: bool,
) -> bool {
    let valid = is_valid_tower_zone(slot.side(), pos, mode)
        && !collides_with_existing_tower(pos, tower_positions)
        && gold.get(slot) >= TOWER_COST;
    if confirm && valid && gold.try_spend(slot, TOWER_COST) {
        spawn_tower(commands, lib, env, slot, Vec3::new(pos.x, 0.0, pos.z));
        placement.clear(slot);
        return true;
    }
    placement.set(
        slot,
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
        slot,
    ));
    false
}

/// Project a screen-space cursor position onto the world ground plane (y = 0)
/// through `camera`, for mouse tower placement.
fn cursor_ground_pos(camera: &Camera, cam_tf: &GlobalTransform, cursor: Vec2) -> Option<Vec3> {
    let ray = camera.viewport_to_world(cam_tf, cursor).ok()?;
    let dist = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(dist))
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
    match index {
        0 => arm_placement(placement, slot, mode),
        1 if gold.try_spend(slot, SOLDIER_COST) => {
            let count = units
                .iter()
                .filter(|(s, k)| **s == slot && **k == UnitKind::Soldier)
                .count();
            spawn_soldier(commands, models, slot, mode, count % LANE_COUNT);
        }
        2 if gold.try_spend(slot, ARCHER_COST) => {
            let count = units
                .iter()
                .filter(|(s, k)| **s == slot && **k == UnitKind::Archer)
                .count();
            spawn_archer(commands, models, slot, mode, count % LANE_COUNT);
        }
        3 if gold.try_spend(slot, PRIEST_COST) => {
            let count = units
                .iter()
                .filter(|(s, k)| **s == slot && **k == UnitKind::Priest)
                .count();
            spawn_priest(commands, models, slot, mode, count % LANE_COUNT);
        }
        4 => {
            let miner_count = units
                .iter()
                .filter(|(s, k)| **s == slot && **k == UnitKind::Miner)
                .count();
            if miner_count < MAX_MINERS_PER_PLAYER && gold.try_spend(slot, MINER_COST) {
                spawn_miner(commands, models, slot, mode, miner_count);
            }
        }
        _ => {}
    }
}

fn arm_placement(placement: &mut PlacementMode, slot: PlayerSlot, mode: GameMode) {
    placement.set(
        slot,
        PlacementSeat {
            world_pos: default_placement_pos(slot, mode),
            armed: false,
        },
    );
}

fn default_placement_pos(slot: PlayerSlot, mode: GameMode) -> Vec3 {
    let x = match slot.side() {
        Side::Left => (LEFT_BASE_X + TOWER_PLACEMENT_MARGIN - ZONE_BOUNDARY) * 0.5,
        Side::Right => (ZONE_BOUNDARY + RIGHT_BASE_X - TOWER_PLACEMENT_MARGIN) * 0.5,
    };
    Vec3::new(x, 0.0, slot.base_z(mode))
}

pub fn settings_input_system(
    mut state: ResMut<GameState>,
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
    if *state != GameState::Settings {
        return;
    }
    if state.is_changed() {
        return;
    }

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
        *state = origin.to_state();
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
        Some(MenuSlot::Back) | None => *state = origin.to_state(),
    }
}

pub fn apply_graphics_settings(
    settings: Res<GameSettings>,
    atmo: Res<AtmosphereHandle>,
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    mut tonemap: Query<&mut Tonemapping>,
    mut exposures: Query<&mut Exposure, With<Camera3d>>,
    mut sun: Query<&mut DirectionalLight, With<Sun>>,
    mut windows: Query<&mut Window>,
    // Cached copy of the last fully-applied settings. We only touch the camera
    // components whose underlying fields actually moved, instead of reinserting
    // a dozen renderer features on every settings change.
    mut last_applied: Local<Option<GameSettings>>,
) {
    if !settings.is_changed() {
        return;
    }
    let first = last_applied.is_none();
    let prev = last_applied.unwrap_or(*settings);
    let curr = *settings;
    let changed_any = |fields: &[bool]| first || fields.iter().any(|b| *b);

    // Window mode + vsync.
    if first || curr.fullscreen != prev.fullscreen || curr.vsync != prev.vsync {
        let mode = if curr.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
        let present = if curr.vsync {
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
    }
    // Per-camera components. Both Solari (raytracing) and TAA force the
    // deferred renderer, which is incompatible with MSAA — Bevy logs a warning
    // every frame the camera setting changes if we'd insert MSAA anyway. Drop
    // it silently in both cases.
    let msaa_changed = first
        || curr.msaa != prev.msaa
        || curr.raytracing != prev.raytracing
        || curr.taa != prev.taa;
    let msaa = if curr.raytracing || curr.taa {
        Msaa::Off
    } else {
        match curr.msaa {
            2 => Msaa::Sample2,
            4 => Msaa::Sample4,
            8 => Msaa::Sample8,
            _ => Msaa::Off,
        }
    };
    let hdr_changed = first || curr.hdr != prev.hdr;
    let bloom_changed =
        first || curr.bloom != prev.bloom || curr.bloom_intensity != prev.bloom_intensity;
    let atmo_changed = first || curr.atmosphere != prev.atmosphere;
    let vfog_changed = changed_any(&[
        curr.volumetric_fog != prev.volumetric_fog,
        curr.fog_density != prev.fog_density,
    ]);
    let dfog_changed = first || curr.distance_fog != prev.distance_fog;
    let taa_changed = first || curr.taa != prev.taa;
    let fxaa_changed = first || curr.fxaa != prev.fxaa;
    let ssao_changed = first || curr.ssao != prev.ssao || curr.ssao_quality != prev.ssao_quality;
    let mblur_changed = first || curr.motion_blur != prev.motion_blur;
    for cam in &cameras {
        let mut e = commands.entity(cam);
        if msaa_changed {
            e.insert(msaa);
        }
        if hdr_changed {
            if curr.hdr {
                e.insert(bevy::render::view::Hdr);
            } else {
                e.remove::<bevy::render::view::Hdr>();
            }
        }
        if bloom_changed {
            if curr.bloom {
                e.insert(Bloom {
                    intensity: bloom_intensity_value(curr.bloom_intensity),
                    ..Bloom::NATURAL
                });
            } else {
                e.remove::<Bloom>();
            }
        }
        if atmo_changed {
            if curr.atmosphere {
                e.insert((
                    Atmosphere::earthlike(atmo.0.clone()),
                    AtmosphereSettings::default(),
                ));
            } else {
                e.remove::<Atmosphere>().remove::<AtmosphereSettings>();
            }
        }
        if vfog_changed {
            if curr.volumetric_fog {
                e.insert(VolumetricFog {
                    ambient_intensity: fog_density_value(curr.fog_density),
                    ..default()
                });
            } else {
                e.remove::<VolumetricFog>();
            }
        }
        if dfog_changed {
            if curr.distance_fog {
                e.insert(DistanceFog {
                    color: Color::srgba(0.55, 0.70, 0.85, 1.0),
                    falloff: FogFalloff::ExponentialSquared { density: 0.012 },
                    ..default()
                });
            } else {
                e.remove::<DistanceFog>();
            }
        }
        if taa_changed {
            if curr.taa {
                e.insert(TemporalAntiAliasing::default());
            } else {
                e.remove::<TemporalAntiAliasing>();
            }
        }
        if fxaa_changed {
            if curr.fxaa {
                e.insert(Fxaa::default());
            } else {
                e.remove::<Fxaa>();
            }
        }
        if ssao_changed {
            if curr.ssao {
                e.insert(ScreenSpaceAmbientOcclusion {
                    quality_level: match curr.ssao_quality {
                        0 => ScreenSpaceAmbientOcclusionQualityLevel::Low,
                        1 => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                        3 => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
                        _ => ScreenSpaceAmbientOcclusionQualityLevel::High,
                    },
                    ..default()
                });
            } else {
                e.remove::<ScreenSpaceAmbientOcclusion>();
            }
        }
        if mblur_changed {
            if curr.motion_blur {
                e.insert(MotionBlur::default());
            } else {
                e.remove::<MotionBlur>();
            }
        }
    }
    // Tonemapping (mutates existing component on the camera).
    if first || curr.tonemapping != prev.tonemapping {
        for mut t in &mut tonemap {
            *t = match curr.tonemapping {
                0 => Tonemapping::AcesFitted,
                1 => Tonemapping::TonyMcMapface,
                2 => Tonemapping::Reinhard,
                _ => Tonemapping::None,
            };
        }
    }
    // Exposure (HDR sub-parameter; meaningful only when HDR is on but applying
    // is harmless either way).
    if first || curr.exposure != prev.exposure {
        let target_ev100 = exposure_ev100(curr.exposure);
        for mut exp in &mut exposures {
            if (exp.ev100 - target_ev100).abs() > f32::EPSILON {
                exp.ev100 = target_ev100;
            }
        }
    }
    // Sun shadows on/off.
    if first || curr.shadows != prev.shadows {
        for mut light in &mut sun {
            if light.shadows_enabled != curr.shadows {
                light.shadows_enabled = curr.shadows;
            }
        }
    }
    *last_applied = Some(curr);
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
