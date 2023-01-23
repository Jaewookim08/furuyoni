use crate::furuyoni_lib::net::Requester;
use async_trait::async_trait;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameNotification, GameRequest, GameToPlayerRequestData, PlayerResponse,
    RequestMainPhaseAction, ServerMessageFrame,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::MessageSender;
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::ViewableState;

pub struct RemotePlayer<TRequester, TSender> {
    requester: TRequester,
    notifier: TSender,
}

impl<TRequester, TSender> RemotePlayer<TRequester, TSender> {
    pub fn new(requester: TRequester, notifier: TSender) -> Self {
        Self {
            requester,
            notifier,
        }
    }
}

#[async_trait]
impl<TRequester, TSender> Player for RemotePlayer<TRequester, TSender>
where
    TRequester: Requester<GameToPlayerRequestData, Response = PlayerResponse> + Send,
    TSender: MessageSender<GameNotification> + Send,
{
    async fn get_main_phase_action(
        &mut self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        let response = self
            .requester
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
