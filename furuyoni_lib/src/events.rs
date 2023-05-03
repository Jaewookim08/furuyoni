use crate::player_actions::BasicAction;
use crate::rules::PlayerPos;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEvent {
    DoBasicAction {
        pos: PlayerPos,
        cost: Option<EventCost>,
        action: BasicAction,
        amount: i32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventCost {
    Vigor,
}
