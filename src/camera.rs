use bevy::prelude::*;
use std::f32::consts::TAU;

const SCREEN_SHAKE_TRAUMA_PER_HIT: f32 = 0.7;
const SCREEN_SHAKE_DECAY_PER_SEC: f32 = 4.0;
const SCREEN_SHAKE_MAX_OFFSET: f32 = 14.0;
const SCREEN_SHAKE_X_HZ: f32 = 8.0;
const SCREEN_SHAKE_Y_HZ: f32 = 6.0;

#[derive(Message)]
pub struct PlayerHit;

#[derive(Component)]
pub(crate) struct GameCamera;

#[derive(Component, Default)]
pub(crate) struct ScreenShake {
    trauma: f32,
    elapsed: f32,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera, ScreenShake::default()));
}

pub fn start_screen_shake(
    mut hit_events: MessageReader<PlayerHit>,
    mut camera_query: Query<&mut ScreenShake, With<GameCamera>>,
) {
    let hit_count = hit_events.read().count() as f32;

    if hit_count == 0.0 {
        return;
    }

    let Ok(mut shake) = camera_query.single_mut() else {
        return;
    };

    shake.trauma = (shake.trauma + hit_count * SCREEN_SHAKE_TRAUMA_PER_HIT).min(1.0);
    shake.elapsed = 0.0;
}

pub fn apply_screen_shake(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut ScreenShake), With<GameCamera>>,
) {
    let Ok((mut transform, mut shake)) = camera_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - SCREEN_SHAKE_DECAY_PER_SEC * dt).max(0.0);

    if shake.trauma <= 0.0 {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
        shake.elapsed = 0.0;
        return;
    }

    shake.elapsed += dt;
    let intensity = shake.trauma;
    let t = shake.elapsed * TAU;

    transform.translation.x = (t * SCREEN_SHAKE_X_HZ).cos() * SCREEN_SHAKE_MAX_OFFSET * intensity;
    transform.translation.y =
        (t * SCREEN_SHAKE_Y_HZ + 1.7).sin() * SCREEN_SHAKE_MAX_OFFSET * intensity;
}
