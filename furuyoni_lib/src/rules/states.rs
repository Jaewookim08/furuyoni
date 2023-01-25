use crate::cards::Card;
use crate::players::PlayerData;
use crate::rules::{Phase, PlayerPos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ViewablePlayerState {
    SelfState(ViewableSelfState),
    Opponent(ViewableOpponentState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ViewableOpponentState {
    pub hand_count: usize,
    pub deck_count: usize,
    pub enhancements: Vec<Card>,
    pub played_pile: Vec<Card>,
    pub discard_pile_count: usize,

    pub vigor: i32,
    pub aura: i32,
    pub life: i32,
    pub flare: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ViewableSelfState {
    pub hands: Vec<Card>,
    pub deck_count: usize,
    pub enhancements: Vec<Card>,
    pub played_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,

    pub vigor: i32,
    pub aura: i32,
    pub life: i32,
    pub flare: i32,
}
// Todo: Divide Self-Opponent state correctly so that no properties are repeated.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ViewableState {
    pub turn_number: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
    pub distance: i32,
    pub dust: i32,
    pub player_states: ViewablePlayerStates,
}

pub type ViewablePlayerStates = PlayerData<ViewablePlayerState>;
