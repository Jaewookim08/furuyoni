use crate::game_watcher::GameObserver;
use async_trait::async_trait;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::states::*;

pub(crate) struct IdlePlayer {}

#[async_trait]
impl super::Player for IdlePlayer {
    async fn get_main_phase_action(
        &mut self,
        _state: &ViewableState,
        _playable_cards: &Vec<PlayableCardSelector>,
        _performable_basic_actions: &Vec<BasicAction>,
        _available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> Result<MainPhaseAction, ()> {
        Ok(MainPhaseAction::EndMainPhase)
    }
}
impl GameObserver for IdlePlayer {}
