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
pub struct CardSelector {
    pub position: CardsPosition,
    pub index: usize,
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
