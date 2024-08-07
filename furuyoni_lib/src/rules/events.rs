use crate::rules::cards::{ Card, CardsPosition };
use crate::rules::player_actions::BasicAction;
use crate::rules::states::PetalsPosition;
use crate::rules::states::Phase;
use crate::rules::{ GameResult, PlayerPos };
use serde::{ Deserialize, Serialize };

use super::cards::CardSelector;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum UpdateGameState {
    SetTurn {
        turn: u32,
        turn_player: PlayerPos,
    },
    SetPhase(Phase),
    TransferPetals {
        from: PetalsPosition,
        to: PetalsPosition,
        amount: u32,
    },
    AddToVigor {
        player: PlayerPos,
        diff: i32,
    },
    TransferCard {
        from: CardSelector,
        to: CardSelector,
    },
    TransferCardFromHidden {
        from: CardsPosition,
        to: CardSelector,
        card: Option<Card>,
    },
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum GameEvent {
    StateUpdated(UpdateGameState),
    PerformBasicAction {
        player: PlayerPos,
        action: BasicAction,
    },
    GameEnd {
        result: GameResult,
    }, // Todo: card play events, etc...
    // Todo: 메인페이즈 BasicAction(cost 지불 + performBasicAction)을 따로 넣을까. 굳이? 나중에 필요하면.
}
