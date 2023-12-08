use crate::game::{GameError, GameResult};
use std::ops::{ControlFlow, FromResidual, Try};

pub enum GameControlFlow {
    Continue,
    BreakPhase(PhaseBreak),
}

pub enum PhaseBreak {
    EndPhase,
    EndTurn,
    EndGame(GameResult),
}

impl<TOk, TErr> FromResidual<PhaseBreak> for Result<TOk, TErr>
where
    TOk: FromResidual<PhaseBreak>,
{
    fn from_residual(residual: PhaseBreak) -> Self {
        Ok(TOk::from_residual(residual))
    }
}

impl FromResidual for GameControlFlow {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        Self::BreakPhase(residual)
    }
}

impl Try for GameControlFlow {
    type Output = ();
    type Residual = PhaseBreak;

    fn from_output(_: Self::Output) -> Self {
        Self::Continue
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            GameControlFlow::Continue => ControlFlow::Continue(()),
            GameControlFlow::BreakPhase(b) => ControlFlow::Break(b),
        }
    }
}
