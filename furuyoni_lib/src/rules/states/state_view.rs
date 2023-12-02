use crate::rules::cards::Card;
use crate::rules::states::petals::Petals;
use crate::rules::states::players_data::PlayersData;
use crate::rules::{Phase, PlayerPos};
use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerStateView {
    pub hands: CardsView,
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
    pub turn_number: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
    pub distance: Petals,
    pub dust: Petals,
    pub player_states: PlayersData<PlayerStateView>,
}

pub type PlayerStateViews = PlayersData<PlayerStateView>;
