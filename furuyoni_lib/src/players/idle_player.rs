use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::ViewableState;
use async_trait::async_trait;

pub struct IdlePlayer {}

#[async_trait]
impl super::Player for IdlePlayer {
    async fn get_main_phase_action(
        &self,
        _state: &ViewableState,
        _playable_cards: &Vec<PlayableCardSelector>,
        _doable_basic_actions: &Vec<BasicAction>,
        _available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        MainPhaseAction::EndMainPhase
    }
}
