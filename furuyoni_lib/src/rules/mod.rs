pub mod attack;
pub mod cards;
pub mod condition;
pub mod effects;
pub mod events;
pub mod player_actions;
pub mod states;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Phase {
    Beginning,
    Main,
    End,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Debug)]
pub enum PlayerPos {
    P1,
    P2,
}

impl PlayerPos {
    pub fn other(&self) -> Self {
        match self {
            PlayerPos::P1 => Self::P2,
            PlayerPos::P2 => Self::P1,
        }
    }

    pub fn iter() -> impl Iterator<Item = PlayerPos> {
        [PlayerPos::P1, PlayerPos::P2].into_iter()
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub enum PetalPosition {
    Distance,
    Dust,
    Aura(PlayerPos),
    Flare(PlayerPos),
    Life(PlayerPos),
    // Todo: 부여패.
}
