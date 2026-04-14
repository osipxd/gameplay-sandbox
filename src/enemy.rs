use bevy::window::PrimaryWindow;
use bevy::{math::StableInterpolate, prelude::*};

use crate::game_state::RestartGame;
use crate::movement::{self, Velocity};
use crate::player::Player;

const ENEMY_SPEED: f32 = 120.0;
const ENEMY_STEERING_LERP_RATE: f32 = 6.0;
const ENEMY_ROTATION_LERP_RATE: f32 = 12.0;
const ENEMY_SEPARATION_ACCEL: f32 = 240.0;
pub(crate) const ENEMY_SIZE: f32 = 40.0;
pub(crate) const ENEMY_BASE_COLOR: Color = Color::srgb(0.9, 0.2, 0.3);
const ENEMY_SEPARATION_GAP: f32 = 10.0;
const ENEMY_SEPARATION_DISTANCE: f32 = ENEMY_SIZE + ENEMY_SEPARATION_GAP;
const ENEMY_SPAWN_GAP: f32 = 30.0;
const ENEMY_SPAWN_MARGIN: f32 = ENEMY_SIZE * 0.5 + ENEMY_SPAWN_GAP;

type EnemyVelocityQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static mut Velocity), (With<Enemy>, Without<Player>)>;

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
pub(crate) struct EnemySpawner {
    next_spawn_at: f64,
    interval: f64,
}

pub fn setup_enemy_spawner(mut commands: Commands) {
    commands.insert_resource(EnemySpawner {
        next_spawn_at: 0.0,
        interval: 1.0,
    });
}

pub fn despawn_enemies_on_restart(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut restart_requests: MessageReader<RestartGame>,
) {
    if restart_requests.read().next().is_none() {
        return;
    }

    for entity in &enemies {
        commands.entity(entity).despawn();
    }
}

pub fn reset_spawner_on_restart(
    mut spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
    mut restart_requests: MessageReader<RestartGame>,
) {
    if restart_requests.read().next().is_none() {
        return;
    }

    spawner.next_spawn_at = time.elapsed_secs_f64() + spawner.interval;
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let now = time.elapsed_secs_f64();

    if now < spawner.next_spawn_at {
        return;
    }

    spawner.next_spawn_at = now + spawner.interval;

    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let t = now as f32;
    let side = t as i32 % 4;

    let (x, y) = match side {
        0 => {
            let x = (t * 1.3).sin() * half_width;
            (x, half_height + ENEMY_SPAWN_MARGIN)
        }
        1 => {
            let x = (t * 1.7).sin() * half_width;
            (x, -half_height - ENEMY_SPAWN_MARGIN)
        }
        2 => {
            let y = (t * 1.5).cos() * half_height;
            (-half_width - ENEMY_SPAWN_MARGIN, y)
        }
        _ => {
            let y = (t * 1.9).cos() * half_height;
            (half_width + ENEMY_SPAWN_MARGIN, y)
        }
    };

    commands.spawn((
        Enemy,
        Sprite::from_color(ENEMY_BASE_COLOR, Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
        Transform::from_xyz(x, y, crate::ENEMY_Z),
        Velocity(Vec2::ZERO),
    ));
}

pub fn separate_enemies(time: Res<Time>, mut enemies: EnemyVelocityQuery) {
    let dt = time.delta_secs();
    let mut combinations = enemies.iter_combinations_mut();

    while let Some([(transform_a, mut velocity_a), (transform_b, mut velocity_b)]) =
        combinations.fetch_next()
    {
        movement::apply_separation(
            transform_a.translation,
            &mut velocity_a,
            transform_b.translation,
            &mut velocity_b,
            ENEMY_SEPARATION_DISTANCE,
            ENEMY_SEPARATION_ACCEL,
            dt,
        );
    }
}

pub fn enemy_follow_player(
    time: Res<Time>,
    mut enemies: Query<(&mut Transform, &mut Velocity), With<Enemy>>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player.single() else {
        return;
    };

    for (mut enemy_transform, mut velocity) in &mut enemies {
        let dir = (player_transform.translation - enemy_transform.translation).truncate();

        if dir.length() > 0.0 {
            let desired = dir.normalize() * ENEMY_SPEED;
            let target_rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));

            enemy_transform
                .rotation
                .smooth_nudge(&target_rotation, ENEMY_ROTATION_LERP_RATE, dt);
            velocity
                .0
                .smooth_nudge(&desired, ENEMY_STEERING_LERP_RATE, dt);
        }
    }
}
