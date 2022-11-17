use async_trait::async_trait;
use crate::furuyoni::game;

#[async_trait]
pub trait Player {
    async fn get_main_phase_action(&self, state: &game::ViewableState<'_>, available_actions: &Vec<game::MainPhaseAction>) -> game::MainPhaseAction;
}