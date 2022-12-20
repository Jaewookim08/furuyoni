use crate::networking::GameToPlayerConnection;
use async_trait::async_trait;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameRequest, PlayerResponse, RequestMainPhaseAction, ServerMessageFrame,
};
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::ViewableState;

pub struct RemotePlayer {
    game_to_player: GameToPlayerConnection,
}

impl RemotePlayer {
    pub fn new(game_to_player: GameToPlayerConnection) -> Self {
        Self { game_to_player }
    }
}

#[async_trait]
impl Player for RemotePlayer {
    async fn get_main_phase_action(
        &mut self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        let response = self
            .game_to_player
            .request(GameRequest::RequestMainPhaseAction(
                RequestMainPhaseAction {
                    state: state.clone(),
                    playable_cards: playable_cards.clone(),
                    performable_basic_actions: performable_basic_actions.clone(),
                    available_basic_action_costs: available_basic_action_costs.clone(),
                },
            ))
            .await
            .expect("Todo");

        if let PlayerResponse::ResponseMainPhaseAction(response) = response {
            response.action
        } else {
            todo!()
        }
    }
}
