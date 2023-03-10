use crate::player_actions::BasicAction;
use crate::rules::PlayerPos;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEvent {
    DoBasicAction {
        pos: PlayerPos,
        action: BasicAction,
        amount: i32,
    },
    WithCost {
        pos: PlayerPos,
        cost: EventCost,
        event: Box<GameEvent>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventCost {
    Vigor,
}
