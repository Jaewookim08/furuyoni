use crate::attack::Attack;
use serde::{Deserialize, Serialize};

mod yurina;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub enum CardBack {
    Normal,
    Special,
}

pub struct CardData {
    pub basic_data: CardBasicData,
    pub play_data: CardPlayData,
}

pub struct CardBasicData {
    pub card_back: CardBack,
    pub id_str: &'static str,
}

pub enum CardPlayData {
    AttackCard(AttackCard),
}

pub struct AttackCard {
    pub attack: Attack,
}
