use bevy::{math::StableInterpolate, prelude::*};
use rand::Rng;
use std::f32::consts::TAU;

use crate::camera::GameCamera;
use crate::game_state::RestartGame;
use crate::movement::{self, InputVelocity, KinematicBodyBundle, PhysicalTranslation, Velocity};
use crate::player::Player;
use crate::random::{RandomSource, WithJitter};
use crate::textures::GeneratedTextures;

const ENEMY_SPEED: WithJitter<f32> = WithJitter {
    value: 120.0,
    jitter: 12.0,
};
const ENEMY_STEERING_LERP_RATE: WithJitter<f32> = WithJitter {
    value: 6.0,
    jitter: 1.0,
};
const ENEMY_TURN_SPEED: WithJitter<f32> = WithJitter {
    value: 8.75,
    jitter: 1.25,
};
const ENEMY_TARGET_LEAD_FACTOR: WithJitter<f32> = WithJitter {
    value: 0.3,
    jitter: 0.1,
};
const ENEMY_TARGET_JITTER_RADIUS: f32 = 16.0;
const ENEMY_FLANK_DISTANCE: WithJitter<f32> = WithJitter {
    value: 45.0,
    jitter: 15.0,
};
const ENEMY_FLANK_PLAYER_SPEED_THRESHOLD: f32 = 20.0;
const ENEMY_FLANK_MAX_DISTANCE_PCT: f32 = 0.35;
const ENEMY_RETARGET_INTERVAL_SECS: WithJitter<f32> = WithJitter {
    value: 0.17,
    jitter: 0.05,
};
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

type PlayerTargetQuery<'w, 's> = Query<
    'w,
    's,
    (&'static PhysicalTranslation, &'static InputVelocity),
    (With<Player>, Without<Enemy>),
>;

type EnemyFollowQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static PhysicalTranslation,
        &'static mut Velocity,
        &'static mut EnemyTargeting,
    ),
    With<Enemy>,
>;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyTargeting {
    retarget_timer: Timer,
    speed: f32,
    steering_lerp_rate: f32,
    turn_speed: f32,
    lead_factor: f32,
    flank_sign: f32,
    flank_distance: f32,
    target_offset: Vec2,
}

#[derive(Resource)]
pub(crate) struct EnemySpawner {
    next_spawn_at: f64,
    interval: f64,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            next_spawn_at: 0.0,
            interval: 1.0,
        }
    }
}

impl EnemyTargeting {
    fn new(rng: &mut RandomSource) -> Self {
        let mut targeting = Self {
            retarget_timer: Self::retarget_timer(rng),
            speed: ENEMY_SPEED.sample(rng),
            steering_lerp_rate: ENEMY_STEERING_LERP_RATE.sample(rng),
            turn_speed: ENEMY_TURN_SPEED.sample(rng),
            lead_factor: ENEMY_TARGET_LEAD_FACTOR.sample(rng),
            flank_sign: sample_flank_sign(rng),
            flank_distance: ENEMY_FLANK_DISTANCE.sample(rng),
            target_offset: sample_target_offset(rng),
        };
        let initial_elapsed = rng
            .0
            .random_range(0.0..=targeting.retarget_timer.duration().as_secs_f32());
        targeting
            .retarget_timer
            .set_elapsed(std::time::Duration::from_secs_f32(initial_elapsed));
        targeting
    }

    fn tick(&mut self, delta: std::time::Duration, rng: &mut RandomSource) {
        if self.retarget_timer.tick(delta).just_finished() {
            self.retarget(rng);
        }
    }

    fn retarget_timer(rng: &mut RandomSource) -> Timer {
        Timer::from_seconds(ENEMY_RETARGET_INTERVAL_SECS.sample(rng), TimerMode::Once)
    }

    fn retarget(&mut self, rng: &mut RandomSource) {
        self.retarget_timer = Self::retarget_timer(rng);
        self.target_offset = sample_target_offset(rng);
    }

    fn target_position(
        &self,
        player_position: Vec2,
        player_velocity: Vec2,
        enemy_position: Vec2,
    ) -> Vec2 {
        player_position
            + player_velocity * self.lead_factor
            + self.offset(player_position, player_velocity, enemy_position)
    }

    fn offset(&self, player_position: Vec2, player_velocity: Vec2, enemy_position: Vec2) -> Vec2 {
        let enemy_to_player = player_position - enemy_position;
        let max_flank_distance = enemy_to_player.length() * ENEMY_FLANK_MAX_DISTANCE_PCT;
        let flank_distance = self.flank_distance.min(max_flank_distance);

        let flank_offset = if player_velocity.length() > ENEMY_FLANK_PLAYER_SPEED_THRESHOLD {
            let move_direction = player_velocity.normalize();
            let flank_direction = Vec2::new(-move_direction.y, move_direction.x) * self.flank_sign;
            flank_direction * flank_distance
        } else {
            Vec2::ZERO
        };

        flank_offset + self.target_offset
    }
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
        EnemyTargeting::new(&mut rng),
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
    time: Res<Time<Fixed>>,
    mut rng: ResMut<RandomSource>,
    mut enemies: EnemyFollowQuery,
    player: PlayerTargetQuery,
) {
    let dt = time.delta_secs();
    let Ok((player_translation, player_input_velocity)) = player.single() else {
        return;
    };
    let player_position = player_translation.0.truncate();
    let player_velocity = player_input_velocity.0;

    for (mut enemy_transform, enemy_translation, mut velocity, mut targeting) in &mut enemies {
        targeting.tick(time.delta(), &mut rng);
        let enemy_position = enemy_translation.0.truncate();
        let turn_t = (targeting.turn_speed * dt).clamp(0.0, 1.0);
        let target_position =
            targeting.target_position(player_position, player_velocity, enemy_position);
        let dir = target_position - enemy_position;

        if dir.length() > 0.0 {
            let desired = dir.normalize() * targeting.speed;
            let target_rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));

            enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, turn_t);
            velocity
                .0
                .smooth_nudge(&desired, targeting.steering_lerp_rate, dt);
        }
    }
}

fn sample_target_offset(rng: &mut RandomSource) -> Vec2 {
    Vec2::from_angle(rng.0.random_range(0.0..=TAU))
        * rng.0.random_range(0.0..=ENEMY_TARGET_JITTER_RADIUS)
}

fn sample_flank_sign(rng: &mut RandomSource) -> f32 {
    if rng.0.random_bool(0.5) { 1.0 } else { -1.0 }
}
