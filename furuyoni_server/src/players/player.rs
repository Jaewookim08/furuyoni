use crate::game_watcher::GameObserver;
use async_trait::async_trait;
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::states::*;
use furuyoni_lib::rules::PlayerPos;

#[async_trait]
pub(crate) trait Player: GameObserver {
    async fn get_main_phase_action(
        &mut self,
        state: &StateView,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> Result<MainPhaseAction, ()>;

    async fn request_game_start(&mut self, _pos: PlayerPos) -> Result<(), ()> {
        Ok(())
    }
}
