mod states;
use serde::{Deserialize, Serialize};

pub use {
    states::ViewableOpponentState, states::ViewablePlayerState, states::ViewablePlayerStates,
    states::ViewableSelfState, states::ViewableState,
};

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
}
