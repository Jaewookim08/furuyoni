use crate::furuyoni_lib::net::Requester;
use async_trait::async_trait;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameRequest, GameToPlayerRequestData, PlayerResponse,
    RequestMainPhaseAction, ServerMessageFrame,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::ViewableState;

pub struct RemotePlayer<TRequester> {
    game_to_player: TRequester,
}

impl<TRequester> RemotePlayer<TRequester> {
    pub fn new(game_to_player: TRequester) -> Self {
        Self { game_to_player }
    }
}

#[async_trait]
impl<TRequester> Player for RemotePlayer<TRequester>
where
    TRequester: Requester<GameToPlayerRequestData, Response = PlayerResponse> + Send,
{
    async fn get_main_phase_action(
        &mut self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        let response = self
            .game_to_player
            .request(GameToPlayerRequestData::RequestMainPhaseAction(
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
