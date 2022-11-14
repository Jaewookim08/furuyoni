mod yurina;


use super::Game;
use crate::furuyoni::game_runner::attack::Attack;


pub enum Card {
    Slash,
    Brandish,
}

impl Card {
    fn data(&self) -> &'static CardData {
        match &self {
            Card::Slash => { &yurina::SLASH }
            Card::Brandish => { &yurina::BRANDISH }
        }
    }
}

pub enum CardBack {
    Normal,
    Special,
}


pub struct CardData {
    basic_data: CardBasicData,
    play_data: CardPlayData,
}

struct Context {}

struct CardBasicData {
    card_back: CardBack,
    id_str: &'static str,
}

enum CardPlayData {
    AttackCard(AttackCard)
}

struct AttackCard {
    attack: Attack,
}