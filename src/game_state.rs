use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Message)]
pub struct RestartGame;

#[derive(Resource, Default)]
pub struct Score(pub u32);

pub fn reset_score_on_restart(
    mut restart_requests: MessageReader<RestartGame>,
    mut score: ResMut<Score>,
) {
    if restart_requests.read().next().is_some() {
        score.0 = 0;
    }
}

pub fn resume_on_restart(
    mut restart_requests: MessageReader<RestartGame>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if restart_requests.read().next().is_some() {
        next_state.set(GameState::Playing);
    }
}
