use async_trait::async_trait;
use furuyoni_lib::net::frames::{
    GameToPlayerRequest, PlayerToGameResponse, RequestMainPhaseAction,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::rules::events::GameEvent;

use crate::game_watcher::{GameObserver, NotifyFailedError};
use crate::players::Player;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::states::StateView;
use furuyoni_lib::rules::PlayerPos;

type ChannelT = MessageChannel<GameToPlayerRequest, PlayerToGameResponse>;

pub(crate) struct RemotePlayer {
    channel: ChannelT,
}

impl RemotePlayer {
    pub fn new(channel: ChannelT) -> Self {
        Self { channel }
    }
}

impl RemotePlayer {
    fn send_state(&mut self, state: &StateView) -> Result<(), ()> {
        self.channel
            .send(GameToPlayerRequest::CheckGameState(state.clone()))
            .map_err(|_| ())?;
        Ok(())
    }
}
#[async_trait]
impl Player for RemotePlayer {
    async fn get_main_phase_action(
        &mut self,
        state: &StateView,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> Result<MainPhaseAction, ()> {
        self.send_state(state)?;

        self.channel
            .send(GameToPlayerRequest::RequestMainPhaseAction(
                RequestMainPhaseAction {
                    playable_cards: playable_cards.clone(),
                    performable_basic_actions: performable_basic_actions.clone(),
                    available_basic_action_costs: available_basic_action_costs.clone(),
                },
            ))
            .map_err(|_| ())?;

        let response = self.channel.receive().await.map_err(|_| ())?;

        if let PlayerToGameResponse::MainPhaseAction(response) = response {
            Ok(response)
        } else {
            Err(())
        }
    }

    async fn request_game_start(&mut self, pos: PlayerPos) -> Result<(), ()> {
        self.channel
            .send(GameToPlayerRequest::RequestGameStart { pos })
            .map_err(|_| ())?;

        let response = self.channel.receive().await.map_err(|_| ())?;

        if let PlayerToGameResponse::AcknowledgeGameStart = response {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl GameObserver for RemotePlayer {
    fn initialize_state(&mut self, _state: &StateView) -> Result<(), NotifyFailedError> {
        self.channel
            .send(GameToPlayerRequest::InitializeGameState(_state.clone()))
            .map_err(|_| NotifyFailedError)?;
        Ok(())
    }

    fn notify_event(&mut self, event: &GameEvent) -> Result<(), NotifyFailedError> {
        self.channel
            .send(GameToPlayerRequest::NotifyEvent((*event).clone()))
            .map_err(|_| NotifyFailedError)?;
        Ok(())
    }
}
