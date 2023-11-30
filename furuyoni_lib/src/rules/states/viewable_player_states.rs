use crate::rules::cards::Card;
use crate::rules::states::players_data::PlayersData;
use crate::rules::{Phase, PlayerPos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ViewablePlayerState {
    SelfState(ViewableSelfState),
    Opponent(ViewableOpponentState),
}

impl ViewablePlayerState {
    pub fn get_vigor(&self) -> i32 {
        match self {
            ViewablePlayerState::SelfState(s) => s.vigor,
            ViewablePlayerState::Opponent(o) => o.vigor,
        }
    }
    pub fn get_vigor_mut(&mut self) -> &mut i32 {
        match self {
            ViewablePlayerState::SelfState(s) => &mut s.vigor,
            ViewablePlayerState::Opponent(o) => &mut o.vigor,
        }
    }

    pub fn get_aura(&self) -> u32 {
        match self {
            ViewablePlayerState::SelfState(s) => s.aura,
            ViewablePlayerState::Opponent(o) => o.aura,
        }
    }
    pub fn get_aura_mut(&mut self) -> &mut u32 {
        match self {
            ViewablePlayerState::SelfState(s) => &mut s.aura,
            ViewablePlayerState::Opponent(o) => &mut o.aura,
        }
    }

    pub fn get_life(&self) -> u32 {
        match self {
            ViewablePlayerState::SelfState(s) => s.life,
            ViewablePlayerState::Opponent(o) => o.life,
        }
    }
    pub fn get_life_mut(&mut self) -> &mut u32 {
        match self {
            ViewablePlayerState::SelfState(s) => &mut s.life,
            ViewablePlayerState::Opponent(o) => &mut o.life,
        }
    }

    pub fn get_flare(&self) -> u32 {
        match self {
            ViewablePlayerState::SelfState(s) => s.flare,
            ViewablePlayerState::Opponent(o) => o.flare,
        }
    }
    pub fn get_flare_mut(&mut self) -> &mut u32 {
        match self {
            ViewablePlayerState::SelfState(s) => &mut s.flare,
            ViewablePlayerState::Opponent(o) => &mut o.flare,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ViewableOpponentState {
    pub hand_count: usize,
    pub deck_count: usize,
    pub enhancements: Vec<Card>,
    pub played_pile: Vec<Card>,
    pub discard_pile_count: usize,

    pub vigor: i32,
    pub aura: u32,
    pub life: u32,
    pub flare: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ViewableSelfState {
    pub hands: Vec<Card>,
    pub deck_count: usize,
    pub enhancements: Vec<Card>,
    pub played_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,

    pub vigor: i32,
    pub aura: u32,
    pub life: u32,
    pub flare: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ViewableState {
    pub turn_number: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
    pub distance: u32,
    pub dust: u32,
    pub player_states: ViewablePlayerStates,
}

pub type ViewablePlayerStates = PlayersData<ViewablePlayerState>;
