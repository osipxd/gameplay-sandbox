use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::game_state::{GameState, RestartGame, Score};
use crate::player::{Health, Player};

pub(crate) const UI_FONT_PATH: &str = "fonts/FiraSans-Bold.ttf";
const INITIAL_HP_TEXT: &str = "HP: 5";
const INITIAL_SCORE_TEXT: &str = "Score: 0";
const GAME_OVER_TITLE: &str = "Game Over";
const RESTART_BUTTON_LABEL: &str = "Restart";
const HP_LABEL_PREFIX: &str = "HP";
const SCORE_LABEL_PREFIX: &str = "Score";
const VIGNETTE_TEXTURE_SIZE: u32 = 256;
const VIGNETTE_BORDER_PX: f32 = 40.0;
const VIGNETTE_INNER_RADIUS: f32 = 0.55;
const VIGNETTE_MAX_ALPHA: f32 = 0.14;

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

pub fn spawn_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let font = asset_server.load(UI_FONT_PATH);
    spawn_vignette_overlay(&mut commands, &mut images);

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

fn create_vignette_image() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: VIGNETTE_TEXTURE_SIZE,
            height: VIGNETTE_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let texture_size = VIGNETTE_TEXTURE_SIZE as f32;
    let half_diagonal = 2.0_f32.sqrt();

    for y in 0..VIGNETTE_TEXTURE_SIZE {
        for x in 0..VIGNETTE_TEXTURE_SIZE {
            let normalized_x = ((x as f32 + 0.5) / texture_size) * 2.0 - 1.0;
            let normalized_y = ((y as f32 + 0.5) / texture_size) * 2.0 - 1.0;
            let distance = Vec2::new(normalized_x, normalized_y).length() / half_diagonal;
            let edge_factor = ((distance - VIGNETTE_INNER_RADIUS) / (1.0 - VIGNETTE_INNER_RADIUS))
                .clamp(0.0, 1.0);
            let eased_edge = EaseFunction::SmootherStep.sample_clamped(edge_factor);
            let alpha = eased_edge * VIGNETTE_MAX_ALPHA;

            if let Some(pixel) = image.pixel_bytes_mut(UVec3::new(x, y, 0)) {
                pixel[3] = (alpha * u8::MAX as f32) as u8;
            }
        }
    }

    image
}

fn spawn_vignette_overlay(commands: &mut Commands, images: &mut Assets<Image>) {
    let vignette = images.add(create_vignette_image());

    commands.spawn((
        ImageNode::new(vignette).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(VIGNETTE_BORDER_PX),
            ..default()
        })),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
    ));
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
