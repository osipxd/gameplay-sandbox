use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Message)]
pub struct RestartGame;

pub fn resume_on_restart(
    mut restart_requests: MessageReader<RestartGame>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if restart_requests.read().next().is_some() {
        next_state.set(GameState::Playing);
    }
}
