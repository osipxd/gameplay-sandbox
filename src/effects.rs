use bevy::color::Alpha;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::f32::consts::TAU;

use crate::enemy;
use crate::game_state::{GameState, RestartGame};
use crate::player;
use crate::ui;

const CORE_FRAGMENT_GRID_SIZE: usize = 3;
const CORE_FRAGMENT_COUNT: usize = CORE_FRAGMENT_GRID_SIZE * CORE_FRAGMENT_GRID_SIZE;
const EXTRA_PARTICLE_COUNT_MIN: usize = 12;
const EXTRA_PARTICLE_COUNT_MAX: usize = 18;
const EXTRA_PARTICLE_SIZE_MIN_FACTOR: f32 = 0.35;
const EXTRA_PARTICLE_SIZE_MAX_FACTOR: f32 = 0.65;
const EXTRA_PARTICLE_SPAWN_SIDE_FACTOR: f32 = 0.5;
const PARTICLE_SPEED_MIN: f32 = 300.0;
const PARTICLE_SPEED_MAX: f32 = 550.0;
const LARGE_PARTICLE_SPEED_SCALE: f32 = 0.82;
const SMALL_PARTICLE_SPEED_SCALE: f32 = LARGE_PARTICLE_SPEED_SCALE * 1.4;
const PARTICLE_TTL_MIN: f32 = 0.28;
const PARTICLE_TTL_MAX: f32 = 0.34;
const LARGE_PARTICLE_TTL_SCALE: f32 = 1.12;
const SMALL_PARTICLE_TTL_SCALE: f32 = LARGE_PARTICLE_TTL_SCALE * 0.7;
const LARGE_PARTICLE_BRIGHTNESS: f32 = 1.15;
const SMALL_PARTICLE_BRIGHTNESS: f32 = 1.35;
const PARTICLE_FADE_SECS: f32 = 0.08;
const PARTICLE_LIFETIME_SECS: f32 = PARTICLE_TTL_MAX;
const PARTICLE_SPREAD_ARC: f32 = std::f32::consts::PI * 0.5;
const PARTICLE_SPREAD_JITTER: f32 = 0.24;
const PARTICLE_BASE_ALPHA: f32 = 1.0;
const PARTICLE_START_SCALE: f32 = 1.0;
const LARGE_PARTICLE_END_SCALE: f32 = 0.7;
const SMALL_PARTICLE_END_SCALE_MIN: f32 = LARGE_PARTICLE_END_SCALE * 0.6;
const SMALL_PARTICLE_END_SCALE_MAX: f32 = LARGE_PARTICLE_END_SCALE * 0.8;
const PARTICLE_VELOCITY_RETENTION_PER_SEC: f32 = 0.45;
const SCORE_POPUP_BASE_ALPHA: f32 = 1.0;
const SCORE_POPUP_FONT_SIZE: f32 = 28.0;
const SCORE_POPUP_RISE_MIN: f32 = 50.0;
const SCORE_POPUP_RISE_MAX: f32 = 70.0;
const SCORE_POPUP_LIFETIME_SECS: f32 = 0.45;
const SCORE_POPUP_FADE_SECS: f32 = 0.12;
const SCORE_POPUP_START_SCALE: f32 = 0.9;
const SCORE_POPUP_END_SCALE: f32 = 1.05;
const PLAYER_DEATH_SEQUENCE_SECS: f32 = PARTICLE_LIFETIME_SECS + 0.08;

const SCORE_POPUP_COLOR: Color = Color::srgb(0.98, 0.68, 0.18);

#[derive(Message)]
pub struct EnemyDied {
    pub translation: Vec3,
    pub burst_direction: Vec2,
}

#[derive(Message)]
pub struct PlayerDied {
    pub translation: Vec3,
    pub burst_direction: Vec2,
}

#[derive(Resource, Default)]
pub struct PendingGameOver {
    until: Option<f64>,
}

#[derive(Component)]
pub(crate) struct DeathParticle {
    spawned_at: f64,
    ttl_secs: f32,
    velocity: Vec2,
    end_scale: f32,
}

#[derive(Component)]
pub(crate) struct ScorePopup {
    spawned_at: f64,
    origin: Vec3,
    rise: f32,
}

struct ParticleStyle {
    base_color: Color,
    source_size: f32,
}

struct BurstSpec {
    center: Vec3,
    base_color: Color,
    core_size: f32,
    total_count: usize,
    use_arc: bool,
    base_angle: f32,
    now: f64,
}

struct ParticleParams {
    speed_scale: f32,
    ttl_scale: f32,
    end_scale: f32,
    brightness: f32,
}

type EffectEntityQuery<'w, 's> = Query<'w, 's, Entity, Or<(With<DeathParticle>, With<ScorePopup>)>>;

fn spawn_death_particles(
    commands: &mut Commands,
    translation: Vec3,
    burst_direction: Vec2,
    now: f64,
    style: &ParticleStyle,
) {
    let seed = translation.x * 0.013 + translation.y * 0.021;
    let extra_count_range = EXTRA_PARTICLE_COUNT_MAX - EXTRA_PARTICLE_COUNT_MIN;
    let extra_count = EXTRA_PARTICLE_COUNT_MIN
        + (((extra_count_range + 1) as f32) * hash01(seed * 0.91 + 11.0)).floor() as usize;
    let burst = BurstSpec {
        center: translation.truncate().extend(crate::PARTICLE_Z),
        base_color: style.base_color,
        core_size: style.source_size / CORE_FRAGMENT_GRID_SIZE as f32,
        total_count: CORE_FRAGMENT_COUNT + extra_count,
        use_arc: burst_direction.length_squared() > 0.0,
        base_angle: burst_direction.to_angle(),
        now,
    };

    for row in 0..CORE_FRAGMENT_GRID_SIZE {
        for col in 0..CORE_FRAGMENT_GRID_SIZE {
            let particle_index = row * CORE_FRAGMENT_GRID_SIZE + col;
            let particle_seed = seed + particle_index as f32 * 13.37;
            let offset = core_fragment_offset(col, row, style.source_size);
            burst.spawn_core_fragment(commands, offset, particle_seed);
        }
    }

    for extra_index in 0..extra_count {
        let particle_index = CORE_FRAGMENT_COUNT + extra_index;
        let particle_seed = seed + particle_index as f32 * 13.37;
        let size = burst.core_size
            * lerp(
                EXTRA_PARTICLE_SIZE_MIN_FACTOR,
                EXTRA_PARTICLE_SIZE_MAX_FACTOR,
                hash01(particle_seed + 2.0),
            );
        let offset = extra_fragment_offset(style.source_size, size, particle_seed);
        burst.spawn_fragment(commands, particle_index, particle_seed, offset, size);
    }
}

impl BurstSpec {
    fn spawn_core_fragment(&self, commands: &mut Commands, offset: Vec2, seed: f32) {
        self.spawn_particle(
            commands,
            seed,
            offset,
            self.core_size,
            self.core_direction(offset, seed),
            ParticleParams {
                speed_scale: LARGE_PARTICLE_SPEED_SCALE,
                ttl_scale: LARGE_PARTICLE_TTL_SCALE,
                end_scale: LARGE_PARTICLE_END_SCALE,
                brightness: LARGE_PARTICLE_BRIGHTNESS,
            },
        );
    }

    fn spawn_fragment(
        &self,
        commands: &mut Commands,
        index: usize,
        seed: f32,
        offset: Vec2,
        size: f32,
    ) {
        let size_t = (size / self.core_size).clamp(0.0, 1.0);
        self.spawn_particle(
            commands,
            seed,
            offset,
            size,
            self.arc_direction(index, seed),
            ParticleParams {
                speed_scale: SMALL_PARTICLE_SPEED_SCALE,
                ttl_scale: SMALL_PARTICLE_TTL_SCALE,
                end_scale: lerp(
                    SMALL_PARTICLE_END_SCALE_MIN,
                    SMALL_PARTICLE_END_SCALE_MAX,
                    hash01(seed + 9.0),
                ),
                brightness: lerp(SMALL_PARTICLE_BRIGHTNESS, LARGE_PARTICLE_BRIGHTNESS, size_t),
            },
        );
    }

    fn spawn_particle(
        &self,
        commands: &mut Commands,
        seed: f32,
        offset: Vec2,
        size: f32,
        direction: Vec2,
        params: ParticleParams,
    ) {
        let speed =
            lerp(PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX, hash01(seed + 3.0)) * params.speed_scale;
        let ttl_secs =
            lerp(PARTICLE_TTL_MIN, PARTICLE_TTL_MAX, hash01(seed + 4.0)) * params.ttl_scale;
        commands.spawn((
            DeathParticle {
                spawned_at: self.now,
                ttl_secs,
                velocity: direction * speed,
                end_scale: params.end_scale,
            },
            Sprite::from_color(
                brighten_color(self.base_color, params.brightness),
                Vec2::splat(size),
            ),
            Transform::from_translation(self.center + offset.extend(0.0)),
        ));
    }

    fn arc_direction(&self, index: usize, seed: f32) -> Vec2 {
        let t = if self.total_count > 1 {
            index as f32 / (self.total_count - 1) as f32
        } else {
            0.5
        };
        let angle = if self.use_arc {
            self.base_angle
                + lerp(-PARTICLE_SPREAD_ARC * 0.5, PARTICLE_SPREAD_ARC * 0.5, t)
                + (hash01(seed + 1.0) - 0.5) * PARTICLE_SPREAD_JITTER
        } else {
            t * TAU + (hash01(seed + 1.0) - 0.5) * 1.1
        };
        Vec2::from_angle(angle)
    }

    fn core_direction(&self, offset: Vec2, seed: f32) -> Vec2 {
        let outward_dir = offset.normalize_or_zero();
        if outward_dir.length_squared() == 0.0 {
            return self.arc_direction(CORE_FRAGMENT_COUNT / 2, seed);
        }

        if !self.use_arc {
            return Vec2::from_angle(outward_dir.to_angle() + core_fragment_jitter(seed));
        }

        let outward_angle = outward_dir.to_angle();
        let arc_offset = shortest_angle_delta(self.base_angle, outward_angle)
            .clamp(-PARTICLE_SPREAD_ARC * 0.5, PARTICLE_SPREAD_ARC * 0.5);
        Vec2::from_angle(self.base_angle + arc_offset + core_fragment_jitter(seed))
    }
}

fn core_fragment_offset(col: usize, row: usize, source_size: f32) -> Vec2 {
    let cell_size = source_size / CORE_FRAGMENT_GRID_SIZE as f32;
    let x = -source_size * 0.5 + (col as f32 + 0.5) * cell_size;
    let y = source_size * 0.5 - (row as f32 + 0.5) * cell_size;
    Vec2::new(x, y)
}

fn core_fragment_jitter(seed: f32) -> f32 {
    (hash01(seed + 8.0) - 0.5) * PARTICLE_SPREAD_JITTER * 0.5
}

fn shortest_angle_delta(from: f32, to: f32) -> f32 {
    let mut delta = to - from;
    while delta > std::f32::consts::PI {
        delta -= TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += TAU;
    }
    delta
}

fn extra_fragment_offset(source_size: f32, size: f32, seed: f32) -> Vec2 {
    let spawn_size = source_size * EXTRA_PARTICLE_SPAWN_SIDE_FACTOR;
    let half_extent = ((spawn_size - size) * 0.5).max(0.0);
    Vec2::new(
        lerp(-half_extent, half_extent, hash01(seed + 6.0)),
        lerp(-half_extent, half_extent, hash01(seed + 7.0)),
    )
}

fn hash01(seed: f32) -> f32 {
    let value = (seed.sin() * 43_758.547).fract();
    if value < 0.0 { value + 1.0 } else { value }
}

fn brighten_color(color: Color, factor: f32) -> Color {
    let srgba = color.to_srgba();
    let lift = (factor - 1.0).max(0.0);
    Color::srgba(
        (srgba.red + (1.0 - srgba.red) * lift).min(1.0),
        (srgba.green + (1.0 - srgba.green) * lift).min(1.0),
        (srgba.blue + (1.0 - srgba.blue) * lift).min(1.0),
        srgba.alpha,
    )
}

fn ease_in_cubic(t: f32) -> f32 {
    t.powi(3)
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn death_sequence_inactive(pending_game_over: Res<PendingGameOver>) -> bool {
    pending_game_over.until.is_none()
}

pub fn finish_game_over_delay(
    time: Res<Time>,
    mut pending_game_over: ResMut<PendingGameOver>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(until) = pending_game_over.until else {
        return;
    };

    if time.elapsed_secs_f64() >= until {
        pending_game_over.until = None;
        next_state.set(GameState::GameOver);
    }
}

pub fn spawn_enemy_death_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut enemy_deaths: MessageReader<EnemyDied>,
) {
    let now = time.elapsed_secs_f64();
    let font = asset_server.load(ui::UI_FONT_PATH);
    let particle_style = ParticleStyle {
        base_color: enemy::ENEMY_BASE_COLOR,
        source_size: enemy::ENEMY_SIZE,
    };

    for death in enemy_deaths.read() {
        spawn_death_particles(
            &mut commands,
            death.translation,
            death.burst_direction,
            now,
            &particle_style,
        );
        let seed = death.translation.x * 0.031 + death.translation.y * 0.017;
        let rise = lerp(
            SCORE_POPUP_RISE_MIN,
            SCORE_POPUP_RISE_MAX,
            hash01(seed + 5.0),
        );
        let origin = death.translation.truncate().extend(crate::SCORE_POPUP_Z);

        commands.spawn((
            ScorePopup {
                spawned_at: now,
                origin,
                rise,
            },
            Text2d::new("+1"),
            TextFont {
                font: font.clone(),
                font_size: SCORE_POPUP_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_POPUP_COLOR),
            Anchor::CENTER,
            Transform {
                translation: origin,
                scale: Vec3::splat(SCORE_POPUP_START_SCALE),
                ..default()
            },
        ));
    }
}

pub fn handle_player_death(
    mut commands: Commands,
    time: Res<Time>,
    mut pending_game_over: ResMut<PendingGameOver>,
    mut player_deaths: MessageReader<PlayerDied>,
) {
    let now = time.elapsed_secs_f64();
    let particle_style = ParticleStyle {
        base_color: player::PLAYER_BASE_COLOR,
        source_size: player::PLAYER_SIZE,
    };

    for death in player_deaths.read() {
        pending_game_over.until = Some(now + PLAYER_DEATH_SEQUENCE_SECS as f64);
        spawn_death_particles(
            &mut commands,
            death.translation,
            death.burst_direction,
            now,
            &particle_style,
        );
    }
}

pub fn update_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut Sprite, &mut DeathParticle)>,
) {
    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();

    for (entity, mut transform, mut sprite, mut particle) in &mut particles {
        let age = (now - particle.spawned_at) as f32;

        if age >= particle.ttl_secs {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity.extend(0.0) * dt;
        particle.velocity *= PARTICLE_VELOCITY_RETENTION_PER_SEC.powf(dt);
        let progress = (age / particle.ttl_secs).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(lerp(
            PARTICLE_START_SCALE,
            particle.end_scale,
            ease_out_cubic(progress),
        ));

        let fade_start = (particle.ttl_secs - PARTICLE_FADE_SECS).max(0.0);

        if age < fade_start {
            sprite.color.set_alpha(PARTICLE_BASE_ALPHA);
        } else {
            let fade_progress = ((age - fade_start)
                / (particle.ttl_secs - fade_start).max(f32::EPSILON))
            .clamp(0.0, 1.0);
            sprite
                .color
                .set_alpha(PARTICLE_BASE_ALPHA * (1.0 - ease_in_cubic(fade_progress)));
        }
    }
}

pub fn update_score_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut popups: Query<(Entity, &mut Transform, &mut TextColor, &ScorePopup)>,
) {
    let now = time.elapsed_secs_f64();

    for (entity, mut transform, mut color, popup) in &mut popups {
        let age = (now - popup.spawned_at) as f32;

        if age >= SCORE_POPUP_LIFETIME_SECS {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (age / SCORE_POPUP_LIFETIME_SECS).clamp(0.0, 1.0);
        let move_progress = smoothstep(progress);
        transform.translation = popup.origin + Vec3::Y * (popup.rise * move_progress);
        transform.scale = Vec3::splat(lerp(
            SCORE_POPUP_START_SCALE,
            SCORE_POPUP_END_SCALE,
            move_progress,
        ));

        if age < SCORE_POPUP_LIFETIME_SECS - SCORE_POPUP_FADE_SECS {
            color.0.set_alpha(SCORE_POPUP_BASE_ALPHA);
        } else {
            let fade_progress = ((age - (SCORE_POPUP_LIFETIME_SECS - SCORE_POPUP_FADE_SECS))
                / SCORE_POPUP_FADE_SECS)
                .clamp(0.0, 1.0);
            color
                .0
                .set_alpha(SCORE_POPUP_BASE_ALPHA * (1.0 - ease_in_cubic(fade_progress)));
        }
    }
}

pub fn despawn_effects_on_restart(
    mut commands: Commands,
    effects: EffectEntityQuery,
    mut restart_requests: MessageReader<RestartGame>,
    mut pending_game_over: ResMut<PendingGameOver>,
) {
    if restart_requests.read().next().is_none() {
        return;
    }

    pending_game_over.until = None;

    for entity in &effects {
        commands.entity(entity).despawn();
    }
}
