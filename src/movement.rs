use bevy::{math::StableInterpolate, prelude::*};

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct InputVelocity(pub Vec2);

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Clone, Copy)]
pub struct Impulse {
    pub value: Vec2,
    pub damping_rate: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PhysicalTranslation(pub Vec3);

#[derive(Component, Debug, Clone, Copy)]
pub struct PreviousPhysicalTranslation(pub Vec3);

#[derive(Bundle)]
pub struct KinematicBodyBundle {
    pub transform: Transform,
    pub physical_translation: PhysicalTranslation,
    pub previous_physical_translation: PreviousPhysicalTranslation,
    pub velocity: Velocity,
}

impl KinematicBodyBundle {
    pub fn new(translation: Vec3, velocity: Vec2) -> Self {
        Self {
            transform: Transform::from_translation(translation),
            physical_translation: PhysicalTranslation(translation),
            previous_physical_translation: PreviousPhysicalTranslation(translation),
            velocity: Velocity(velocity),
        }
    }
}

impl Impulse {
    pub fn new(damping_rate: f32) -> Self {
        Self {
            value: Vec2::ZERO,
            damping_rate,
        }
    }

    pub fn add(&mut self, impulse: Vec2) {
        self.value += impulse;
    }
}

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

pub fn advance_physics(
    time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &Velocity,
    )>,
) {
    for (mut translation, mut previous_translation, velocity) in &mut query {
        previous_translation.0 = translation.0;
        translation.0 += velocity.0.extend(0.0) * time.delta_secs();
    }
}

pub fn compose_velocity(mut query: Query<(&InputVelocity, &Impulse, &mut Velocity)>) {
    for (input_velocity, impulse, mut velocity) in &mut query {
        velocity.0 = input_velocity.0 + impulse.value;
    }
}

pub fn damp_impulses(time: Res<Time<Fixed>>, mut query: Query<&mut Impulse>) {
    let dt = time.delta_secs();

    for mut impulse in &mut query {
        let damping_rate = impulse.damping_rate;
        impulse.value.smooth_nudge(&Vec2::ZERO, damping_rate, dt);
    }
}

pub fn interpolate_transforms(
    time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &PhysicalTranslation,
        &PreviousPhysicalTranslation,
    )>,
) {
    let alpha = time.overstep_fraction();

    for (mut transform, translation, previous_translation) in &mut query {
        transform.translation = previous_translation.0.lerp(translation.0, alpha);
    }
}
