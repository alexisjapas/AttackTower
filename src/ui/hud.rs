//! Persistent in-game HUD: the global clock bar, one corner panel per player
//! (buy buttons, base HP, gold, stats card), their text refreshers and the
//! focus/defeat repaint. Shown/hidden on the InMatch transitions.

use bevy::prelude::*;

use crate::common::*;

use super::*;

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

/// OnEnter(InMatch): reveal the HUD. The top corners only exist in 2v2 (the
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
            // Order matches navigation order (0 → 4, top to bottom).
            spawn_category_header(panel, "Buildings");
            spawn_slot(panel, slot, 0, &format!("Tower ({}g)", TOWER_COST));
            spawn_category_header(panel, "Combat");
            spawn_slot(panel, slot, 1, &unit_slot_label(UnitKind::Soldier));
            spawn_slot(panel, slot, 2, &unit_slot_label(UnitKind::Archer));
            spawn_slot(panel, slot, 3, &unit_slot_label(UnitKind::Priest));
            spawn_category_header(panel, "Resources");
            spawn_slot(panel, slot, 4, &unit_slot_label(UnitKind::Miner));
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

/// HUD button label for a unit kind, from the shared stats table.
fn unit_slot_label(kind: UnitKind) -> String {
    format!("{} ({}g)", kind.label(), kind.stats().cost)
}

/// Stats text for one of the five slot indices, matching the panel button
/// order (0 Tower, 1 Soldier, 2 Archer, 3 Priest, 4 Miner). Unit numbers come
/// from `UnitKind::stats()`; only the kind-specific third value (range, heal,
/// carry capacity) still reads its dedicated constant.
fn focus_stats_string(index: usize) -> String {
    match index {
        0 => format!(
            "Tower\nHP {}  DMG {}  RNG {:.1}  CD {:.1}s",
            TOWER_HP, TOWER_DAMAGE, TOWER_RANGE, TOWER_COOLDOWN
        ),
        1 => {
            let s = UnitKind::Soldier.stats();
            format!(
                "Soldier\nHP {}  DMG {}  SPD {:.1}  CD {:.1}s",
                s.hp, s.damage, s.speed, s.cooldown
            )
        }
        2 => {
            let s = UnitKind::Archer.stats();
            format!(
                "Archer\nHP {}  DMG {}  RNG {:.1}  CD {:.1}s",
                s.hp, s.damage, ARCHER_RANGE, s.cooldown
            )
        }
        3 => {
            let s = UnitKind::Priest.stats();
            format!(
                "Priest\nHP {}  HEAL {}  RNG {:.1}  CD {:.1}s",
                s.hp, PRIEST_HEAL, PRIEST_RANGE, s.cooldown
            )
        }
        4 => {
            let s = UnitKind::Miner.stats();
            format!(
                "Miner\nHP {}  CAP {}  SPD {:.1}  CD {:.1}s",
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

pub fn apply_player_focus_visual(
    state: Res<State<GameState>>,
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
        let (new_bg, new_border) = if defeated {
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
                (BTN_NORMAL, panel.slot.side().color())
            }
        };
        set_bg(&mut bg, new_bg);
        border.set_if_neq(BorderColor::all(new_border));
    }
    let hud_bg = Color::srgba(0.0, 0.0, 0.0, 0.65);
    let hud_border = Color::srgb(0.85, 0.85, 0.9);
    for (corner, mut bg, mut border) in &mut corners {
        let (new_bg, new_border) = if alive[corner.0.index()] {
            (hud_bg, hud_border)
        } else {
            (HUD_BG_DISABLED, HUD_BORDER_DISABLED)
        };
        set_bg(&mut bg, new_bg);
        border.set_if_neq(BorderColor::all(new_border));
    }
}
