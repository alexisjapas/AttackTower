//! Persistent in-game HUD: the centered clock chip and one action bar per
//! player (HP/gold pills, five buy cells, stats line, hint line), plus their
//! text refreshers and the focus/defeat repaint. Shown/hidden on the InMatch
//! transitions. Layout rule: every container has a fixed size — state changes
//! only recolor or rewrite text, they never add/remove/resize nodes (see
//! docs/hud-redesign.md). Phase 1 of the redesign fakes the frosted glass
//! with flat translucency; phase 2 swaps the chip background for a real
//! backdrop-blur `UiMaterial`.

use bevy::prelude::*;

use crate::common::*;

use super::*;

// Frosted-glass chip palette.
const CHIP_BG: Color = Color::srgba(0.04, 0.05, 0.08, 0.72);
const CHIP_BORDER: Color = Color::srgba(0.9, 0.9, 0.95, 0.35);
const GOLD_TEXT: Color = Color::srgb(0.95, 0.80, 0.35);
const COST_UNAFFORDABLE: Color = Color::srgb(0.95, 0.35, 0.30);
const COST_DISABLED: Color = Color::srgb(0.55, 0.55, 0.58);
const HINT_COLOR: Color = Color::srgba(0.85, 0.86, 0.92, 0.8);
const STATS_COLOR: Color = Color::srgb(0.92, 0.92, 0.96);
const GAUGE_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);

const CHIP_RADIUS: f32 = 8.0;
const CELL_W: f32 = 76.0;
const CELL_H: f32 = 54.0;
const GAUGE_W: f32 = 70.0;
const GAUGE_H: f32 = 8.0;

// Bevy's default font is an ASCII subset, so the button glyphs are spelled
// out; a proper icon font is a follow-up of the redesign.
const HINT_NORMAL: &str = "D-Pad navigate  (A) buy  (X) tower";
const HINT_PLACING: &str = "Stick move  (A) place  (B) cancel";

#[derive(Component, Clone, Copy)]
pub struct PanelSlot {
    pub slot: PlayerSlot,
    pub index: usize,
}

/// Cost label inside a buy cell; rewritten/recolored by [`update_cell_costs`]
/// (gold vs unaffordable-red vs `MAX` at the miner cap).
#[derive(Component, Clone, Copy)]
pub struct CellCost {
    pub slot: PlayerSlot,
    pub index: usize,
    pub cost: u32,
}

#[derive(Component)]
pub struct GoldText(pub PlayerSlot);

/// Stats line in a player's bar: refreshes whenever that player's focus index
/// changes (showing Tower/Soldier/Archer/Priest/Miner specs).
#[derive(Component)]
pub struct FocusStatsText(pub PlayerSlot);

#[derive(Component)]
pub struct BaseHpText(pub PlayerSlot);

/// Inner fill of the base HP gauge; its width tracks the HP fraction.
#[derive(Component)]
pub struct BaseHpFill(pub PlayerSlot);

/// Button-hint line at the foot of a player's bar; swaps content while that
/// player is placing a tower and empties on defeat.
#[derive(Component)]
pub struct HintText(pub PlayerSlot);

/// HP/gold pill chips; greyed as a whole when the slot's base is destroyed.
#[derive(Component, Clone, Copy)]
pub struct PillChip(pub PlayerSlot);

/// Marker for the top player bars (LeftTop / RightTop). They are hidden in
/// 1v1 by [`show_game_hud`].
#[derive(Component)]
pub struct TopPlayerHud;

/// Marker on the root Node of each player bar (no background of its own —
/// the bar is a loose stack of fixed-size chips).
#[derive(Component, Clone, Copy)]
pub struct PlayerCorner;

#[derive(Component)]
pub struct ClockText;

#[derive(Component)]
pub struct GameHud;

/// Shared look of every HUD chip (translucent dark, thin light border,
/// rounded corners).
fn chip_node(padding: UiRect) -> impl Bundle {
    (
        Node {
            padding,
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            border_radius: BorderRadius::all(Val::Px(CHIP_RADIUS)),
            ..default()
        },
        BackgroundColor(CHIP_BG),
        BorderColor::all(CHIP_BORDER),
    )
}

pub fn setup_ui(mut commands: Commands) {
    // Top clock chip, centered inside an invisible full-width container.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                right: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            GameHud,
        ))
        .with_children(|parent| {
            parent
                .spawn(chip_node(UiRect::axes(Val::Px(20.0), Val::Px(8.0))))
                .with_child((
                    Text::new("06:00"),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::srgb(0.95, 0.93, 0.78)),
                    ClockText,
                ));
        });

    spawn_player_bar(&mut commands, PlayerSlot::LeftBottom);
    spawn_player_bar(&mut commands, PlayerSlot::RightBottom);
    spawn_player_bar(&mut commands, PlayerSlot::LeftTop);
    spawn_player_bar(&mut commands, PlayerSlot::RightTop);
}

fn spawn_player_bar(commands: &mut Commands, slot: PlayerSlot) {
    let side = slot.side();
    let (left, right, align) = match side {
        Side::Left => (Val::Px(12.0), Val::Auto, AlignItems::FlexStart),
        Side::Right => (Val::Auto, Val::Px(12.0), AlignItems::FlexEnd),
    };
    let (top, bottom) = if slot.is_top() {
        (Val::Px(12.0), Val::Auto)
    } else {
        (Val::Auto, Val::Px(12.0))
    };
    // Rows run from the screen edge inward on both sides (Tower outermost),
    // so each player reads their bar from "their" corner.
    let row_dir = match side {
        Side::Left => FlexDirection::Row,
        Side::Right => FlexDirection::RowReverse,
    };

    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top,
            bottom,
            left,
            right,
            flex_direction: FlexDirection::Column,
            align_items: align,
            row_gap: Val::Px(6.0),
            ..default()
        },
        Visibility::Hidden,
        GameHud,
        PlayerCorner,
    ));
    if slot.is_top() {
        root.insert(TopPlayerHud);
    }
    root.with_children(|bar| {
        // Pills row: base HP gauge + gold purse.
        bar.spawn(Node {
            flex_direction: row_dir,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|pills| {
            pills
                .spawn((
                    chip_node(UiRect::axes(Val::Px(12.0), Val::Px(6.0))),
                    PillChip(slot),
                ))
                .with_children(|hp| {
                    hp.spawn((
                        Text::new("HP"),
                        TextFont::from_font_size(14.0),
                        TextColor(side.color()),
                    ));
                    hp.spawn((
                        Text::new(format!("{}/{}", BASE_HP, BASE_HP)),
                        TextFont::from_font_size(16.0),
                        TextColor(Color::WHITE),
                        BaseHpText(slot),
                    ));
                    hp.spawn((
                        Node {
                            width: Val::Px(GAUGE_W),
                            height: Val::Px(GAUGE_H),
                            border_radius: BorderRadius::all(Val::Px(GAUGE_H * 0.5)),
                            ..default()
                        },
                        BackgroundColor(GAUGE_BG),
                    ))
                    .with_child((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(GAUGE_H * 0.5)),
                            ..default()
                        },
                        BackgroundColor(side.color()),
                        BaseHpFill(slot),
                    ));
                });
            pills
                .spawn((
                    chip_node(UiRect::axes(Val::Px(12.0), Val::Px(6.0))),
                    PillChip(slot),
                ))
                .with_child((
                    Text::new("10 g"),
                    TextFont::from_font_size(16.0),
                    TextColor(GOLD_TEXT),
                    GoldText(slot),
                ));
        });
        // Buy cells. Order matches navigation order (0 → 4).
        bar.spawn(Node {
            flex_direction: row_dir,
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|cells| {
            spawn_cell(cells, slot, 0, "Tower", TOWER_COST);
            spawn_cell(
                cells,
                slot,
                1,
                UnitKind::Soldier.label(),
                UnitKind::Soldier.stats().cost,
            );
            spawn_cell(
                cells,
                slot,
                2,
                UnitKind::Archer.label(),
                UnitKind::Archer.stats().cost,
            );
            spawn_cell(
                cells,
                slot,
                3,
                UnitKind::Priest.label(),
                UnitKind::Priest.stats().cost,
            );
            spawn_cell(
                cells,
                slot,
                4,
                UnitKind::Miner.label(),
                UnitKind::Miner.stats().cost,
            );
        });
        // Stats of the focused cell, one reserved line (lit by
        // update_focus_stats_text).
        bar.spawn((
            Node {
                height: Val::Px(16.0),
                ..default()
            },
            Text::new(focus_stats_string(0)),
            TextFont::from_font_size(12.0),
            TextColor(STATS_COLOR),
            FocusStatsText(slot),
        ));
        // Button hints, one reserved line (swapped by update_hint_text).
        bar.spawn((
            Node {
                height: Val::Px(16.0),
                ..default()
            },
            Text::new(HINT_NORMAL),
            TextFont::from_font_size(12.0),
            TextColor(HINT_COLOR),
            HintText(slot),
        ));
    });
}

fn spawn_cell(
    row: &mut ChildSpawnerCommands,
    slot: PlayerSlot,
    index: usize,
    label: &str,
    cost: u32,
) {
    row.spawn((
        Node {
            width: Val::Px(CELL_W),
            height: Val::Px(CELL_H),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(2.0),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(CHIP_RADIUS)),
            ..default()
        },
        BackgroundColor(CHIP_BG),
        BorderColor::all(slot.side().color()),
        PanelSlot { slot, index },
        Button,
    ))
    .with_children(|cell| {
        cell.spawn((
            Text::new(label),
            TextFont::from_font_size(13.0),
            TextColor(Color::WHITE),
        ));
        cell.spawn((
            Text::new(format!("{cost} g")),
            TextFont::from_font_size(13.0),
            TextColor(GOLD_TEXT),
            CellCost { slot, index, cost },
        ));
    });
}

/// OnEnter(InMatch): reveal the HUD. The top bars only exist in 2v2 (the
/// mode is final by the time a match starts).
pub fn show_game_hud(
    mode: Res<GameMode>,
    mut hud: Query<(&mut Visibility, Option<&TopPlayerHud>), With<GameHud>>,
) {
    let two_v_two = *mode == GameMode::TwoVsTwo;
    for (mut vis, top) in &mut hud {
        *vis = if top.is_none() || two_v_two {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// OnExit(InMatch): hide the whole HUD (menus, settings and endgame screens).
pub fn hide_game_hud(mut hud: Query<&mut Visibility, With<GameHud>>) {
    for mut vis in &mut hud {
        *vis = Visibility::Hidden;
    }
}

/// Stats line for one of the five cell indices, matching the bar order
/// (0 Tower, 1 Soldier, 2 Archer, 3 Priest, 4 Miner). Unit numbers come from
/// `UnitKind::stats()`; only the kind-specific third value (range, heal,
/// carry capacity) still reads its dedicated constant.
fn focus_stats_string(index: usize) -> String {
    match index {
        0 => format!(
            "Tower  HP {}  DMG {}  RNG {:.1}  CD {:.1}s",
            TOWER_HP, TOWER_DAMAGE, TOWER_RANGE, TOWER_COOLDOWN
        ),
        1 => {
            let s = UnitKind::Soldier.stats();
            format!(
                "Soldier  HP {}  DMG {}  SPD {:.1}  CD {:.1}s",
                s.hp, s.damage, s.speed, s.cooldown
            )
        }
        2 => {
            let s = UnitKind::Archer.stats();
            format!(
                "Archer  HP {}  DMG {}  RNG {:.1}  CD {:.1}s",
                s.hp, s.damage, ARCHER_RANGE, s.cooldown
            )
        }
        3 => {
            let s = UnitKind::Priest.stats();
            format!(
                "Priest  HP {}  HEAL {}  RNG {:.1}  CD {:.1}s",
                s.hp, PRIEST_HEAL, PRIEST_RANGE, s.cooldown
            )
        }
        4 => {
            let s = UnitKind::Miner.stats();
            format!(
                "Miner  HP {}  CAP {}  SPD {:.1}  CD {:.1}s",
                s.hp, MINER_CAPACITY, s.speed, s.cooldown
            )
        }
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
        let s = focus_stats_string(idx);
        if text.0 != s {
            text.0 = s;
        }
    }
}

pub fn update_gold_text(gold: Res<Gold>, mut texts: Query<(&GoldText, &mut Text)>) {
    if !gold.is_changed() {
        return;
    }
    for (tag, mut text) in &mut texts {
        text.0 = format!("{} g", gold.get(tag.0));
    }
}

/// Rewrites/recolors every cell's cost label: gold when affordable, red when
/// not, grey `MAX` on the miner cell at the cap, grey on defeat. Runs only
/// in-match; compare-before-write keeps idle frames free of Text changes.
pub fn update_cell_costs(
    state: Res<State<GameState>>,
    gold: Res<Gold>,
    units: Query<(&PlayerSlot, &UnitKind), With<Unit>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    mut costs: Query<(&CellCost, &mut Text, &mut TextColor)>,
) {
    let active = matches!(*state.get(), GameState::Playing | GameState::Paused);
    if !active && !state.is_changed() {
        return;
    }
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
    for (cell, mut text, mut color) in &mut costs {
        let capped = cell.index == 4 && miners_per_slot[cell.slot.index()] >= MAX_MINERS_PER_PLAYER;
        let (label, col) = if capped {
            ("MAX".to_string(), COST_DISABLED)
        } else if !alive[cell.slot.index()] {
            (format!("{} g", cell.cost), COST_DISABLED)
        } else if gold.get(cell.slot) < cell.cost {
            (format!("{} g", cell.cost), COST_UNAFFORDABLE)
        } else {
            (format!("{} g", cell.cost), GOLD_TEXT)
        };
        if text.0 != label {
            text.0 = label;
        }
        color.set_if_neq(TextColor(col));
    }
}

/// Swaps each bar's hint line with its player's input mode: normal buy hints,
/// placement hints while that player is placing a tower, empty on defeat.
/// Only runs on a placement change or a base spawn/destruction.
pub fn update_hint_text(
    placement: Res<PlacementMode>,
    spawned: Query<&PlayerSlot, Added<Base>>,
    destroyed: Query<&PlayerSlot, Added<BaseDestroyed>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    mut texts: Query<(&HintText, &mut Text)>,
) {
    if !placement.is_changed() && spawned.is_empty() && destroyed.is_empty() {
        return;
    }
    let mut alive = [false; 4];
    for slot in &alive_bases {
        alive[slot.index()] = true;
    }
    for (tag, mut text) in &mut texts {
        let hint = if !alive[tag.0.index()] {
            ""
        } else if placement.get(tag.0).is_some() {
            HINT_PLACING
        } else {
            HINT_NORMAL
        };
        if text.0 != hint {
            text.0 = hint.to_string();
        }
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
    mut fills: Query<(&BaseHpFill, &mut Node)>,
) {
    // Only refresh when a base's HP actually changed (or a base was spawned).
    if bases.is_empty() {
        return;
    }
    for (tag, mut text) in &mut texts {
        if let Some((_, hp)) = all_bases.iter().find(|(s, _)| **s == tag.0) {
            text.0 = format!("{}/{}", hp.current.max(0), hp.max);
        }
    }
    for (tag, mut node) in &mut fills {
        if let Some((_, hp)) = all_bases.iter().find(|(s, _)| **s == tag.0) {
            node.width = Val::Percent(hp.current.max(0) as f32 / hp.max as f32 * 100.0);
        }
    }
}

pub fn apply_player_focus_visual(
    state: Res<State<GameState>>,
    focuses: Query<&PlayerFocus>,
    mouse: Res<MouseUi>,
    units: Query<(&PlayerSlot, &UnitKind), With<Unit>>,
    alive_bases: Query<&PlayerSlot, (With<Base>, Without<BaseDestroyed>)>,
    mut panels: Query<(&PanelSlot, &mut BackgroundColor, &mut BorderColor), Without<PillChip>>,
    mut pills: Query<(&PillChip, &mut BackgroundColor, &mut BorderColor), Without<PanelSlot>>,
) {
    let active = matches!(*state.get(), GameState::Playing | GameState::Paused);
    // Outside a match the HUD is hidden, so repainting it is pure waste; one
    // extra pass on the transition frame leaves everything in the idle state.
    if !active && !state.is_changed() {
        return;
    }
    // Compare-before-write below: unconditional writes would flag every panel
    // changed every frame and re-extract the whole HUD for nothing.
    let set_bg = |bg: &mut BackgroundColor, c: Color| {
        if bg.0 != c {
            bg.0 = c;
        }
    };
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
    for (panel, mut bg, mut border) in &mut panels {
        let defeated = !alive[panel.slot.index()];
        let capped =
            panel.index == 4 && miners_per_slot[panel.slot.index()] >= MAX_MINERS_PER_PLAYER;
        // The miner cell at cap greys out in place — it is never removed, so
        // the bar's layout stays fixed (focus already skips it in input).
        let (new_bg, new_border) = if defeated || capped {
            (BTN_DISABLED, BORDER_DISABLED)
        } else {
            let focused = active
                && (focuses
                    .iter()
                    .any(|f| f.slot == panel.slot && f.index == panel.index)
                    || mouse.panel_hover == Some((panel.slot, panel.index)));
            if focused {
                (BTN_FOCUSED, Color::WHITE)
            } else {
                (CHIP_BG, panel.slot.side().color())
            }
        };
        set_bg(&mut bg, new_bg);
        border.set_if_neq(BorderColor::all(new_border));
    }
    for (pill, mut bg, mut border) in &mut pills {
        let (new_bg, new_border) = if alive[pill.0.index()] {
            (CHIP_BG, CHIP_BORDER)
        } else {
            (HUD_BG_DISABLED, HUD_BORDER_DISABLED)
        };
        set_bg(&mut bg, new_bg);
        border.set_if_neq(BorderColor::all(new_border));
    }
}
