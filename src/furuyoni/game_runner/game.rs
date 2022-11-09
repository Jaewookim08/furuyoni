use std::future::Future;
use futures::future::BoxFuture;
use super::cards::*;

pub struct Game {
    state: GameState,
}

pub struct GameResult {
    pub winner: PlayerPos,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PlayerPos {
    P1,
    P2,
}

struct GameState {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: Phase,
}

struct PlayerState {
    hand: Vec<Cards>,
}


enum StepResult<'a> {
    TailCall(BoxFuture<'a, StepResult<'a>>),
    Result(GameResult),
}

fn rec_call<'a>(future: impl Future<Output=StepResult<'a>> + Send + 'a) -> StepResult<'a> {
    StepResult::TailCall(Box::pin(future))
}

fn rec_ret<'a>(result: GameResult) -> StepResult<'a> {
    StepResult::Result(result)
}

enum Phase {
    Beginning,
    Main,
    End,
}


impl GameResult {
    pub fn new(winner: PlayerPos) -> Self {
        Self { winner }
    }
}


type Continuation = fn() -> GameResult;

impl GameState {
    fn new(turn_number: u32, turn_player: PlayerPos, phase: Phase) -> Self {
        GameState {
            turn_player,
            phase,
            turn_number,
        }
    }
}


impl Game {
    pub fn new() -> Self {
        Game { state: GameState::new(0, PlayerPos::P2, Phase::Main) }
    }

    pub fn hello(&self) {
        print!("Hello!!")
    }

    pub async fn run(&mut self) -> GameResult {
        let mut next: BoxFuture<StepResult> = Box::pin(self.next_turn());

        let result = loop {
            let step_result = next.await;

            match step_result {
                StepResult::TailCall(future) => { next = future }
                StepResult::Result(res) => { break res; }
            }
        };

        result
    }
}




impl Game {
    async fn next_turn(&mut self) -> StepResult {
        // increase turn number
        self.state.turn_number += 1;

        // switch current player
        let next_player = if self.state.turn_player == PlayerPos::P1 { PlayerPos::P2 } else { PlayerPos::P1 };
        self.state.turn_player = next_player;

        const UNREACHABLE_CONT: Continuation = || panic!("This continuation should never be executed.\
             This indicates that the game has ended without a winner");

        let step_result = if self.state.turn_number <= 2 {
            rec_call(self.run_from_main_phase(UNREACHABLE_CONT))
        } else {
            rec_call(self.run_from_beginning_phase(UNREACHABLE_CONT))
        };

        step_result
    }

    async fn run_from_beginning_phase(&mut self, cont: Continuation) -> StepResult {
        self.test_win(cont)
    }

    async fn run_from_main_phase(&mut self, cont: Continuation) -> StepResult {
        self.test_win(cont)
    }

    fn test_win(&mut self, _cont: Continuation) -> StepResult {
        // ignore cont
        rec_ret(GameResult::new(PlayerPos::P1))
    }
}

