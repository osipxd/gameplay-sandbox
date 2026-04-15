use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode, WindowResolution};

mod camera;
mod combat;
mod effects;
mod enemy;
mod game_state;
mod movement;
mod player;
mod random;
mod textures;
mod ui;

use game_state::GameState;

const WIDTH: u32 = 1800;
const HEIGHT: u32 = 1200;
pub(crate) const PARTICLE_Z: f32 = 1.0;
pub(crate) const BULLET_Z: f32 = -2.0;
pub(crate) const ENEMY_Z: f32 = -1.0;
pub(crate) const PLAYER_Z: f32 = 0.0;
pub(crate) const SCORE_POPUP_Z: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window()),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_state::<GameState>()
        .init_resource::<game_state::Score>()
        .init_resource::<enemy::EnemySpawner>()
        .init_resource::<effects::EffectsConfigHandle>()
        .init_resource::<effects::PendingGameOver>()
        .init_asset::<effects::EffectsConfig>()
        .init_asset_loader::<effects::EffectsConfigLoader>()
        .init_resource::<random::RandomSource>()
        .init_resource::<textures::GeneratedTextures>()
        .init_resource::<ui::UiFonts>()
        .add_message::<camera::PlayerHit>()
        .add_message::<effects::EnemyDied>()
        .add_message::<effects::PlayerDied>()
        .add_message::<game_state::RestartGame>()
        .add_systems(
            Startup,
            (
                camera::spawn_camera,
                effects::load_effects_config,
                player::spawn_initial_player,
                ui::spawn_ui,
            ),
        )
        .add_systems(Update, effects::sync_effects_config)
        .add_systems(
            FixedUpdate,
            (
                player::control_player,
                player::shoot_system,
                enemy::spawn_enemies,
                enemy::enemy_follow_player,
                enemy::separate_enemies,
                movement::compose_velocity,
                movement::advance_physics,
                movement::damp_impulses,
                combat::cleanup_bullets,
                combat::bullet_enemy_collision,
                combat::player_enemy_collision,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(effects::effects_config_ready)
                .run_if(effects::death_sequence_inactive),
        )
        .add_systems(
            RunFixedMainLoop,
            movement::interpolate_transforms
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop)
                .run_if(effects::effects_config_ready),
        )
        .add_systems(
            Update,
            (
                player::update_player_visuals,
                ui::update_hp_text,
                ui::update_score_text,
                ui::update_game_over_overlay,
            ),
        )
        .add_systems(
            Update,
            (
                camera::start_screen_shake.after(combat::player_enemy_collision),
                effects::spawn_enemy_death_effects.after(combat::bullet_enemy_collision),
                effects::handle_player_death.after(combat::player_enemy_collision),
                camera::apply_screen_shake,
                effects::update_death_particles,
                effects::update_score_popups,
            )
                .chain()
                .run_if(effects::effects_config_ready),
        )
        .add_systems(Update, effects::finish_game_over_delay)
        .add_systems(
            Update,
            (
                ui::restart_button_system,
                combat::despawn_bullets_on_restart,
                enemy::despawn_enemies_on_restart,
                effects::despawn_effects_on_restart,
                player::restart_player_on_restart,
                enemy::reset_spawner_on_restart,
                game_state::reset_score_on_restart,
                game_state::resume_on_restart,
            )
                .chain()
                .run_if(in_state(GameState::GameOver)),
        )
        .run();
}

fn primary_window() -> Window {
    Window {
        title: "Playground".into(),
        resolution: WindowResolution::new(WIDTH, HEIGHT),
        mode: desktop_window_mode(),
        canvas: Some("#bevy".into()),
        fit_canvas_to_parent: true,
        ..default()
    }
}

fn desktop_window_mode() -> WindowMode {
    if cfg!(target_arch = "wasm32") {
        WindowMode::Windowed
    } else {
        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
    }
}
