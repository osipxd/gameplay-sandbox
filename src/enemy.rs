use bevy::{math::StableInterpolate, prelude::*};
use rand::Rng;
use std::f32::consts::TAU;

use crate::camera::GameCamera;
use crate::game_state::RestartGame;
use crate::movement::{self, KinematicBodyBundle, PhysicalTranslation, Velocity};
use crate::player::Player;
use crate::random_source::RandomSource;
use crate::textures::GeneratedTextures;

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

type EnemyVelocityQuery<'w, 's> = Query<
    'w,
    's,
    (&'static PhysicalTranslation, &'static mut Velocity),
    (With<Enemy>, Without<Player>),
>;

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
    mut rng: ResMut<RandomSource>,
    textures: Res<GeneratedTextures>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let now = time.elapsed_secs_f64();

    if now < spawner.next_spawn_at {
        return;
    }

    spawner.next_spawn_at = now + spawner.interval;

    let Some(view_rect) = camera_world_view_rect(camera, camera_transform) else {
        return;
    };

    let direction = Vec2::from_angle(rng.0.random_range(0.0..TAU));
    let spawn_position = spawn_position_from_direction(direction, view_rect, ENEMY_SPAWN_MARGIN);

    commands.spawn((
        Enemy,
        textures.face_sprite(ENEMY_SIZE, ENEMY_BASE_COLOR),
        KinematicBodyBundle::new(spawn_position.extend(crate::ENEMY_Z), Vec2::ZERO),
    ));
}

fn camera_world_view_rect(camera: &Camera, camera_transform: &GlobalTransform) -> Option<Rect> {
    let viewport = camera.logical_viewport_rect()?;
    let top_left = camera
        .viewport_to_world_2d(camera_transform, viewport.min)
        .ok()?;
    let bottom_right = camera
        .viewport_to_world_2d(camera_transform, viewport.max)
        .ok()?;

    Some(Rect {
        min: top_left.min(bottom_right),
        max: top_left.max(bottom_right),
    })
}

fn spawn_position_from_direction(direction: Vec2, view_rect: Rect, margin: f32) -> Vec2 {
    let dir = direction.normalize_or_zero();
    let expanded_view = view_rect.inflate(margin);
    let center = expanded_view.center();
    let half_size = expanded_view.half_size();
    let max_x = half_size.x;
    let max_y = half_size.y;
    let scale_x = if dir.x.abs() > f32::EPSILON {
        max_x / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let scale_y = if dir.y.abs() > f32::EPSILON {
        max_y / dir.y.abs()
    } else {
        f32::INFINITY
    };

    center + dir * scale_x.min(scale_y)
}

pub fn separate_enemies(time: Res<Time>, mut enemies: EnemyVelocityQuery) {
    let dt = time.delta_secs();
    let mut combinations = enemies.iter_combinations_mut();

    while let Some([(transform_a, mut velocity_a), (transform_b, mut velocity_b)]) =
        combinations.fetch_next()
    {
        movement::apply_separation(
            transform_a.0,
            &mut velocity_a,
            transform_b.0,
            &mut velocity_b,
            ENEMY_SEPARATION_DISTANCE,
            ENEMY_SEPARATION_ACCEL,
            dt,
        );
    }
}

pub fn enemy_follow_player(
    time: Res<Time>,
    mut enemies: Query<(&mut Transform, &PhysicalTranslation, &mut Velocity), With<Enemy>>,
    player: Query<&PhysicalTranslation, (With<Player>, Without<Enemy>)>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player.single() else {
        return;
    };

    for (mut enemy_transform, enemy_translation, mut velocity) in &mut enemies {
        let dir = (player_transform.0 - enemy_translation.0).truncate();

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
