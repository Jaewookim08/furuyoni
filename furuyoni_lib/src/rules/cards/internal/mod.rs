use crate::rules::attack::Attack;
use crate::rules::PlayerPos;
use serde::{Deserialize, Serialize};

mod yurina;

pub type Cards = Vec<Card>;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum CardsPosition {
    Hand(PlayerPos),
    Playing(PlayerPos),
    Deck(PlayerPos),
    Enhancements(PlayerPos),
    Played(PlayerPos),
    Discards(PlayerPos),
}

impl CardsPosition {
    pub fn get_player_pos(&self) -> PlayerPos {
        match self {
            CardsPosition::Hand(p)
            | CardsPosition::Playing(p)
            | CardsPosition::Deck(p)
            | CardsPosition::Enhancements(p)
            | CardsPosition::Played(p)
            | CardsPosition::Discards(p) => *p,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum CardSelector {
    PushLast(CardsPosition),
    Last(CardsPosition),
    First(CardsPosition),
    Index {
        position: CardsPosition,
        index: usize,
    },
}

impl CardSelector {
    pub fn cards_position(self) -> CardsPosition {
        match self {
            CardSelector::PushLast(p)
            | CardSelector::Last(p)
            | CardSelector::First(p)
            | CardSelector::Index { position: p, .. } => p,
        }
    }

    pub fn index(self, cards_len: usize) -> usize {
        match self {
            CardSelector::Last(_) => cards_len - 1,
            CardSelector::First(_) => 0,
            CardSelector::Index { index, .. } => index,
            CardSelector::PushLast(_) => cards_len,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Card {
    Slash,
    Brandish,
}

impl Card {
    fn data(&self) -> &'static CardData {
        match &self {
            Card::Slash => &yurina::SLASH,
            Card::Brandish => &yurina::BRANDISH,
        }
    }
}

pub enum CardType {
    Normal,
    Special { flare_cost: u32 },
}

pub struct CardData {
    pub id_str: &'static str,
    pub card_type: CardType,
    pub card_sub_type: CardSubType,
    pub play_data: CardPlayData,
}

pub enum CardPlayData {
    AttackCard { attack: Attack },
}

pub enum CardSubType {
    None,
    Reaction,
    Throughout,
}
