use crate::rules::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use crate::rules::PlayerPos;

use crate::rules::events::GameEvent;
use crate::rules::states::StateView;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerMessage {
    Request(GameToPlayerRequest),
    Response(GameToPlayerResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerRequest {
    NotifyEvent(GameEvent),
    InitializeGameState(StateView),
    CheckGameState(StateView),
    RequestMainPhaseAction(RequestMainPhaseAction),
    RequestGameStart { pos: PlayerPos },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMainPhaseAction {
    pub playable_cards: Vec<PlayableCardSelector>,
    pub performable_basic_actions: Vec<BasicAction>,
    pub available_basic_action_costs: Vec<BasicActionCost>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerResponse {
    State(StateView),
    Ack,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameMessage {
    Response(PlayerToGameResponse),
    Request(PlayerToGameRequest),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameResponse {
    AcknowledgeGameStart,
    MainPhaseAction(MainPhaseAction),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameRequest {
    RequestState,
    RequestSurrender,
}
