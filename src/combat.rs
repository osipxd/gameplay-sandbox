use bevy::prelude::*;
use std::collections::HashSet;

use crate::enemy::{self, Enemy};
use crate::game_state::{GameState, RestartGame, Score};
use crate::movement::Velocity;
use crate::player::{self, Health, Player};

const BULLET_SIZE: f32 = 12.0;
const BULLET_ENEMY_HITBOX_SCALE: f32 = 0.96;
const PLAYER_ENEMY_HITBOX_SCALE: f32 = 0.9;

#[derive(Component)]
pub(crate) struct Bullet;

#[derive(Component)]
pub(crate) struct DespawnAt(f64);

fn bullet_enemy_hit_distance() -> f32 {
    (BULLET_SIZE + enemy::ENEMY_SIZE) * 0.5 * BULLET_ENEMY_HITBOX_SCALE
}

fn player_enemy_hit_distance() -> f32 {
    (player::PLAYER_SIZE + enemy::ENEMY_SIZE) * 0.5 * PLAYER_ENEMY_HITBOX_SCALE
}

pub fn spawn_bullet(
    commands: &mut Commands,
    translation: Vec3,
    velocity: Vec2,
    despawn_at: f64,
) {
    commands.spawn((
        Bullet,
        Sprite::from_color(
            Color::srgb(1.0, 0.9, 0.2),
            Vec2::new(BULLET_SIZE, BULLET_SIZE),
        ),
        Transform::from_translation(translation),
        Velocity(velocity),
        DespawnAt(despawn_at),
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
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_transform, mut health)) = player_query.single_mut() else {
        return;
    };

    for (enemy_entity, enemy_transform) in &enemies {
        let distance = player_transform
            .translation
            .truncate()
            .distance(enemy_transform.translation.truncate());

        if distance < player_enemy_hit_distance() {
            health.0 -= 1;

            if health.0 <= 0 {
                health.0 = 0;
                next_state.set(GameState::GameOver);
                break;
            }

            commands.entity(enemy_entity).despawn();
        }
    }
}

pub fn bullet_enemy_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut score: ResMut<Score>,
) {
    let mut hit_bullets = HashSet::new();
    let mut hit_enemies = HashSet::new();

    for (bullet_entity, bullet_transform) in &bullets {
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
                break;
            }
        }
    }

    for bullet_entity in hit_bullets {
        commands.entity(bullet_entity).despawn();
    }

    for enemy_entity in hit_enemies {
        commands.entity(enemy_entity).despawn();
        score.0 += 1;
    }
}

pub fn cleanup_bullets(
    mut commands: Commands,
    time: Res<Time>,
    bullets: Query<(Entity, &DespawnAt)>,
) {
    let now = time.elapsed_secs_f64();

    for (entity, despawn_at) in &bullets {
        if now >= despawn_at.0 {
            commands.entity(entity).despawn();
        }
    }
}
