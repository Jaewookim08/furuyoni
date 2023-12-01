use crate::game::states::phase_state::PhaseState;
use crate::game::states::BoardState;
use furuyoni_lib::rules::events::UpdateGameState;
use std::ops::Deref;
use thiserror::Error;

// Todo: Merge phase_state and board_state.
pub(crate) struct GameStateInner {
    pub phase_state: PhaseState,
    pub board_state: BoardState,
}

pub(crate) struct GameState {
    inner: GameStateInner,
}

#[derive(Debug, Error)]
pub(crate) enum InvalidGameUpdateError {
    #[error("There was no card that matches the hand selector.")]
    HandSelectorOutOfBounds,
    #[error("Vigor has been pushed to go below 0.")]
    NegativeVigor,
    #[error(
        "Invalid petal transfer: the transfer will result in negative or over-max petal value."
    )]
    InvalidPetalTransfer,
}

impl GameState {
    pub fn new(phase_state: PhaseState, board_state: BoardState) -> Self {
        Self {
            inner: GameStateInner {
                phase_state,
                board_state,
            },
        }
    }

    pub fn apply_update(&mut self, update: &UpdateGameState) -> Result<(), InvalidGameUpdateError> {
        let GameStateInner {
            phase_state,
            board_state,
        } = &mut self.inner;

        match *update {
            UpdateGameState::TransferPetals { from, to, amount } => {
                let from_petals = board_state.get_petals_mut(from);
                let from_new = from_petals
                    .count
                    .checked_sub(amount)
                    .ok_or(InvalidGameUpdateError::InvalidPetalTransfer)?;

                let to_petals = board_state.get_petals_mut(to);
                let to_new = to_petals.count + amount;
                if let Some(max) = to_petals.max && to_new > max {
                    return Err(InvalidGameUpdateError::InvalidPetalTransfer);
                }

                board_state.get_petals_mut(from).count = from_new;
                board_state.get_petals_mut(to).count = to_new;
            }
            UpdateGameState::AddToVigor { player, diff } => {
                const MAX_VIGOR: i32 = 2;
                const MIN_VIGOR: i32 = 0;

                let vigor = &mut board_state.player_states[player].vigor;

                let new = vigor.0 + diff;

                if new < MIN_VIGOR {
                    return Err(InvalidGameUpdateError::NegativeVigor);
                }

                vigor.0 = std::cmp::min(MAX_VIGOR, new);
            }
            UpdateGameState::DiscardCard { player, selector } => {
                let player_state = &mut board_state.player_states[player];
                let hand = &mut player_state.hand;

                if selector.0 > hand.len() {
                    return Err(InvalidGameUpdateError::HandSelectorOutOfBounds);
                }
                let card = hand.remove(selector.0);

                player_state.discard_pile.push(card)
            }
            UpdateGameState::SetTurn { turn, turn_player } => {
                phase_state.turn = turn;
                phase_state.turn_player = turn_player;
            }
            UpdateGameState::SetPhase(phase) => {
                phase_state.phase = phase;
            }
        }

        Ok(())
    }
}

impl Deref for GameState {
    type Target = GameStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
