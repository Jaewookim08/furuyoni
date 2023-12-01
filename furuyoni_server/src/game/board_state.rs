use crate::game::petals::Petals;
use crate::game::PlayerStates;
use furuyoni_lib::rules::events::UpdateBoardState;
use std::ops::Deref;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum InvalidBoardUpdateError {
    #[error("There was no card that matches the hand selector.")]
    HandSelectorOutOfBounds,
    #[error("Vigor has been pushed to go below 0.")]
    NegativeVigor,
}

pub(crate) struct BoardStateInner {
    pub distance: Petals,
    pub dust: Petals,
    pub player_states: PlayerStates,
}

pub(crate) struct BoardState {
    inner: BoardStateInner,
}

impl BoardState {
    pub fn new(distance: Petals, dust: Petals, player_states: PlayerStates) -> Self {
        Self {
            inner: BoardStateInner {
                distance,
                dust,
                player_states,
            },
        }
    }

    pub fn apply_update(
        &mut self,
        update: UpdateBoardState,
    ) -> Result<(), InvalidBoardUpdateError> {
        let board_state = &mut self.inner;

        match update {
            UpdateBoardState::TransferPetals { .. } => {
                todo!()
            }
            UpdateBoardState::AddToVigor { player, diff } => {
                const MAX_VIGOR: i32 = 2;
                const MIN_VIGOR: i32 = 0;

                let vigor = &mut board_state.player_states[player].vigor;

                let new = vigor.0 + diff;

                if new < MIN_VIGOR {
                    return Err(InvalidBoardUpdateError::NegativeVigor);
                }

                vigor.0 = std::cmp::min(MAX_VIGOR, new);
            }
            UpdateBoardState::DiscardCard { player, selector } => {
                let player_state = &mut board_state.player_states[player];
                let hand = &mut player_state.hand;

                if selector.0 > hand.len() {
                    return Err(InvalidBoardUpdateError::HandSelectorOutOfBounds);
                }
                let card = hand.remove(selector.0);

                player_state.discard_pile.push(card)
            }
        }

        Ok(())
    }
}

impl Deref for BoardState {
    type Target = BoardStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
