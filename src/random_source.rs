use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const RANDOM_SEED_ENV: &str = "GAMEPLAY_SANDBOX_SEED";

#[derive(Resource)]
pub struct RandomSource(pub ChaCha8Rng);

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
