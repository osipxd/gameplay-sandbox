use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

pub fn apply_separation(
    translation_a: Vec3,
    velocity_a: &mut Velocity,
    translation_b: Vec3,
    velocity_b: &mut Velocity,
    separation_distance: f32,
    separation_accel: f32,
    dt: f32,
) {
    let diff = (translation_a - translation_b).truncate();
    let distance = diff.length();

    if distance > 0.0 && distance < separation_distance {
        let push_dir = diff.normalize();
        let strength = (separation_distance - distance) * separation_accel * dt;

        velocity_a.0 += push_dir * strength;
        velocity_b.0 -= push_dir * strength;
    }
}

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}
