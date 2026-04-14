use bevy::{color::Mix, math::StableInterpolate, prelude::*};

use crate::combat;
use crate::game_state::RestartGame;
use crate::movement::{Impulse, InputVelocity, KinematicBodyBundle, PhysicalTranslation};

const PLAYER_SPEED: f32 = 200.0;
pub(crate) const PLAYER_SIZE: f32 = 50.0;
const PLAYER_START_HEALTH: i32 = 5;
const PLAYER_FIRE_RATE_SECS: f32 = 0.3;
pub(crate) const BULLET_SPEED: f32 = 400.0;
const BULLET_LIFETIME_SECS: f32 = 1.5;
const BULLET_PLAYER_VELOCITY_INHERITANCE: f32 = 0.2;
const PLAYER_RECOIL_STRENGTH_PCT: f32 = 0.4;
const PLAYER_IMPULSE_DAMPING_RATE: f32 = 8.0;
const PLAYER_INVINCIBILITY_SECS: f32 = 1.0;
const PLAYER_ROTATION_LERP_RATE: f32 = 14.0;
const PLAYER_SHOOT_SQUASH_SECS: f32 = 0.18;
const PLAYER_SHOOT_SQUASH_X_MIN: f32 = 0.9;
const PLAYER_SHOOT_SQUASH_Y_MAX: f32 = 1.06;
const PLAYER_SHOOT_SQUASH_SHRINK_PCT: f32 = 0.25;
const PLAYER_SHOOT_FLASH_MIX: f32 = 0.45;
pub(crate) const PLAYER_BASE_COLOR: Color = Color::srgb(0.3, 0.7, 0.9);
const PLAYER_INVINCIBLE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const PLAYER_INVINCIBILITY_BLINK_HZ: f64 = 12.0;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Invincibility {
    timer: Timer,
}

#[derive(Component)]
pub struct Weapon {
    cooldown: Timer,
}

#[derive(Component)]
pub(crate) struct ShootSquash {
    timer: Timer,
}

type PlayerShootQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static PhysicalTranslation,
        &'static InputVelocity,
        &'static mut Weapon,
        &'static mut Impulse,
        &'static mut ShootSquash,
    ),
    With<Player>,
>;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite: Sprite,
    body: KinematicBodyBundle,
    input_velocity: InputVelocity,
    weapon: Weapon,
    health: Health,
    invincibility: Invincibility,
    impulse: Impulse,
    shoot_squash: ShootSquash,
}

impl PlayerBundle {
    fn new() -> Self {
        Self {
            player: Player,
            sprite: Sprite::from_color(PLAYER_BASE_COLOR, Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
            body: KinematicBodyBundle::new(Vec3::new(0.0, 0.0, crate::PLAYER_Z), Vec2::ZERO),
            input_velocity: InputVelocity::default(),
            weapon: Weapon::new(PLAYER_FIRE_RATE_SECS),
            health: Health(PLAYER_START_HEALTH),
            invincibility: Invincibility::default(),
            impulse: Impulse::new(PLAYER_IMPULSE_DAMPING_RATE),
            shoot_squash: ShootSquash::new(),
        }
    }
}

impl Weapon {
    fn new(fire_rate_secs: f32) -> Self {
        let mut cooldown = Timer::from_seconds(fire_rate_secs, TimerMode::Once);
        cooldown.set_elapsed(cooldown.duration());
        Self { cooldown }
    }

    fn tick(&mut self, delta: std::time::Duration) {
        self.cooldown.tick(delta);
    }

    fn can_fire(&self) -> bool {
        self.cooldown.is_finished()
    }

    fn trigger(&mut self) {
        self.cooldown.reset();
    }
}

impl Default for Invincibility {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(PLAYER_INVINCIBILITY_SECS, TimerMode::Once);
        timer.set_elapsed(timer.duration());
        Self { timer }
    }
}

impl Invincibility {
    pub fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
    }

    pub fn start(&mut self) {
        self.timer.reset();
    }

    pub fn is_active(&self) -> bool {
        !self.timer.is_finished()
    }

    fn blink_on(&self) -> bool {
        (self.timer.elapsed_secs_f64() * PLAYER_INVINCIBILITY_BLINK_HZ).floor() as i64 % 2 == 0
    }
}

impl ShootSquash {
    fn new() -> Self {
        let mut timer = Timer::from_seconds(PLAYER_SHOOT_SQUASH_SECS, TimerMode::Once);
        timer.set_elapsed(timer.duration());
        Self { timer }
    }

    fn restart(&mut self) {
        self.timer.reset();
    }

    fn scale_at(&self) -> Vec3 {
        if self.timer.is_finished() {
            return Vec3::ONE;
        }

        let progress = self.timer.fraction();
        if progress < PLAYER_SHOOT_SQUASH_SHRINK_PCT {
            let shrink_progress =
                (progress / PLAYER_SHOOT_SQUASH_SHRINK_PCT.max(f32::EPSILON)).clamp(0.0, 1.0);
            Vec3::new(
                1.0_f32.lerp(PLAYER_SHOOT_SQUASH_X_MIN, shrink_progress),
                1.0_f32.lerp(PLAYER_SHOOT_SQUASH_Y_MAX, shrink_progress),
                1.0,
            )
        } else {
            let return_progress = ((progress - PLAYER_SHOOT_SQUASH_SHRINK_PCT)
                / (1.0 - PLAYER_SHOOT_SQUASH_SHRINK_PCT).max(f32::EPSILON))
            .clamp(0.0, 1.0);
            let eased_return = EaseFunction::BackOut.sample_clamped(return_progress);
            Vec3::new(
                PLAYER_SHOOT_SQUASH_X_MIN.lerp(1.0, eased_return),
                PLAYER_SHOOT_SQUASH_Y_MAX.lerp(1.0, eased_return),
                1.0,
            )
        }
    }

    fn flash_mix(&self) -> f32 {
        if self.timer.is_finished() {
            return 0.0;
        }

        PLAYER_SHOOT_FLASH_MIX
            * (1.0 - EaseFunction::QuadraticIn.sample_clamped(self.timer.fraction()))
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
    mut query: Query<&mut InputVelocity, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let direction = movement_input_direction(&keyboard);

    for mut input_velocity in &mut query {
        input_velocity.0 = direction * PLAYER_SPEED;
    }
}

pub fn update_player_visuals(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut players: Query<
        (
            &mut Transform,
            &mut Sprite,
            &InputVelocity,
            &Invincibility,
            &mut ShootSquash,
        ),
        With<Player>,
    >,
) {
    let aim_direction = shooting_input_direction(&keyboard);
    let dt = time.delta_secs();
    for (mut transform, mut sprite, input_velocity, invincibility, mut shoot_squash) in &mut players
    {
        shoot_squash.timer.tick(time.delta());
        transform.scale = shoot_squash.scale_at();
        sprite.color = player_color(invincibility, &shoot_squash);

        let facing_direction = if aim_direction != Vec2::ZERO {
            aim_direction
        } else if input_velocity.0 != Vec2::ZERO {
            input_velocity.0.normalize()
        } else {
            continue;
        };

        let target_rotation = Quat::from_rotation_z(facing_direction.y.atan2(facing_direction.x));
        transform
            .rotation
            .smooth_nudge(&target_rotation, PLAYER_ROTATION_LERP_RATE, dt);
    }
}

pub fn shoot_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    mut player_query: PlayerShootQuery,
) {
    let bullet_dir = shooting_input_direction(&keyboard);

    for (translation, input_velocity, mut weapon, mut impulse, mut shoot_squash) in
        &mut player_query
    {
        weapon.tick(time.delta());

        if bullet_dir != Vec2::ZERO && weapon.can_fire() {
            let bullet_velocity =
                bullet_dir * BULLET_SPEED + input_velocity.0 * BULLET_PLAYER_VELOCITY_INHERITANCE;

            combat::spawn_bullet(
                &mut commands,
                translation.0,
                bullet_velocity,
                BULLET_LIFETIME_SECS,
            );

            impulse.add(-bullet_dir * (BULLET_SPEED * PLAYER_RECOIL_STRENGTH_PCT));
            shoot_squash.restart();
            weapon.trigger();
        }
    }
}

fn player_color(invincibility: &Invincibility, shoot_squash: &ShootSquash) -> Color {
    let base_color = if invincibility.is_active() && invincibility.blink_on() {
        PLAYER_INVINCIBLE_COLOR
    } else {
        PLAYER_BASE_COLOR
    };

    base_color.mix(&Color::WHITE, shoot_squash.flash_mix())
}

fn movement_input_direction(keyboard: &ButtonInput<KeyCode>) -> Vec2 {
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

    direction.normalize_or_zero()
}

fn shooting_input_direction(keyboard: &ButtonInput<KeyCode>) -> Vec2 {
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x = -1.0;
    } else if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x = 1.0;
    } else if keyboard.pressed(KeyCode::ArrowUp) {
        direction.y = 1.0;
    } else if keyboard.pressed(KeyCode::ArrowDown) {
        direction.y = -1.0;
    }

    direction
}
