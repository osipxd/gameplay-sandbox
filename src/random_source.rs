use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const RANDOM_SEED: u64 = 68_941_654_987_813_521;

#[derive(Resource)]
pub struct RandomSource(pub ChaCha8Rng);

impl Default for RandomSource {
    fn default() -> Self {
        Self(ChaCha8Rng::seed_from_u64(RANDOM_SEED))
    }
}
