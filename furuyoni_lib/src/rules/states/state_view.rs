use crate::rules::cards::{Card, Cards, CardsPosition};
use crate::rules::events::UpdateGameState;
use crate::rules::states::petals::Petals;
use crate::rules::states::players_data::PlayersData;
use crate::rules::states::{PetalsPosition, Phase};
use crate::rules::PlayerPos;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CardsView {
    Open { cards: Cards },
    Hidden { length: usize },
}

impl CardsView {
    pub fn from(cards: &Cards, is_owner: bool) -> Self {
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

    fn get_ref_mut(&mut self) -> CardsViewMutRef {
        match self {
            CardsView::Open { cards } => CardsViewMutRef::Open { cards },
            CardsView::Hidden { length } => CardsViewMutRef::Hidden { length },
        }
    }

    fn get_ref(&self) -> CardsViewRef {
        match self {
            CardsView::Open { cards } => CardsViewRef::Open { cards },
            CardsView::Hidden { length } => CardsViewRef::Hidden { length },
        }
    }
}

#[derive(Debug)]
enum CardsViewMutRef<'a> {
    Open { cards: &'a mut Cards },
    Hidden { length: &'a mut usize },
}

impl<'a> From<&'a mut Cards> for CardsViewMutRef<'a> {
    fn from(cards: &'a mut Cards) -> Self {
        CardsViewMutRef::Open { cards }
    }
}

impl<'a> CardsViewMutRef<'a> {
    pub fn len(&self) -> usize {
        match self {
            CardsViewMutRef::Open { cards } => cards.len(),
            CardsViewMutRef::Hidden { length } => **length,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CardsViewRef<'a> {
    Open { cards: &'a Cards },
    Hidden { length: &'a usize },
}

impl<'a> CardsViewRef<'a> {
    pub fn len(&self) -> usize {
        match *self {
            CardsViewRef::Open { cards } => cards.len(),
            CardsViewRef::Hidden { length } => *length,
        }
    }
}

impl<'a> From<&'a Cards> for CardsViewRef<'a> {
    fn from(cards: &'a Cards) -> Self {
        CardsViewRef::Open { cards }
    }
}

impl<'a> CardsViewMutRef<'a> {
    fn insert_card(&mut self, index: usize, card: Card) -> Result<(), InvalidGameViewUpdateError> {
        match self {
            CardsViewMutRef::Open { cards: cards_to } => {
                if index > cards_to.len() {
                    return Err(InvalidGameViewUpdateError::CardSelectorOutOfBounds);
                }
                cards_to.insert(index, card);
            }
            CardsViewMutRef::Hidden { length } => **length += 1,
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerStateView {
    pub hand: CardsView,
    pub deck: CardsView,
    pub playing: Cards,
    pub enhancements: Cards,
    pub played_pile: Cards,
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
    #[error("The given card selector's index was over the size of the cards.")]
    CardSelectorOutOfBounds,
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
    pub fn petals(&self, petal_position: PetalsPosition) -> &Petals {
        match petal_position {
            PetalsPosition::Distance => &self.distance,
            PetalsPosition::Dust => &self.dust,
            PetalsPosition::Aura(player) => &self.player_states[player].aura,
            PetalsPosition::Flare(player) => &self.player_states[player].flare,
            PetalsPosition::Life(player) => &self.player_states[player].life,
        }
    }

    fn petals_mut(&mut self, petal_position: PetalsPosition) -> &'_ mut Petals {
        match petal_position {
            PetalsPosition::Distance => &mut self.distance,
            PetalsPosition::Dust => &mut self.dust,
            PetalsPosition::Aura(player) => &mut self.player_states[player].aura,
            PetalsPosition::Flare(player) => &mut self.player_states[player].flare,
            PetalsPosition::Life(player) => &mut self.player_states[player].life,
        }
    }

    pub fn cards_view(&self, cards_position: CardsPosition) -> CardsViewRef {
        match cards_position {
            CardsPosition::Hand(p) => self.player_states[p].hand.get_ref(),
            CardsPosition::Playing(p) => (&self.player_states[p].playing).into(),
            CardsPosition::Deck(p) => self.player_states[p].deck.get_ref(),
            CardsPosition::Enhancements(p) => (&self.player_states[p].enhancements).into(),
            CardsPosition::Played(p) => (&self.player_states[p].played_pile).into(),
            CardsPosition::Discards(p) => self.player_states[p].discard_pile.get_ref(),
        }
    }

    fn cards_view_mut(&mut self, cards_position: CardsPosition) -> CardsViewMutRef {
        match cards_position {
            CardsPosition::Hand(p) => self.player_states[p].hand.get_ref_mut(),
            CardsPosition::Playing(p) => (&mut self.player_states[p].playing).into(),
            CardsPosition::Deck(p) => self.player_states[p].deck.get_ref_mut(),
            CardsPosition::Enhancements(p) => (&mut self.player_states[p].enhancements).into(),
            CardsPosition::Played(p) => (&mut self.player_states[p].played_pile).into(),
            CardsPosition::Discards(p) => self.player_states[p].discard_pile.get_ref_mut(),
        }
    }

    // Todo: client의 board가 apply_update하고 자기가 보여주는 board를 GameStateView로 뽑을 수 있도록.
    pub fn apply_update(
        &mut self,
        update: UpdateGameState,
    ) -> Result<(), InvalidGameViewUpdateError> {
        match update {
            UpdateGameState::TransferPetals { from, to, amount } => {
                let from_petals = self.petals_mut(from);
                from_petals.count = from_petals
                    .count
                    .checked_sub(amount)
                    .ok_or(InvalidGameViewUpdateError::InvalidPetalTransfer)?;

                let to_petals = self.petals_mut(to);
                to_petals.count += amount;
            }
            UpdateGameState::AddToVigor { player, diff } => {
                self.player_states[player].vigor += diff;
            }
            UpdateGameState::SetTurn { turn, turn_player } => {
                self.turn = turn;
                self.turn_player = turn_player;
            }
            UpdateGameState::SetPhase(phase) => {
                self.phase = phase;
            }
            UpdateGameState::TransferCard { from, to } => {
                let from_cards = match self.cards_view_mut(from.cards_position()) {
                    CardsViewMutRef::Open { cards } => cards,
                    CardsViewMutRef::Hidden { .. } => {
                        return Err(InvalidGameViewUpdateError::VisibilityMismatch);
                    }
                };

                let from_index = from.index(from_cards.len());

                if from_index >= from_cards.len() {
                    return Err(InvalidGameViewUpdateError::CardSelectorOutOfBounds);
                }
                let taken = from_cards.remove(from_index);

                let mut to_cards = self.cards_view_mut(to.cards_position());
                let to_index = to.index(to_cards.len());

                to_cards.insert_card(to_index, taken)?;
            }
            UpdateGameState::TransferCardFromHidden { from, to, card } => {
                let cards_from_len = match self.cards_view_mut(from) {
                    CardsViewMutRef::Open { .. } => {
                        return Err(InvalidGameViewUpdateError::VisibilityMismatch);
                    }
                    CardsViewMutRef::Hidden { length } => length,
                };

                *cards_from_len -= 1;

                let mut cards_to = self.cards_view_mut(to.cards_position());
                cards_to.insert_card(to.index(cards_to.len()), card.clone())?;
            }
        }

        Ok(())
    }
}
