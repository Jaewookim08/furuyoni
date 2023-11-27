use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::{PlayerPos, ViewableState};

use crate::events::GameEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerMessage {
    Request(GameToPlayerRequest),
    Response(GameToPlayerResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerRequest {
    Notify(GameToPlayerNotification),
    RequestData(GameToPlayerRequestData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerNotification {
    Event(GameEvent),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerRequestData {
    RequestMainPhaseAction(RequestMainPhaseAction),
    RequestGameStart {
        state: ViewableState,
        pos: PlayerPos,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMainPhaseAction {
    pub state: ViewableState,
    pub playable_cards: Vec<PlayableCardSelector>,
    pub performable_basic_actions: Vec<BasicAction>,
    pub available_basic_action_costs: Vec<BasicActionCost>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerResponse {
    State(ViewableState),
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
    MainPhaseAction(ResponseMainPhaseAction),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMainPhaseAction {
    pub action: MainPhaseAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameRequest {
    RequestState,
    Surrender,
}
