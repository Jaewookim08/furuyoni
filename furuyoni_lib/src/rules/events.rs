use crate::rules::player_actions::BasicAction;
use crate::rules::{PetalPosition, PlayerPos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameStateUpdatedEvent {
    PetalsTransferred {
        from: PetalPosition,
        to: PetalPosition,
        amount: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEvent {
    StateUpdated(GameStateUpdatedEvent),
    PerformBasicAction {
        player: PlayerPos,
        action: BasicAction,
    }, // Todo: card play events, etc...
}
