mod cards;
mod attack;
mod effects;
mod condition;


use std::cmp;
use std::collections::VecDeque;
use std::future::Future;
use std::ops::{Index, IndexMut};
use enum_dispatch::enum_dispatch;
use futures::future::BoxFuture;
use cards::Card;
use crate::furuyoni::Player;
use std::marker::{Send, Sync};

#[enum_dispatch(Player)]
enum GamePlayer<T1: Player, T2: Player> {
    Player1(T1),
    Player2(T2),
}

type Players<TPlayer1, TPlayer2> = PlayerData<GamePlayer<TPlayer1, TPlayer2>>;

pub struct Game<TPlayer1: Player + Send, TPlayer2: Player + Send> {
    state: GameState,
    players: Players<TPlayer1, TPlayer2>,
}

pub struct GameResult {
    pub winner: PlayerPos,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PlayerPos {
    P1,
    P2,
}

pub enum BasicAction {
    MoveForward,
    MoveBackward,
    Recover,
    Focus,
}

pub enum MainPhaseAction {
    BasicAction(BasicAction),
    PlayCard(&'static Card),
    EndMainPhase,
}


struct GameState {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: Phase,
    player_states: PlayerStates,
}

type ViewablePlayerStates = PlayerData<ViewablePlayerState>;

enum ViewablePlayerState {
    Transparent(PlayerState)
}

pub struct ViewableState {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: Phase,
    player_states: ViewablePlayerStates,
}

struct PlayerData<TData> {
    p1_data: TData,
    p2_data: TData,
}

type PlayerStates = PlayerData<PlayerState>;

impl<T> PlayerData<T> {
    fn new(p1_data: T, p2_data: T) -> Self {
        Self {
            p1_data,
            p2_data,
        }
    }
}

impl<T> Index<PlayerPos> for PlayerData<T> {
    type Output = T;

    fn index(&self, index: PlayerPos) -> &Self::Output {
        match index {
            PlayerPos::P1 => { &self.p1_data }
            PlayerPos::P2 => { &self.p2_data }
        }
    }
}

impl<T> IndexMut<PlayerPos> for PlayerData<T> {
    fn index_mut(&mut self, index: PlayerPos) -> &mut Self::Output {
        match index {
            PlayerPos::P1 => { &mut self.p1_data }
            PlayerPos::P2 => { &mut self.p2_data }
        }
    }
}


pub struct PlayerState {
    hand: Vec<Card>,
    deck: VecDeque<Card>,
    enhancements: Vec<Card>,
    played_pile: Vec<Card>,
    discard_pile: Vec<Card>,

    vigor: i32,
    aura: i32,
    life: i32,
    flare: i32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            hand: vec![],
            deck: VecDeque::default(),
            enhancements: vec![],
            played_pile: vec![],
            discard_pile: vec![],
            vigor: 0,
            aura: 3,
            life: 10,
            flare: 0,
        }
    }
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

pub enum Phase {
    Beginning,
    Main,
    End,
}


impl GameResult {
    pub fn new(winner: PlayerPos) -> Self {
        Self { winner }
    }
}


struct Continuation<'a>(StepResult<'a>);

impl GameState {
    fn new(turn_number: u32, turn_player: PlayerPos, phase: Phase, player_states: PlayerStates) -> Self {
        GameState {
            turn_player,
            phase,
            turn_number,
            player_states,
        }
    }
}


impl<TPlayer1: Player + Send, TPlayer2: Player + Send> Game<TPlayer1, TPlayer2> {
    pub fn new(player_1: TPlayer1, player_2: TPlayer2) -> Self {
        let p1_state = PlayerState {
            deck: VecDeque::from([Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash]),
            ..Default::default()
        };

        let p2_state = PlayerState {
            deck: VecDeque::from([Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash]),
            ..Default::default()
        };

        Game {
            state: GameState::new(0, PlayerPos::P2, Phase::Main,
                                  PlayerStates::new(p1_state, p2_state)),
            players: Players::new(GamePlayer::Player1(player_1), GamePlayer::Player2(player_2)),
        }
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
    async fn next_turn(&mut self) -> StepResult {
        // increase turn number
        self.state.turn_number += 1;

        // switch current player
        let next_player = if self.state.turn_player == PlayerPos::P1 { PlayerPos::P2 } else { PlayerPos::P1 };
        self.state.turn_player = next_player;

        let unreachable_cont: Continuation = Continuation(StepResult::TailCall(Box::pin(async {
            panic!("This continuation should never be executed. \
             This indicates that the game has ended without a winner");
        })));

        let step_result = if self.state.turn_number <= 2 {
            rec_call(self.run_from_main_phase(unreachable_cont))
        } else {
            rec_call(self.run_from_beginning_phase(unreachable_cont))
        };

        step_result
    }

    async fn run_from_beginning_phase<'a>(&mut self, cont: Continuation<'a>) -> StepResult<'a> {
        self.state.phase = Phase::Beginning;

        let current_player = self.state.turn_player;
        self.add_to_vigor(current_player, 1);
        // Todo: remove sakura tokens from enhancements, reshuffle deck, draw cards.

        self.test_win(cont)
    }

    async fn run_from_main_phase<'a>(&mut self, cont: Continuation<'a>) -> StepResult<'a> {
        self.state.phase = Phase::Main;


        self.test_win(cont)
    }

    fn test_win<'a>(&mut self, _cont: Continuation<'a>) -> StepResult<'a> {
        // ignore cont
        rec_ret(GameResult::new(PlayerPos::P1))
    }

    fn add_to_vigor(&mut self, player: PlayerPos, diff: i32) {
        const MAX_VIGOR: i32 = 2;
        const MIN_VIGOR: i32 = 0;

        let vigor = &mut self.state.player_states[player].vigor;

        *vigor = cmp::min(MAX_VIGOR, cmp::max(MIN_VIGOR, *vigor + diff));
    }
}




