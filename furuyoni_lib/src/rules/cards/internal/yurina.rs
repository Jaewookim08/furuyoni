use super::*;
use crate::rules::attack::{Attack, AttackDamage};

pub const SLASH: CardData = CardData {
    basic_data: CardBasicData {
        card_back: CardBack::Normal,
        id_str: "NA-01-yurina-O-N1",
    },
    play_data: CardPlayData::AttackCard(AttackCard {
        attack: Attack {
            base_damage: AttackDamage {
                aura_damage: Some(3),
                life_damage: Some(1),
            },
            range: &[3, 4],
            after_attack: &[],
            damage_modifiers: &[],
        },
    }),
};

pub const BRANDISH: CardData = CardData {
    basic_data: CardBasicData {
        card_back: CardBack::Normal,
        id_str: "NA-02-yurina-O-N2",
    },
    play_data: CardPlayData::AttackCard(AttackCard {
        attack: Attack {
            base_damage: AttackDamage {
                aura_damage: Some(2),
                life_damage: Some(2),
            },
            range: &[3],
            after_attack: &[],
            damage_modifiers: &[],
        },
    }),
};
