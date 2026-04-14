use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::color::{Alpha, Mix};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::sprite::Anchor;
use rand::Rng;
use serde::Deserialize;
use std::f32::consts::TAU;
use thiserror::Error;

use crate::enemy;
use crate::game_state::{GameState, RestartGame};
use crate::player;
use crate::random_source::RandomSource;
use crate::ui;

const EFFECTS_CONFIG_PATH: &str = "effects.ron";
const SCORE_POPUP_TEXT: &str = "+1";
const EXTRA_PARTICLE_SPAWN_SIDE_FACTOR: f32 = 0.5;
const PARTICLE_BASE_ALPHA: f32 = 1.0;
const PARTICLE_START_SCALE: f32 = 1.0;
const SCORE_POPUP_BASE_ALPHA: f32 = 1.0;
const SCORE_POPUP_FONT_SIZE: f32 = 28.0;
const SCORE_POPUP_OVERSHOOT_PROGRESS: f32 = 0.45;
const SCORE_POPUP_START_SCALE: f32 = 0.9;

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize, PartialEq)]
pub struct EffectsConfig {
    pub death_particles: DeathParticlesConfig,
    pub score_popup: ScorePopupConfig,
    pub player_death: PlayerDeathConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeathParticlesConfig {
    pub core_grid_size: usize,
    pub extra_count: WithJitter<usize>,
    pub extra_size_factor: WithJitterPct,
    pub impact_bias: f32,
    pub fade_pct: f32,
    pub velocity_retention_per_sec: f32,
    pub large: FragmentConfig,
    pub small: FragmentConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FragmentConfig {
    pub speed: WithJitterPct,
    pub ttl_secs: WithJitterPct,
    pub brightness: f32,
    pub direction_jitter: f32,
    pub end_scale: WithJitterPct,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ScorePopupConfig {
    pub drift_x: WithJitter<f32>,
    pub rise: WithJitterPct,
    pub lifetime_secs: f32,
    pub fade_pct: f32,
    pub scale_overshoot_pct: f32,
    pub end_scale: f32,
    pub color: ColorConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlayerDeathConfig {
    pub extra_game_over_delay_secs: f32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ColorConfig {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct WithJitter<T> {
    pub value: T,
    pub jitter: T,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct WithJitterPct {
    pub value: f32,
    pub jitter_pct: f32,
}

#[derive(Resource, Default)]
pub struct EffectsConfigHandle(pub Handle<EffectsConfig>);

#[derive(Default, TypePath)]
pub struct EffectsConfigLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EffectsConfigLoaderError {
    #[error("Could not load effects config: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse effects config: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

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
    timer: Option<Timer>,
}

#[derive(Component)]
pub(crate) struct DeathParticle {
    lifetime: Timer,
    velocity: Vec2,
    end_scale: f32,
}

#[derive(Component)]
pub(crate) struct ScorePopup {
    lifetime: Timer,
    origin: Vec3,
    rise: f32,
}

struct ParticleStyle {
    base_color: Color,
    source_size: f32,
}

struct BurstSpec<'a> {
    center: Vec3,
    base_color: Color,
    core_size: f32,
    impact_direction: Option<Vec2>,
    config: &'a DeathParticlesConfig,
}

type EffectEntityQuery<'w, 's> = Query<'w, 's, Entity, Or<(With<DeathParticle>, With<ScorePopup>)>>;

impl AssetLoader for EffectsConfigLoader {
    type Asset = EffectsConfig;
    type Settings = ();
    type Error = EffectsConfigLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes::<EffectsConfig>(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl DeathParticlesConfig {
    fn random_extra_count(&self, rng: &mut RandomSource) -> usize {
        self.extra_count.sample(rng)
    }

    fn random_extra_size_factor(&self, rng: &mut RandomSource) -> f32 {
        self.extra_size_factor.sample(rng).max(0.0)
    }

    fn max_ttl_secs(&self) -> f32 {
        self.large.max_ttl_secs().max(self.small.max_ttl_secs())
    }
}

impl FragmentConfig {
    fn random_speed(&self, rng: &mut RandomSource) -> f32 {
        self.speed.sample(rng).max(0.0)
    }

    fn random_ttl_secs(&self, rng: &mut RandomSource) -> f32 {
        self.ttl_secs.sample(rng).max(f32::EPSILON)
    }

    fn random_end_scale(&self, rng: &mut RandomSource) -> f32 {
        self.end_scale.sample(rng).max(0.0)
    }

    fn max_ttl_secs(&self) -> f32 {
        self.ttl_secs.max_value()
    }
}

impl ScorePopupConfig {
    fn random_drift_x(&self, rng: &mut RandomSource) -> f32 {
        self.drift_x.sample(rng)
    }

    fn random_rise(&self, rng: &mut RandomSource) -> f32 {
        self.rise.sample(rng).max(0.0)
    }

    fn scale_at(&self, progress: f32) -> f32 {
        let overshoot_scale = self.end_scale * (1.0 + self.scale_overshoot_pct.max(0.0));

        if progress < SCORE_POPUP_OVERSHOOT_PROGRESS {
            let inflate_progress =
                (progress / SCORE_POPUP_OVERSHOOT_PROGRESS.max(f32::EPSILON)).clamp(0.0, 1.0);
            SCORE_POPUP_START_SCALE.lerp(
                overshoot_scale,
                EaseFunction::CubicOut.sample_clamped(inflate_progress),
            )
        } else {
            let settle_progress = ((progress - SCORE_POPUP_OVERSHOOT_PROGRESS)
                / (1.0 - SCORE_POPUP_OVERSHOOT_PROGRESS).max(f32::EPSILON))
            .clamp(0.0, 1.0);
            overshoot_scale.lerp(
                self.end_scale,
                EaseFunction::CubicOut.sample_clamped(settle_progress),
            )
        }
    }
}

impl ColorConfig {
    fn to_color(&self) -> Color {
        Color::srgba(self.red, self.green, self.blue, self.alpha)
    }
}

impl EffectsConfig {
    fn player_death_sequence_secs(&self) -> f32 {
        self.death_particles.max_ttl_secs() + self.player_death.extra_game_over_delay_secs
    }
}

impl WithJitter<usize> {
    fn sample(&self, rng: &mut RandomSource) -> usize {
        let min = self.value.saturating_sub(self.jitter);
        let max = self.value + self.jitter;
        rng.0.random_range(min..=max)
    }
}

impl WithJitter<f32> {
    fn sample(&self, rng: &mut RandomSource) -> f32 {
        rng.0
            .random_range((self.value - self.jitter)..=(self.value + self.jitter))
    }
}

impl WithJitterPct {
    fn sample(&self, rng: &mut RandomSource) -> f32 {
        sample_pct(rng, self.value, self.jitter_pct)
    }

    fn max_value(&self) -> f32 {
        self.value * (1.0 + self.jitter_pct)
    }
}

pub fn effects_config_ready(config: Option<Res<EffectsConfig>>) -> bool {
    config.is_some()
}

pub fn load_effects_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(EffectsConfigHandle(asset_server.load(EFFECTS_CONFIG_PATH)));
}

pub fn sync_effects_config(
    handle: Res<EffectsConfigHandle>,
    loaded_configs: Res<Assets<EffectsConfig>>,
    current: Option<Res<EffectsConfig>>,
    mut commands: Commands,
) {
    if let Some(loaded) = loaded_configs.get(&handle.0) {
        let current = current.as_deref();
        if current != Some(loaded) {
            commands.insert_resource(loaded.clone());
        }
    }
}

fn spawn_death_particles(
    commands: &mut Commands,
    rng: &mut RandomSource,
    config: &DeathParticlesConfig,
    translation: Vec3,
    burst_direction: Vec2,
    style: &ParticleStyle,
) {
    let extra_count = config.random_extra_count(rng);
    let burst = BurstSpec {
        center: translation.truncate().extend(crate::PARTICLE_Z),
        base_color: style.base_color,
        core_size: style.source_size / config.core_grid_size as f32,
        impact_direction: (burst_direction.length_squared() > 0.0)
            .then(|| burst_direction.normalize_or_zero()),
        config,
    };

    for row in 0..config.core_grid_size {
        for col in 0..config.core_grid_size {
            let offset = core_fragment_offset(col, row, style.source_size, config.core_grid_size);
            burst.spawn_core_fragment(commands, rng, offset);
        }
    }

    for _ in 0..extra_count {
        let size = burst.core_size * config.random_extra_size_factor(rng);
        let offset = extra_fragment_offset(
            style.source_size,
            size,
            EXTRA_PARTICLE_SPAWN_SIDE_FACTOR,
            rng,
        );
        burst.spawn_extra_fragment(commands, rng, offset, size);
    }
}

impl BurstSpec<'_> {
    fn spawn_core_fragment(&self, commands: &mut Commands, rng: &mut RandomSource, offset: Vec2) {
        self.spawn_particle(commands, rng, offset, self.core_size, &self.config.large);
    }

    fn spawn_extra_fragment(
        &self,
        commands: &mut Commands,
        rng: &mut RandomSource,
        offset: Vec2,
        size: f32,
    ) {
        self.spawn_particle(commands, rng, offset, size, &self.config.small);
    }

    fn spawn_particle(
        &self,
        commands: &mut Commands,
        rng: &mut RandomSource,
        offset: Vec2,
        size: f32,
        fragment: &FragmentConfig,
    ) {
        let direction = self.fragment_direction(offset, rng, fragment.direction_jitter);
        commands.spawn((
            DeathParticle {
                lifetime: Timer::from_seconds(fragment.random_ttl_secs(rng), TimerMode::Once),
                velocity: direction * fragment.random_speed(rng),
                end_scale: fragment.random_end_scale(rng),
            },
            Sprite::from_color(
                brighten_color(self.base_color, fragment.brightness),
                Vec2::splat(size),
            ),
            Transform::from_translation(self.center + offset.extend(0.0)),
        ));
    }

    fn fragment_direction(&self, offset: Vec2, rng: &mut RandomSource, jitter: f32) -> Vec2 {
        let radial_dir = if offset.length_squared() > 0.0 {
            offset.normalize()
        } else {
            Vec2::from_angle(rng.0.random_range(0.0..TAU))
        };
        let base_dir = if let Some(impact_dir) = self.impact_direction {
            radial_dir
                .lerp(impact_dir, self.config.impact_bias.clamp(0.0, 1.0))
                .normalize_or_zero()
        } else {
            radial_dir
        };
        let base_angle = if base_dir.length_squared() > 0.0 {
            base_dir.to_angle()
        } else {
            rng.0.random_range(0.0..TAU)
        };

        Vec2::from_angle(base_angle + angle_jitter(rng, jitter))
    }
}

fn core_fragment_offset(col: usize, row: usize, source_size: f32, grid_size: usize) -> Vec2 {
    let cell_size = source_size / grid_size as f32;
    let x = -source_size * 0.5 + (col as f32 + 0.5) * cell_size;
    let y = source_size * 0.5 - (row as f32 + 0.5) * cell_size;
    Vec2::new(x, y)
}

fn angle_jitter(rng: &mut RandomSource, jitter: f32) -> f32 {
    rng.0.random_range(-jitter..=jitter)
}

fn sample_pct(rng: &mut RandomSource, value: f32, jitter_pct: f32) -> f32 {
    let factor = rng.0.random_range((1.0 - jitter_pct)..=(1.0 + jitter_pct));
    value * factor
}

fn extra_fragment_offset(
    source_size: f32,
    size: f32,
    spawn_side_factor: f32,
    rng: &mut RandomSource,
) -> Vec2 {
    let spawn_size = source_size * spawn_side_factor;
    let half_extent = ((spawn_size - size) * 0.5).max(0.0);
    Vec2::new(
        rng.0.random_range(-half_extent..=half_extent),
        rng.0.random_range(-half_extent..=half_extent),
    )
}

fn brighten_color(color: Color, factor: f32) -> Color {
    color.mix(&Color::WHITE, (factor - 1.0).clamp(0.0, 1.0))
}

fn fade_alpha(progress: f32, fade_pct: f32, base_alpha: f32, easing: EaseFunction) -> f32 {
    let fade_start = (1.0 - fade_pct).clamp(0.0, 1.0);

    if progress < fade_start {
        base_alpha
    } else {
        let fade_progress =
            ((progress - fade_start) / (1.0 - fade_start).max(f32::EPSILON)).clamp(0.0, 1.0);
        base_alpha * (1.0 - easing.sample_clamped(fade_progress))
    }
}

pub fn death_sequence_inactive(pending_game_over: Res<PendingGameOver>) -> bool {
    pending_game_over.timer.is_none()
}

pub fn finish_game_over_delay(
    time: Res<Time>,
    mut pending_game_over: ResMut<PendingGameOver>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(timer) = pending_game_over.timer.as_mut() else {
        return;
    };

    timer.tick(time.delta());

    if timer.is_finished() {
        pending_game_over.timer = None;
        next_state.set(GameState::GameOver);
    }
}

pub fn spawn_enemy_death_effects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    effects_config: Res<EffectsConfig>,
    mut rng: ResMut<RandomSource>,
    mut enemy_deaths: MessageReader<EnemyDied>,
) {
    let font = asset_server.load(ui::UI_FONT_PATH);
    let particle_style = ParticleStyle {
        base_color: enemy::ENEMY_BASE_COLOR,
        source_size: enemy::ENEMY_SIZE,
    };

    for death in enemy_deaths.read() {
        spawn_death_particles(
            &mut commands,
            &mut rng,
            &effects_config.death_particles,
            death.translation,
            death.burst_direction,
            &particle_style,
        );
        let drift_x = effects_config.score_popup.random_drift_x(&mut rng);
        let rise = effects_config.score_popup.random_rise(&mut rng);
        let origin = death.translation.truncate().extend(crate::SCORE_POPUP_Z) + Vec3::X * drift_x;

        commands.spawn((
            ScorePopup {
                lifetime: Timer::from_seconds(
                    effects_config.score_popup.lifetime_secs,
                    TimerMode::Once,
                ),
                origin,
                rise,
            },
            Text2d::new(SCORE_POPUP_TEXT),
            TextFont {
                font: font.clone(),
                font_size: SCORE_POPUP_FONT_SIZE,
                ..default()
            },
            TextColor(effects_config.score_popup.color.to_color()),
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
    effects_config: Res<EffectsConfig>,
    mut rng: ResMut<RandomSource>,
    mut pending_game_over: ResMut<PendingGameOver>,
    mut player_deaths: MessageReader<PlayerDied>,
) {
    let particle_style = ParticleStyle {
        base_color: player::PLAYER_BASE_COLOR,
        source_size: player::PLAYER_SIZE,
    };

    for death in player_deaths.read() {
        pending_game_over.timer = Some(Timer::from_seconds(
            effects_config.player_death_sequence_secs(),
            TimerMode::Once,
        ));
        spawn_death_particles(
            &mut commands,
            &mut rng,
            &effects_config.death_particles,
            death.translation,
            death.burst_direction,
            &particle_style,
        );
    }
}

pub fn update_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    effects_config: Res<EffectsConfig>,
    mut particles: Query<(Entity, &mut Transform, &mut Sprite, &mut DeathParticle)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut sprite, mut particle) in &mut particles {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity.extend(0.0) * dt;
        particle.velocity *= effects_config
            .death_particles
            .velocity_retention_per_sec
            .powf(dt);

        let progress = particle.lifetime.fraction().clamp(0.0, 1.0);
        transform.scale = Vec3::splat(PARTICLE_START_SCALE.lerp(
            particle.end_scale,
            EaseFunction::CubicOut.sample_clamped(progress),
        ));
        sprite.color = sprite.color.with_alpha(fade_alpha(
            progress,
            effects_config.death_particles.fade_pct,
            PARTICLE_BASE_ALPHA,
            EaseFunction::QuadraticOut,
        ));
    }
}

pub fn update_score_popups(
    mut commands: Commands,
    time: Res<Time>,
    effects_config: Res<EffectsConfig>,
    mut popups: Query<(Entity, &mut Transform, &mut TextColor, &mut ScorePopup)>,
) {
    for (entity, mut transform, mut color, mut popup) in &mut popups {
        popup.lifetime.tick(time.delta());
        if popup.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = popup.lifetime.fraction().clamp(0.0, 1.0);
        let move_progress = EaseFunction::QuadraticOut.sample_clamped(progress);
        transform.translation = popup.origin + Vec3::Y * (popup.rise * move_progress);
        transform.scale = Vec3::splat(effects_config.score_popup.scale_at(progress));
        color.0 = color.0.with_alpha(fade_alpha(
            progress,
            effects_config.score_popup.fade_pct,
            SCORE_POPUP_BASE_ALPHA,
            EaseFunction::CubicOut,
        ));
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

    pending_game_over.timer = None;

    for entity in &effects {
        commands.entity(entity).despawn();
    }
}
