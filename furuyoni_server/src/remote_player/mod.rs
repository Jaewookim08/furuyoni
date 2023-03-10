use crate::furuyoni_lib::net::Requester;
use async_trait::async_trait;
use furuyoni_lib::events::GameEvent;
use furuyoni_lib::net::frames::{
    GameToPlayerNotification, GameToPlayerRequestData, PlayerToGameResponse, RequestMainPhaseAction,
};

use furuyoni_lib::net::message_sender::MessageSender;
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::{PlayerPos, ViewableState};

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
    TRequester: Requester<GameToPlayerRequestData, Response = PlayerToGameResponse> + Send,
    TSender: MessageSender<GameToPlayerNotification> + Send,
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

        if let PlayerToGameResponse::MainPhaseAction(response) = response {
            response.action
        } else {
            todo!()
        }
    }

    async fn start_game(&mut self, state: &ViewableState, pos: PlayerPos) -> Result<(), ()> {
        let response = self
            .requester
            .request(GameToPlayerRequestData::RequestGameStart {
                state: state.clone(),
                pos,
            })
            .await
            .map_err(|_| ())?;

        if let PlayerToGameResponse::AcknowledgeGameStart = response {
            Ok(())
        } else {
            Err(())
        }
    }

    fn notify_event(&mut self, event: GameEvent) -> Result<(), ()> {
        self.notifier
            .send_message(GameToPlayerNotification::Event(event))
            .map_err(|_| ())?;
        Ok(())
    }
}
