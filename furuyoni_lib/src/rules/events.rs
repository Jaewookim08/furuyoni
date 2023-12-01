use crate::rules::player_actions::{BasicAction, HandSelector};
use crate::rules::{PetalPosition, Phase, PlayerPos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum UpdateBoardState {
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

#[derive(Serialize, Deserialize, Debug)]
pub enum UpdatePhaseState {
    SetTurn { turn: u32, turn_player: PlayerPos },
    SetPhase(Phase),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEvent {
    BoardUpdated(UpdateBoardState),
    PhaseUpdated(UpdatePhaseState),
    PerformBasicAction {
        player: PlayerPos,
        action: BasicAction,
    }, // Todo: card play events, etc...
       // Todo: 메인페이즈 BasicAction(cost 지불 + performBasicAction)을 따로 넣을까. 굳이? 나중에 필요하면.
}
