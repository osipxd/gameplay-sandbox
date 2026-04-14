use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::camera::PlayerHit;
use crate::effects::{EnemyDied, PlayerDied};
use crate::enemy::{self, Enemy};
use crate::game_state::{RestartGame, Score};
use crate::movement::Velocity;
use crate::player::{self, Health, Invincibility, Player};

const BULLET_SIZE: f32 = 12.0;
const BULLET_ENEMY_HITBOX_SCALE: f32 = 0.96;
const PLAYER_ENEMY_HITBOX_SCALE: f32 = 0.9;
const PLAYER_ENEMY_SEPARATION_ACCEL: f32 = 120.0;
const PLAYER_ENEMY_SEPARATION_OVERLAP: f32 = 8.0;
const PLAYER_CONTACT_PUSH_RATE: f32 = 12.0;
const PLAYER_HIT_KNOCKBACK_SPEED: f32 = 260.0;
const PLAYER_HIT_PUSH_DISTANCE: f32 = 20.0;

#[derive(Component)]
pub(crate) struct Bullet;

#[derive(Component)]
pub(crate) struct BulletLifetime(Timer);

type PlayerCollisionQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static mut Health,
        &'static mut Invincibility,
    ),
    (With<Player>, Without<Enemy>),
>;

type EnemyCollisionQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static mut Velocity), (With<Enemy>, Without<Player>)>;

fn bullet_enemy_hit_distance() -> f32 {
    (BULLET_SIZE + enemy::ENEMY_SIZE) * 0.5 * BULLET_ENEMY_HITBOX_SCALE
}

fn player_enemy_hit_distance() -> f32 {
    (player::PLAYER_SIZE + enemy::ENEMY_SIZE) * 0.5 * PLAYER_ENEMY_HITBOX_SCALE
}

fn player_enemy_separation_distance() -> f32 {
    (player::PLAYER_SIZE + enemy::ENEMY_SIZE) * 0.5 - PLAYER_ENEMY_SEPARATION_OVERLAP
}

pub fn spawn_bullet(
    commands: &mut Commands,
    translation: Vec3,
    velocity: Vec2,
    lifetime_secs: f32,
) {
    let mut bullet_translation = translation;
    bullet_translation.z = crate::BULLET_Z;
    let bullet_rotation = if velocity == Vec2::ZERO {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_z(velocity.y.atan2(velocity.x))
    };

    commands.spawn((
        Bullet,
        Sprite::from_color(
            Color::srgb(1.0, 0.9, 0.2),
            Vec2::new(BULLET_SIZE, BULLET_SIZE),
        ),
        Transform {
            translation: bullet_translation,
            rotation: bullet_rotation,
            ..default()
        },
        Velocity(velocity),
        BulletLifetime(Timer::from_seconds(lifetime_secs, TimerMode::Once)),
    ));
}

pub fn despawn_bullets_on_restart(
    mut commands: Commands,
    bullets: Query<Entity, With<Bullet>>,
    mut restart_requests: MessageReader<RestartGame>,
) {
    if restart_requests.read().next().is_none() {
        return;
    }

    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}

pub fn player_enemy_collision(
    mut commands: Commands,
    time: Res<Time>,
    mut player_query: PlayerCollisionQuery,
    mut enemies: EnemyCollisionQuery,
    mut player_hit_events: MessageWriter<PlayerHit>,
    mut player_died_events: MessageWriter<PlayerDied>,
) {
    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();
    let Ok((player_entity, mut player_transform, mut health, mut invincibility)) =
        player_query.single_mut()
    else {
        return;
    };

    let mut can_take_damage = now >= invincibility.until;
    let separation_distance = player_enemy_separation_distance();

    for (enemy_transform, mut enemy_velocity) in &mut enemies {
        let push_dir = (enemy_transform.translation - player_transform.translation).truncate();
        let distance = push_dir.length();

        if distance > 0.0 && distance < separation_distance {
            let push_normal = push_dir.normalize();
            let overlap = separation_distance - distance;
            let enemy_push = overlap * PLAYER_ENEMY_SEPARATION_ACCEL * dt;
            let player_push = overlap * PLAYER_CONTACT_PUSH_RATE * dt;

            enemy_velocity.0 += push_normal * enemy_push;
            player_transform.translation -= push_normal.extend(0.0) * player_push;
        }

        if can_take_damage && distance < player_enemy_hit_distance() {
            let push_normal = push_dir.normalize();
            health.0 -= 1;
            player_hit_events.write(PlayerHit);

            if health.0 <= 0 {
                health.0 = 0;
                player_died_events.write(PlayerDied {
                    translation: player_transform.translation,
                    burst_direction: Vec2::ZERO,
                });
                commands.entity(player_entity).despawn();
            } else {
                invincibility.until = player::invincibility_until(now);
            }

            enemy_velocity.0 += push_normal * PLAYER_HIT_KNOCKBACK_SPEED;
            player_transform.translation -= push_normal.extend(0.0) * PLAYER_HIT_PUSH_DISTANCE;

            can_take_damage = false;

            if health.0 <= 0 {
                break;
            }
        }
    }
}

pub fn bullet_enemy_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Velocity), With<Bullet>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut enemy_died_events: MessageWriter<EnemyDied>,
    mut score: ResMut<Score>,
) {
    let mut hit_bullets = HashSet::new();
    let mut hit_enemies = HashSet::new();
    let mut enemy_burst_directions = HashMap::new();

    for (bullet_entity, bullet_transform, bullet_velocity) in &bullets {
        if hit_bullets.contains(&bullet_entity) {
            continue;
        }

        for (enemy_entity, enemy_transform) in &enemies {
            if hit_enemies.contains(&enemy_entity) {
                continue;
            }

            let distance = bullet_transform
                .translation
                .truncate()
                .distance(enemy_transform.translation.truncate());

            if distance < bullet_enemy_hit_distance() {
                hit_bullets.insert(bullet_entity);
                hit_enemies.insert(enemy_entity);
                enemy_burst_directions.insert(enemy_entity, bullet_velocity.0.normalize_or_zero());
                break;
            }
        }
    }

    for bullet_entity in hit_bullets {
        commands.entity(bullet_entity).despawn();
    }

    for enemy_entity in hit_enemies {
        if let Ok((_, enemy_transform)) = enemies.get(enemy_entity) {
            enemy_died_events.write(EnemyDied {
                translation: enemy_transform.translation,
                burst_direction: enemy_burst_directions
                    .get(&enemy_entity)
                    .copied()
                    .unwrap_or(Vec2::ZERO),
            });
        }
        commands.entity(enemy_entity).despawn();
        score.0 += 1;
    }
}

pub fn cleanup_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut BulletLifetime)>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.0.tick(time.delta());
        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
