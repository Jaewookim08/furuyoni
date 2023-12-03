use crate::rules::attack::Attack;
use serde::{Deserialize, Serialize};

mod yurina;

pub type Cards = Vec<Card>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
