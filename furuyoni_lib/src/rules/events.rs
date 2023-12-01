use crate::rules::player_actions::{BasicAction, HandSelector};
use crate::rules::{PetalPosition, Phase, PlayerPos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UpdateGameState {
    SetTurn {
        turn: u32,
        turn_player: PlayerPos,
    },
    SetPhase(Phase),
    TransferPetals {
        from: PetalPosition,
        to: PetalPosition,
        amount: u32,
    },
    AddToVigor {
        player: PlayerPos,
        diff: i32,
    },
    DiscardCard {
        player: PlayerPos,
        selector: HandSelector,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameEvent {
    StateUpdated(UpdateGameState),
    PerformBasicAction {
        player: PlayerPos,
        action: BasicAction,
    }, // Todo: card play events, etc...
       // Todo: 메인페이즈 BasicAction(cost 지불 + performBasicAction)을 따로 넣을까. 굳이? 나중에 필요하면.
}
