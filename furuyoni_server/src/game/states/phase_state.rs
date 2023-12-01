use furuyoni_lib::rules::{Phase, PlayerPos};

pub struct PhaseState {
    pub turn: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
}
impl PhaseState {
    pub fn new(turn_number: u32, turn_player: PlayerPos, phase: Phase) -> Self {
        Self {
            turn: turn_number,
            turn_player,
            phase,
        }
    }
}
