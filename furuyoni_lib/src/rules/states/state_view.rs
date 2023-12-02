use crate::rules::cards::Card;
use crate::rules::events::UpdateGameState;
use crate::rules::states::petals::Petals;
use crate::rules::states::players_data::PlayersData;
use crate::rules::{PetalPosition, Phase, PlayerPos};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CardsView {
    Open { cards: Vec<Card> },
    Hidden { length: usize },
}

impl CardsView {
    pub fn from(cards: &Vec<Card>, is_owner: bool) -> Self {
        if is_owner {
            Self::Open {
                cards: cards.clone(),
            }
        } else {
            Self::Hidden {
                length: cards.len(),
            }
        }
    }

    pub fn push(&mut self, card: Card) {
        match self {
            CardsView::Open { cards } => cards.push(card),
            CardsView::Hidden { length } => *length += 1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerStateView {
    pub hand: CardsView,
    pub deck: CardsView,
    pub enhancements: Vec<Card>,
    pub played_pile: Vec<Card>,
    pub discard_pile: CardsView,

    pub vigor: i32,
    pub aura: Petals,
    pub life: Petals,
    pub flare: Petals,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StateView {
    pub turn: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
    pub distance: Petals,
    pub dust: Petals,
    pub player_states: PlayersData<PlayerStateView>,
}

pub type PlayerStateViews = PlayersData<PlayerStateView>;

#[derive(Debug, Error)]
pub enum InvalidGameViewUpdateError {
    #[error("There was no card that matches the hand selector.")]
    HandSelectorOutOfBounds,
    #[error("Vigor has been pushed to go below 0.")]
    NegativeVigor,
    #[error(
        "Invalid petal transfer: the transfer will result in negative or over-max petal value."
    )]
    InvalidPetalTransfer,
    #[error("The update and state's visibility(hidden/open) didn't match.")]
    VisibilityMismatch,
}

impl StateView {
    fn get_petals_mut(&mut self, petal_position: PetalPosition) -> &'_ mut Petals {
        match petal_position {
            PetalPosition::Distance => &mut self.distance,
            PetalPosition::Dust => &mut self.dust,
            PetalPosition::Aura(player) => &mut self.player_states[player].aura,
            PetalPosition::Flare(player) => &mut self.player_states[player].flare,
            PetalPosition::Life(player) => &mut self.player_states[player].life,
        }
    }

    // Todo: client의 board가 apply_update하고 자기가 보여주는 board를 GameStateView로 뽑을 수 있도록.
    pub fn apply_update(
        &mut self,
        update: &UpdateGameState,
    ) -> Result<(), InvalidGameViewUpdateError> {
        match *update {
            UpdateGameState::TransferPetals { from, to, amount } => {
                let from_petals = self.get_petals_mut(from);
                from_petals.count = from_petals
                    .count
                    .checked_sub(amount)
                    .ok_or(InvalidGameViewUpdateError::InvalidPetalTransfer)?;

                let to_petals = self.get_petals_mut(to);
                to_petals.count += amount;
            }
            UpdateGameState::AddToVigor { player, diff } => {
                self.player_states[player].vigor += diff;
            }
            UpdateGameState::DiscardCard { player, selector } => {
                let player_state = &mut self.player_states[player];
                let hand = &mut player_state.hand;

                match hand {
                    CardsView::Open { cards } => {
                        if selector.0 > cards.len() {
                            return Err(InvalidGameViewUpdateError::HandSelectorOutOfBounds);
                        }
                        let card = cards.remove(selector.0);

                        player_state.discard_pile.push(card)
                    }
                    CardsView::Hidden { .. } => {
                        return Err(InvalidGameViewUpdateError::VisibilityMismatch)
                    }
                }
            }
            UpdateGameState::SetTurn { turn, turn_player } => {
                self.turn = turn;
                self.turn_player = turn_player;
            }
            UpdateGameState::SetPhase(phase) => {
                self.phase = phase;
            }
        }

        Ok(())
    }
}
