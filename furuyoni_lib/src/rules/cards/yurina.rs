use super::*;
use crate::rules::attack::{Attack, AttackDamage};

pub const SLASH: CardData = CardData {
    id_str: "NA-01-yurina-O-N1",
    card_type: CardType::Normal,
    card_sub_type: CardSubType::None,
    play_data: CardPlayData::AttackCard {
        attack: Attack {
            base_damage: AttackDamage {
                aura_damage: Some(3),
                life_damage: Some(1),
            },
            range: &[3, 4],
            after_attack: &[],
            damage_modifiers: &[],
        },
    },
};

pub const BRANDISH: CardData = CardData {
    id_str: "NA-02-yurina-O-N2",
    card_type: CardType::Normal,
    card_sub_type: CardSubType::None,
    play_data: CardPlayData::AttackCard {
        attack: Attack {
            base_damage: AttackDamage {
                aura_damage: Some(2),
                life_damage: Some(2),
            },
            range: &[3],
            after_attack: &[],
            damage_modifiers: &[],
        },
    },
};
