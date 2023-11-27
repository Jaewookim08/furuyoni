use async_trait::async_trait;
use furuyoni_lib::events::GameEvent;
use furuyoni_lib::net::frames::{
    GameToPlayerRequest, PlayerToGameResponse, RequestMainPhaseAction,
};
use furuyoni_lib::net::message_channel::MessageChannel;

use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::{PlayerPos, ViewableState};

type ChannelT = MessageChannel<GameToPlayerRequest, PlayerToGameResponse>;

pub struct RemotePlayer {
    channel: ChannelT,
}

impl RemotePlayer {
    pub fn new(channel: ChannelT) -> Self {
        Self { channel }
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
    ) -> Result<MainPhaseAction, ()> {
        self.channel
            .send(GameToPlayerRequest::RequestMainPhaseAction(
                RequestMainPhaseAction {
                    state: state.clone(),
                    playable_cards: playable_cards.clone(),
                    performable_basic_actions: performable_basic_actions.clone(),
                    available_basic_action_costs: available_basic_action_costs.clone(),
                },
            ))
            .expect("Todo: add error type for Player");

        let response = self.channel.receive().await.map_err(|_| ())?;

        if let PlayerToGameResponse::MainPhaseAction(response) = response {
            Ok(response.action)
        } else {
            Err(())
        }
    }

    async fn notify_game_start(&mut self, state: &ViewableState, pos: PlayerPos) -> Result<(), ()> {
        self.channel
            .send(GameToPlayerRequest::RequestGameStart {
                state: state.clone(),
                pos,
            })
            .map_err(|_| ())?;

        let response = self.channel.receive().await.map_err(|_| ())?;

        if let PlayerToGameResponse::AcknowledgeGameStart = response {
            Ok(())
        } else {
            Err(())
        }
    }

    fn notify_event(&mut self, event: GameEvent) -> Result<(), ()> {
        self.channel
            .send(GameToPlayerRequest::NotifyEvent(event))
            .map_err(|_| ())?;
        Ok(())
    }
}
