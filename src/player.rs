use bevy::prelude::*;

use crate::combat;
use crate::game_state::RestartGame;
use crate::movement::Velocity;

const PLAYER_SPEED: f32 = 200.0;
pub(crate) const PLAYER_SIZE: f32 = 50.0;
const PLAYER_START_HEALTH: i32 = 5;
const PLAYER_FIRE_RATE: f64 = 0.2;
const BULLET_SPEED: f32 = 400.0;
const BULLET_LIFETIME_SECS: f64 = 1.5;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Weapon {
    fire_rate: f64,
    ready_at: f64,
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite: Sprite,
    transform: Transform,
    velocity: Velocity,
    weapon: Weapon,
    health: Health,
}

impl PlayerBundle {
    fn new() -> Self {
        Self {
            player: Player,
            sprite: Sprite::from_color(
                Color::srgb(0.3, 0.7, 0.9),
                Vec2::new(PLAYER_SIZE, PLAYER_SIZE),
            ),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            velocity: Velocity::default(),
            weapon: Weapon::new(PLAYER_FIRE_RATE),
            health: Health(PLAYER_START_HEALTH),
        }
    }
}

impl Weapon {
    fn new(fire_rate: f64) -> Self {
        Self {
            fire_rate,
            ready_at: 0.0,
        }
    }
}

pub fn spawn_initial_player(mut commands: Commands) {
    spawn_player(&mut commands);
}

pub fn restart_player_on_restart(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    mut restart_requests: MessageReader<RestartGame>,
) {
    if restart_requests.read().next().is_none() {
        return;
    }

    for entity in &players {
        commands.entity(entity).despawn();
    }

    spawn_player(&mut commands);
}

pub fn spawn_player(commands: &mut Commands) {
    commands.spawn(PlayerBundle::new());
}

pub fn control_player(
    mut query: Query<&mut Velocity, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mut velocity in &mut query {
        let mut direction = Vec2::ZERO;

        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }

        velocity.0 = if direction == Vec2::ZERO {
            Vec2::ZERO
        } else {
            direction.normalize() * PLAYER_SPEED
        };
    }
}

pub fn shoot_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Transform, &Velocity, &mut Weapon), With<Player>>,
) {
    let now = time.elapsed_secs_f64();

    for (transform, player_velocity, mut weapon) in &mut query {
        let mut bullet_dir = Vec2::ZERO;

        if keyboard.pressed(KeyCode::ArrowLeft) {
            bullet_dir.x = -1.0;
        } else if keyboard.pressed(KeyCode::ArrowRight) {
            bullet_dir.x = 1.0;
        } else if keyboard.pressed(KeyCode::ArrowUp) {
            bullet_dir.y = 1.0;
        } else if keyboard.pressed(KeyCode::ArrowDown) {
            bullet_dir.y = -1.0;
        }

        if bullet_dir != Vec2::ZERO && now >= weapon.ready_at {
            let bullet_velocity = bullet_dir.normalize() * BULLET_SPEED + player_velocity.0;

            combat::spawn_bullet(
                &mut commands,
                transform.translation,
                bullet_velocity,
                now + BULLET_LIFETIME_SECS,
            );

            weapon.ready_at = now + weapon.fire_rate;
        }
    }
}
