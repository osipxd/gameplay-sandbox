use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}
