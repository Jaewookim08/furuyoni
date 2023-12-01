use crate::game::PlayerStates;
use furuyoni_lib::rules::events::UpdateBoardState;
use furuyoni_lib::rules::states::petals::Petals;
use furuyoni_lib::rules::PetalPosition;
use std::ops::Deref;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum InvalidBoardUpdateError {
    #[error("There was no card that matches the hand selector.")]
    HandSelectorOutOfBounds,
    #[error("Vigor has been pushed to go below 0.")]
    NegativeVigor,
    #[error(
        "Invalid petal transfer: the transfer will result in negative or over-max petal value."
    )]
    InvalidPetalTransfer,
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
            UpdateBoardState::TransferPetals { from, to, amount } => {
                let from_petals = self.get_petals_mut(from);
                let from_new = from_petals
                    .count
                    .checked_sub(amount)
                    .ok_or(InvalidBoardUpdateError::InvalidPetalTransfer)?;

                let to_petals = self.get_petals_mut(to);
                let to_new = to_petals.count + amount;
                if let Some(max) = to_petals.max && to_new > max {
                    return Err(InvalidBoardUpdateError::InvalidPetalTransfer);
                }

                self.get_petals_mut(from).count = from_new;
                self.get_petals_mut(to).count = to_new;
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

    fn get_petals_mut(&mut self, petal_position: PetalPosition) -> &'_ mut Petals {
        let inner = &mut self.inner;
        match petal_position {
            PetalPosition::Distance => &mut inner.distance,
            PetalPosition::Dust => &mut inner.dust,
            PetalPosition::Aura(player) => &mut inner.player_states[player].aura,
            PetalPosition::Flare(player) => &mut inner.player_states[player].flare,
            PetalPosition::Life(player) => &mut inner.player_states[player].life,
        }
    }
}

impl Deref for BoardState {
    type Target = BoardStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
