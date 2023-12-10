use crate::game::states::player_state::PlayerState;
use furuyoni_lib::rules::cards::{Cards, CardsPosition};
use furuyoni_lib::rules::events::UpdateGameState;
use furuyoni_lib::rules::states::{Petals, PetalsPosition, Phase, PlayersData};
use furuyoni_lib::rules::PlayerPos;
use std::ops::Deref;
use thiserror::Error;

pub(crate) type PlayerStates = PlayersData<PlayerState>;

#[derive(Debug, Clone)]
pub(crate) struct GameStateInner {
    pub turn: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
    pub distance: Petals,
    pub dust: Petals,
    pub player_states: PlayerStates,
}

#[derive(Debug, Clone)]
pub(crate) struct GameState {
    inner: GameStateInner,
}

#[derive(Debug, Error)]
pub(crate) enum InvalidGameUpdateError {
    #[error("The given card selector's index was over the size of the cards")]
    CardSelectorOutOfBounds,
    #[error("Vigor has been pushed to go below 0 or above 2.")]
    InvalidVigor,
    #[error(
        "Invalid petal transfer: the transfer will result in negative or over-max petal value."
    )]
    InvalidPetalTransfer,
    #[error("An update only for state views have been requested.")]
    UpdateOnlyForView,
}

impl GameStateInner {
    fn get_petals_mut(&mut self, petal_position: PetalsPosition) -> &'_ mut Petals {
        match petal_position {
            PetalsPosition::Distance => &mut self.distance,
            PetalsPosition::Dust => &mut self.dust,
            PetalsPosition::Aura(player) => &mut self.player_states[player].aura,
            PetalsPosition::Flare(player) => &mut self.player_states[player].flare,
            PetalsPosition::Life(player) => &mut self.player_states[player].life,
        }
    }

    pub fn get_petals(&self, petal_position: PetalsPosition) -> &Petals {
        match petal_position {
            PetalsPosition::Distance => &self.distance,
            PetalsPosition::Dust => &self.dust,
            PetalsPosition::Aura(player) => &self.player_states[player].aura,
            PetalsPosition::Flare(player) => &self.player_states[player].flare,
            PetalsPosition::Life(player) => &self.player_states[player].life,
        }
    }

    fn get_cards_mut(&mut self, cards_position: CardsPosition) -> &mut Cards {
        match cards_position {
            CardsPosition::Hand(p) => &mut self.player_states[p].hand,
            CardsPosition::Playing(p) => &mut self.player_states[p].hand,
            CardsPosition::Deck(p) => &mut self.player_states[p].deck,
            CardsPosition::Enhancements(p) => &mut self.player_states[p].enhancements,
            CardsPosition::Played(p) => &mut self.player_states[p].played_pile,
            CardsPosition::Discards(p) => &mut self.player_states[p].discard_pile,
        }
    }

    pub fn get_cards(&self, cards_position: CardsPosition) -> &Cards {
        match cards_position {
            CardsPosition::Hand(p) => &self.player_states[p].hand,
            CardsPosition::Playing(p) => &self.player_states[p].hand,
            CardsPosition::Deck(p) => &self.player_states[p].deck,
            CardsPosition::Enhancements(p) => &self.player_states[p].enhancements,
            CardsPosition::Played(p) => &self.player_states[p].played_pile,
            CardsPosition::Discards(p) => &self.player_states[p].discard_pile,
        }
    }
}

impl GameState {
    pub fn new(
        turn: u32,
        turn_player: PlayerPos,
        phase: Phase,
        distance: Petals,
        dust: Petals,
        player_states: PlayerStates,
    ) -> Self {
        Self {
            inner: GameStateInner {
                turn,
                turn_player,
                phase,
                distance,
                dust,
                player_states,
            },
        }
    }

    pub fn apply_update(&mut self, update: UpdateGameState) -> Result<(), InvalidGameUpdateError> {
        let state = &mut self.inner;

        match update {
            UpdateGameState::TransferPetals { from, to, amount } => {
                let from_petals = state.get_petals_mut(from);
                let from_new = from_petals
                    .count
                    .checked_sub(amount)
                    .ok_or(InvalidGameUpdateError::InvalidPetalTransfer)?;

                let to_petals = state.get_petals_mut(to);
                let to_new = to_petals.count + amount;
                if let Some(max) = to_petals.max
                    && to_new > max
                {
                    return Err(InvalidGameUpdateError::InvalidPetalTransfer);
                }

                state.get_petals_mut(from).count = from_new;
                state.get_petals_mut(to).count = to_new;
            }
            UpdateGameState::AddToVigor { player, diff } => {
                let vigor = &mut state.player_states[player].vigor;
                vigor.0 += diff;
                if vigor.0 < 0 || vigor.0 > 2 {
                    return Err(InvalidGameUpdateError::InvalidVigor);
                }
            }
            UpdateGameState::SetTurn { turn, turn_player } => {
                state.turn = turn;
                state.turn_player = turn_player;
            }
            UpdateGameState::SetPhase(phase) => {
                state.phase = phase;
            }
            UpdateGameState::TransferCard { from, to } => {
                let cards_from = self.inner.get_cards_mut(from.cards_position());

                let index_from = from.index(cards_from.len());
                if index_from >= cards_from.len() || index_from < 0 {
                    return Err(InvalidGameUpdateError::CardSelectorOutOfBounds);
                }

                let taken = cards_from.remove(index_from);

                let cards_to = self.inner.get_cards_mut(to.cards_position());
                let index_to = to.index(cards_to.len());

                if index_to > cards_to.len() || index_to < 0 {
                    return Err(InvalidGameUpdateError::CardSelectorOutOfBounds);
                }
                cards_to.insert(index_to, taken);
            }
            UpdateGameState::TransferCardFromHidden { .. } => {
                return Err(InvalidGameUpdateError::UpdateOnlyForView)
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
// DerefMut is not implemented. User should use apply_update to change state.
