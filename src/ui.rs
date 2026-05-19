use bevy::prelude::*;

use crate::common::*;
use crate::units::{spawn_archer, spawn_miner, spawn_soldier};

#[derive(Component)]
pub struct BuyButton {
    pub side: Side,
    pub kind: UnitKind,
}

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct GoldText(pub Side);

#[derive(Component)]
pub struct BaseHpText(pub Side);

#[derive(Component)]
pub struct EndgameOverlay;

#[derive(Component)]
pub struct WinnerText;

const BTN_NORMAL: Color = Color::srgb(0.16, 0.16, 0.20);
const BTN_HOVERED: Color = Color::srgb(0.25, 0.25, 0.30);
const BTN_PRESSED: Color = Color::srgb(0.10, 0.10, 0.14);

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
            spawn_buy_button(panel, side, UnitKind::Soldier, "Soldier", SOLDIER_COST);
            spawn_buy_button(panel, side, UnitKind::Miner, "Miner", MINER_COST);
            spawn_buy_button(panel, side, UnitKind::Archer, "Archer", ARCHER_COST);
            panel.spawn((
                Text::new("Gold: 10"),
                TextFont::from_font_size(18.0),
                TextColor(side.color()),
                GoldText(side),
            ));
        });
}

fn spawn_buy_button(
    panel: &mut ChildSpawnerCommands,
    side: Side,
    kind: UnitKind,
    label: &str,
    cost: u32,
) {
    panel
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
            BorderColor::all(side.color()),
            BuyButton { side, kind },
        ))
        .with_child((
            Text::new(format!("{} ({}g)", label, cost)),
            TextFont::from_font_size(15.0),
            TextColor(Color::WHITE),
        ));
}

pub fn buy_button_system(
    mut commands: Commands,
    state: Res<GameState>,
    lib: Res<MatLibrary>,
    mut gold: ResMut<Gold>,
    mut interactions: Query<(&Interaction, &BuyButton, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, buy, mut bg) in interactions.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                bg.0 = BTN_PRESSED;
                if *state != GameState::Playing {
                    continue;
                }
                let cost = match buy.kind {
                    UnitKind::Soldier => SOLDIER_COST,
                    UnitKind::Miner => MINER_COST,
                    UnitKind::Archer => ARCHER_COST,
                };
                if gold.try_spend(buy.side, cost) {
                    match buy.kind {
                        UnitKind::Soldier => spawn_soldier(&mut commands, &lib, buy.side),
                        UnitKind::Miner => spawn_miner(&mut commands, &lib, buy.side),
                        UnitKind::Archer => spawn_archer(&mut commands, &lib, buy.side),
                    }
                }
            }
            Interaction::Hovered => bg.0 = BTN_HOVERED,
            Interaction::None => bg.0 = BTN_NORMAL,
        }
    }
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

pub fn update_endgame_overlay(
    mut commands: Commands,
    state: Res<GameState>,
    overlay: Query<Entity, With<EndgameOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if let GameState::Ended(winner) = *state {
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
                    WinnerText,
                ));
                parent
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(BTN_NORMAL),
                        BorderColor::all(Color::WHITE),
                        RestartButton,
                    ))
                    .with_child((
                        Text::new("Restart"),
                        TextFont::from_font_size(22.0),
                        TextColor(Color::WHITE),
                    ));
            });
    }
}

pub fn restart_button_system(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut gold: ResMut<Gold>,
    units: Query<Entity, With<Unit>>,
    arrows: Query<Entity, With<Arrow>>,
    mut bases: Query<&mut Health, With<Base>>,
) {
    for (interaction, mut bg) in interactions.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                bg.0 = BTN_PRESSED;
                for entity in &units {
                    commands.entity(entity).despawn();
                }
                for entity in &arrows {
                    commands.entity(entity).despawn();
                }
                for mut hp in bases.iter_mut() {
                    hp.current = hp.max;
                }
                *gold = Gold::default();
                *state = GameState::Playing;
            }
            Interaction::Hovered => bg.0 = BTN_HOVERED,
            Interaction::None => bg.0 = BTN_NORMAL,
        }
    }
}
