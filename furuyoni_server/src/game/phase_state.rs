use furuyoni_lib::rules::events::UpdatePhaseState;
use furuyoni_lib::rules::{Phase, PlayerPos};
use std::ops::Deref;

pub(crate) struct PhaseStateInner {
    pub turn: u32,
    pub turn_player: PlayerPos,
    pub phase: Phase,
}

pub(crate) struct PhaseState {
    inner: PhaseStateInner,
}
impl PhaseState {
    pub fn new(turn_number: u32, turn_player: PlayerPos, phase: Phase) -> Self {
        Self {
            inner: PhaseStateInner {
                turn: turn_number,
                turn_player,
                phase,
            },
        }
    }

    pub fn apply_update(&mut self, update: UpdatePhaseState) {
        match update {
            UpdatePhaseState::SetTurn { turn, turn_player } => {
                self.inner.turn = turn;
                self.inner.turn_player = turn_player;
            }
            UpdatePhaseState::SetPhase(phase) => {
                self.inner.phase = phase;
            }
        }
    }
}

impl Deref for PhaseState {
    type Target = PhaseStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
