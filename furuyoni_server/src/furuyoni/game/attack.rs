use super::effects::Effect;
use crate::furuyoni::game::condition::Condition;

pub type Damage = Option<u32>;

pub struct AttackDamage {
    pub aura_damage: Damage,
    pub life_damage: Damage,
}

pub enum DamageModifier {}

pub struct Attack {
    pub base_damage: AttackDamage,
    pub range: &'static [i32],
    pub after_attack: &'static [Effect],
    pub damage_modifiers: &'static [(Condition, DamageModifier)],
}
