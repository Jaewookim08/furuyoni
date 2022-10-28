use crate::furuyoni::game_runner::game::Phase::Beginning;

pub struct Game {
    state: GameState,
}

struct GameState {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: Phase,
}

struct GameResult {
    winner: PlayerPos,
}

struct PlayerState {
    hand: Vec<Card1>,
}

#[derive(Eq, PartialEq)]
enum PlayerPos {
    P1,
    P2,
}

enum Phase {
    Beginning,
    Main,
    End,
}

struct Card1 {}

struct Card2 {}

enum Cards {
    Card1(Card1),
    Card2(Card2),
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
        self.next_turn().await
    }
}


impl Game {
    async fn next_turn(&mut self) -> GameResult {
        self.state.turn_number += 1;
        let next_player = if self.state.turn_player == PlayerPos::P1 { PlayerPos::P2 } else { PlayerPos::P1 };
        self.state.turn_player = next_player;
        const UNREACHABLE_CONT: Continuation = || panic!("This continuation should never be executed.\
             This indicates that the game has ended without a winner");

        if self.state.turn_number <= 2 {
            self.run_from_main_phase(UNREACHABLE_CONT).await
        } else {
            self.run_from_beginning_phase(UNREACHABLE_CONT).await
        }
    }

    async fn run_from_beginning_phase(&mut self, cont: Continuation) -> GameResult {
        self.test_win(cont)
    }

    async fn run_from_main_phase(&mut self, cont: Continuation) -> GameResult {
        self.test_win(cont)
    }

    fn test_win(&mut self, _cont: Continuation) -> GameResult {
        // ignore cont
        GameResult::new(PlayerPos::P1)
    }
}

