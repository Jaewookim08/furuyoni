use crate::game::states::player_state::PlayerState;
use furuyoni_lib::rules::events::UpdateGameState;
use furuyoni_lib::rules::states::{Petals, PetalsPosition, Phase, PlayersData};
use furuyoni_lib::rules::PlayerPos;
use std::ops::Deref;
use thiserror::Error;

pub(crate) type PlayerStates = PlayersData<PlayerState>;

pub(crate) struct GameStateInner {
    pub turn: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
    pub distance: Petals,
    pub dust: Petals,
    pub player_states: PlayerStates,
}

pub(crate) struct GameState {
    inner: GameStateInner,
}

#[derive(Debug, Error)]
pub(crate) enum InvalidGameUpdateError {
    #[error("There was no card that matches the hand selector.")]
    HandSelectorOutOfBounds,
    #[error("Vigor has been pushed to go below 0 or above 2.")]
    InvalidVigor,
    #[error(
        "Invalid petal transfer: the transfer will result in negative or over-max petal value."
    )]
    InvalidPetalTransfer,
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

    pub fn apply_update(&mut self, update: &UpdateGameState) -> Result<(), InvalidGameUpdateError> {
        let state = &mut self.inner;

        match *update {
            UpdateGameState::TransferPetals { from, to, amount } => {
                let from_petals = state.get_petals_mut(from);
                let from_new = from_petals
                    .count
                    .checked_sub(amount)
                    .ok_or(InvalidGameUpdateError::InvalidPetalTransfer)?;

                let to_petals = state.get_petals_mut(to);
                let to_new = to_petals.count + amount;
                if let Some(max) = to_petals.max && to_new > max {
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
            UpdateGameState::DiscardCard { player, selector } => {
                let player_state = &mut state.player_states[player];
                let hand = &mut player_state.hand;

                if selector.0 > hand.len() {
                    return Err(InvalidGameUpdateError::HandSelectorOutOfBounds);
                }
                let card = hand.remove(selector.0);

                player_state.discard_pile.push(card)
            }
            UpdateGameState::SetTurn { turn, turn_player } => {
                state.turn = turn;
                state.turn_player = turn_player;
            }
            UpdateGameState::SetPhase(phase) => {
                state.phase = phase;
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
