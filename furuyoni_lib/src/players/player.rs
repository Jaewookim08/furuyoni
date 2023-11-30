use crate::rules::events::GameEvent;
use crate::rules::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use crate::rules::states::*;
use crate::rules::PlayerPos;
use async_trait::async_trait;

#[async_trait]
pub trait Player {
    async fn get_main_phase_action(
        &mut self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> Result<MainPhaseAction, ()>;

    async fn notify_game_start(
        &mut self,
        _state: &ViewableState,
        _pos: PlayerPos,
    ) -> Result<(), ()> {
        Ok(())
    }

    fn notify_event(&mut self, _event: GameEvent) -> Result<(), ()> {
        Ok(())
    }
}
