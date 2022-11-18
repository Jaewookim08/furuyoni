use async_trait::async_trait;
use crate::furuyoni::game;
use crate::furuyoni::game::{BasicAction, BasicActionCost, PlayableCardSelector};

#[async_trait]
pub trait Player {
    async fn get_main_phase_action(&self,
                                   state: &game::ViewableState<'_>,
                                   playable_cards: &Vec<PlayableCardSelector>,
                                   doable_basic_actions: &Vec<BasicAction>,
                                   available_basic_action_costs: &Vec<BasicActionCost>) -> game::MainPhaseAction;
}


