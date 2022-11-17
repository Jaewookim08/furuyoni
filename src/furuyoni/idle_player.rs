use async_trait::async_trait;
use crate::furuyoni;
use crate::furuyoni::game::{MainPhaseAction, ViewableState};


pub struct IdlePlayer {}

#[async_trait]
impl furuyoni::Player for IdlePlayer {
    async fn get_main_phase_action(&self, state: &ViewableState<'_>, available_actions: &Vec<MainPhaseAction>) -> MainPhaseAction {
        MainPhaseAction::EndMainPhase
    }
}