use std::collections::VecDeque;
use furuyoni_lib::cards::Card;
use furuyoni_lib::rules::{ViewableOpponentState, ViewableSelfState};
use crate::game::petals::Petals;
use crate::game::Vigor;

#[derive(Debug)]
pub(crate) struct PlayerState {
    pub hand: Vec<Card>,
    pub deck: VecDeque<Card>,
    pub enhancements: Vec<Card>,
    pub played_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,

    pub vigor: Vigor,
    pub aura: Petals,
    pub life: Petals,
    pub flare: Petals,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            hand: vec![],
            deck: VecDeque::default(),
            enhancements: vec![],
            played_pile: vec![],
            discard_pile: vec![],
            vigor: Vigor(0),
            aura: Petals::new(3),
            life: Petals::new(10),
            flare: Petals::new(0),
        }
    }
}


impl From<&PlayerState> for ViewableOpponentState {
    fn from(player_state: &PlayerState) -> Self {
        ViewableOpponentState {
            hand_count: player_state.hand.len(),
            deck_count: player_state.deck.len(),
            enhancements: player_state.enhancements.clone(),
            played_pile: player_state.played_pile.clone(),
            discard_pile_count: player_state.discard_pile.len(),

            vigor: player_state.vigor.0,
            aura: player_state.aura.get_count(),
            life: player_state.life.get_count(),
            flare: player_state.flare.get_count(),
        }
    }
}

impl From<&PlayerState> for ViewableSelfState {
    fn from(player_state: &PlayerState) -> Self {
        ViewableSelfState {
            hands: player_state.hand.clone(),
            deck_count: player_state.deck.len(),
            enhancements: player_state.enhancements.clone(),
            played_pile: player_state.played_pile.clone(),
            discard_pile: player_state.discard_pile.clone(),

            vigor: player_state.vigor.0,
            aura: player_state.aura.get_count(),
            life: player_state.life.get_count(),
            flare: player_state.flare.get_count(),
        }
    }
}