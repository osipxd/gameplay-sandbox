use bevy::prelude::*;
use bevy::window::WindowResolution;

mod combat;
mod enemy;
mod game_state;
mod movement;
mod player;
mod ui;

use game_state::GameState;

const WIDTH: u32 = 1800;
const HEIGHT: u32 = 1200;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Playground".into(),
                resolution: WindowResolution::new(WIDTH, HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_message::<game_state::RestartGame>()
        .add_systems(
            Startup,
            (
                spawn_camera,
                player::spawn_initial_player,
                enemy::setup_enemy_spawner,
                ui::spawn_ui,
            ),
        )
        .add_systems(
            Update,
            (
                player::control_player,
                player::shoot_system,
                enemy::spawn_enemies,
                enemy::enemy_follow_player,
                enemy::separate_enemies,
                movement::apply_velocity,
                combat::cleanup_bullets,
                combat::bullet_enemy_collision,
                combat::player_enemy_collision,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, (ui::update_hp_text, ui::update_game_over_overlay))
        .add_systems(
            Update,
            (
                ui::restart_button_system,
                combat::despawn_bullets_on_restart,
                enemy::despawn_enemies_on_restart,
                player::restart_player_on_restart,
                enemy::reset_spawner_on_restart,
                game_state::resume_on_restart,
            )
                .chain()
                .run_if(in_state(GameState::GameOver)),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
