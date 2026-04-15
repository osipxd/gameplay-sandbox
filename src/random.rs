use bevy::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;

const RANDOM_SEED_ENV: &str = "GAMEPLAY_SANDBOX_SEED";

#[derive(Resource)]
pub struct RandomSource(pub ChaCha8Rng);

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct WithJitter<T> {
    pub value: T,
    pub jitter: T,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct WithJitterPct {
    pub value: f32,
    pub jitter_pct: f32,
}

impl WithJitter<usize> {
    pub fn sample(&self, rng: &mut RandomSource) -> usize {
        let min = self.value.saturating_sub(self.jitter);
        let max = self.value + self.jitter;
        rng.0.random_range(min..=max)
    }
}

impl WithJitter<f32> {
    pub fn sample(&self, rng: &mut RandomSource) -> f32 {
        rng.0
            .random_range((self.value - self.jitter)..=(self.value + self.jitter))
    }
}

impl WithJitterPct {
    pub fn sample(&self, rng: &mut RandomSource) -> f32 {
        let factor = rng
            .0
            .random_range((1.0 - self.jitter_pct)..=(1.0 + self.jitter_pct));
        self.value * factor
    }

    pub fn max_value(&self) -> f32 {
        self.value * (1.0 + self.jitter_pct)
    }
}

impl Default for RandomSource {
    fn default() -> Self {
        let rng = std::env::var(RANDOM_SEED_ENV)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(ChaCha8Rng::seed_from_u64)
            .unwrap_or_else(ChaCha8Rng::from_os_rng);

        Self(rng)
    }
}
