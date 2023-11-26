use std::ops::{ControlFlow, FromResidual, Try};
use crate::game::{GameError, GameResult};

pub enum GameControlFlow {
    Continue,
    BreakPhase(PhaseBreak)
}

pub enum PhaseBreak {
    EndPhase,
    EndTurn,
    EndGame(GameResult),
}

impl FromResidual<GameControlFlow> for Result<GameControlFlow, GameError> {
    fn from_residual(residual: GameControlFlow) -> Self {
        Ok(residual)
    }
}

impl FromResidual for GameControlFlow {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        residual
    }
}

impl Try for GameControlFlow {
    type Output = ();
    type Residual = Self;

    fn from_output(_: Self::Output) -> Self {
        Self::Continue
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            GameControlFlow::Continue => ControlFlow::Continue(()),
            GameControlFlow::BreakPhase(b) => ControlFlow::Break( GameControlFlow::BreakPhase(b)),
        }
    }
}
