use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::{PlayerPos, ViewableState};

use crate::events::GameEvent;
use serde::{Deserialize, Serialize};

use super::base::WithRequestId;

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerMessage {
    Request(GameToPlayerRequest),
    Response(GameToPlayerResponseFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerRequest {
    Notify(GameToPlayerNotification),
    RequestData(GameToPlayerRequestDataFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerNotification {
    Event(GameEvent),
}

pub type GameToPlayerRequestDataFrame = WithRequestId<GameToPlayerRequestData>;

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

pub type GameToPlayerResponseFrame = WithRequestId<GameToPlayerResponse>;

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerResponse {
    State(ViewableState),
    Ack,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameMessage {
    Response(PlayerToGameResponseFrame),
    Request(PlayerToGameRequestFrame),
}

pub type PlayerToGameResponseFrame = WithRequestId<PlayerToGameResponse>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameResponse {
    AcknowledgeGameStart,
    MainPhaseAction(ResponseMainPhaseAction),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMainPhaseAction {
    pub action: MainPhaseAction,
}

pub type PlayerToGameRequestFrame = WithRequestId<PlayerToGameRequest>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameRequest {
    RequestState,
    Surrender,
}
