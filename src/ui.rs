use bevy::prelude::*;

use crate::game_state::{GameState, RestartGame, Score};
use crate::player::{Health, Player};

pub(crate) const UI_FONT_PATH: &str = "fonts/FiraSans-Bold.ttf";
const INITIAL_HP_TEXT: &str = "HP: 5";
const INITIAL_SCORE_TEXT: &str = "Score: 0";
const GAME_OVER_TITLE: &str = "Game Over";
const RESTART_BUTTON_LABEL: &str = "Restart";
const HP_LABEL_PREFIX: &str = "HP";
const SCORE_LABEL_PREFIX: &str = "Score";

#[derive(Component)]
pub(crate) struct HpText;

#[derive(Component)]
pub(crate) struct ScoreText;

#[derive(Component)]
pub(crate) struct GameOverOverlay;

#[derive(Component)]
pub(crate) struct RestartButton;

type RestartButtonInteractions<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<RestartButton>),
>;

fn ui_text_font(font: Handle<Font>, font_size: f32) -> TextFont {
    TextFont {
        font,
        font_size,
        ..default()
    }
}

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(UI_FONT_PATH);

    commands.spawn((
        HpText,
        Text::new(INITIAL_HP_TEXT),
        ui_text_font(font.clone(), 30.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        ScoreText,
        Text::new(INITIAL_SCORE_TEXT),
        ui_text_font(font.clone(), 30.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        GameOverOverlay,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        children![
            (
                Text::new(GAME_OVER_TITLE),
                ui_text_font(font.clone(), 64.0),
                TextColor(Color::WHITE),
            ),
            (
                RestartButton,
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(70.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                children![(
                    Text::new(RESTART_BUTTON_LABEL),
                    ui_text_font(font, 32.0),
                    TextColor(Color::WHITE),
                )]
            )
        ],
    ));
}

pub fn update_hp_text(
    player_query: Query<&Health, With<Player>>,
    mut hp_text_query: Query<&mut Text, With<HpText>>,
    game_state: Res<State<GameState>>,
) {
    let hp_label = match game_state.get() {
        GameState::GameOver => format!("{HP_LABEL_PREFIX}: 0"),
        GameState::Playing => {
            if let Ok(health) = player_query.single() {
                format!("{HP_LABEL_PREFIX}: {}", health.0.max(0))
            } else {
                format!("{HP_LABEL_PREFIX}: 0")
            }
        }
    };

    for mut text in &mut hp_text_query {
        text.0 = hp_label.clone();
    }
}

pub fn update_score_text(
    score: Res<Score>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    let score_label = format!("{SCORE_LABEL_PREFIX}: {}", score.0);

    for mut text in &mut score_text_query {
        text.0 = score_label.clone();
    }
}

pub fn update_game_over_overlay(
    game_state: Res<State<GameState>>,
    mut overlay_query: Query<&mut Node, With<GameOverOverlay>>,
) {
    for mut node in &mut overlay_query {
        node.display = if *game_state.get() == GameState::GameOver {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn restart_button_system(
    mut interactions: RestartButtonInteractions,
    mut restart_requests: MessageWriter<RestartGame>,
) {
    for (interaction, mut background_color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgb(0.15, 0.45, 0.15));
                restart_requests.write(RestartGame);
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb(0.25, 0.7, 0.25));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2));
            }
        }
    }
}
