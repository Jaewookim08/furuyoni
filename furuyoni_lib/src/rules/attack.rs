use crate::rules::condition::Condition;
use crate::rules::effects::Effect;

pub type Damage = Option<u32>;

#[derive(Debug, Copy, Clone)]
pub struct AttackDamage {
    pub aura_damage: Damage,
    pub life_damage: Damage,
}

#[derive(Debug, Copy, Clone)]
pub enum DamageModifier {}

#[derive(Debug, Copy, Clone)]
pub struct Attack {
    pub base_damage: AttackDamage,
    pub range: &'static [i32],
    pub after_attack: &'static [Effect],
    pub damage_modifiers: &'static [(Condition, DamageModifier)],
}
