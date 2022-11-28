use crate::networking::GameConnection;
use async_trait::async_trait;
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::ViewableState;

pub struct RemotePlayer {
    connection: GameConnection,
}

impl RemotePlayer {
    pub fn new(connection: GameConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl Player for RemotePlayer {
    async fn get_main_phase_action(
        &self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        doable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        todo!()
    }
}