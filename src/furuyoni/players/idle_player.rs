use crate::furuyoni;
use crate::furuyoni::game::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector, ViewableState,
};
use async_trait::async_trait;

pub struct IdlePlayer {}

#[async_trait]
impl furuyoni::Player for IdlePlayer {
    async fn get_main_phase_action(
        &self,
        state: &ViewableState<'_>,
        playable_cards: &Vec<PlayableCardSelector>,
        doable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        MainPhaseAction::EndMainPhase
    }
}
