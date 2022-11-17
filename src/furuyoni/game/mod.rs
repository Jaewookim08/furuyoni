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
use std::marker::{Send, Sync};
use crate::furuyoni;
use crate::furuyoni::Player;


type Players = PlayerData<Box<dyn Player + Send + Sync>>;

pub struct Game {
    players: Players,
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

type ViewablePlayerStates<'a> = PlayerData<ViewablePlayerState<'a>>;

struct ViewableEnemyState<'a> {
    hand_count: usize,
    deck_count: usize,
    enhancements: &'a Vec<Card>,
    played_pile: &'a Vec<Card>,
    discard_pile_count: usize,

    vigor: i32,
    aura: i32,
    life: i32,
    flare: i32,
}

impl<'a> From<&'a PlayerState> for ViewableEnemyState<'a> {
    fn from(player_state: &'a PlayerState) -> Self {
        ViewableEnemyState {
            hand_count: player_state.hand.len(),
            deck_count: player_state.deck.len(),
            enhancements: &player_state.enhancements,
            played_pile: &player_state.played_pile,
            discard_pile_count: player_state.discard_pile.len(),

            vigor: player_state.vigor,
            aura: player_state.aura,
            life: player_state.life,
            flare: player_state.flare,
        }
    }
}

enum ViewablePlayerState<'a> {
    Transparent(&'a PlayerState),
    Enemy(ViewableEnemyState<'a>),
}

pub struct ViewableState<'a> {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: &'a Phase,
    player_states: ViewablePlayerStates<'a>,
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
    StepResult::TailCall(
        Box::pin(future)
    )
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


impl Game {
    pub fn new(player_1: Box<dyn Player + Sync + Send>, player_2: Box<dyn Player + Sync + Send>) -> Self {
        Game {
            players: Players::new(player_1, player_2),
        }
    }

    fn default_player_states() -> PlayerStates {
        let p1_state = PlayerState {
            deck: VecDeque::from([Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash]),
            ..Default::default()
        };

        let p2_state = PlayerState {
            deck: VecDeque::from([Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash, Card::Slash]),
            ..Default::default()
        };

        PlayerStates::new(p1_state, p2_state)
    }

    pub async fn run(&self) -> GameResult {
        let mut state = GameState::new(0, PlayerPos::P2, Phase::Main,
                                       Self::default_player_states());

        let mut next: BoxFuture<StepResult> = Box::pin(self.next_turn(&mut state));

        let result = loop {
            let step_result = next.await;

            match step_result {
                StepResult::TailCall(future) => { next = future }
                StepResult::Result(res) => { break res; }
            }
        };

        result
    }

    async fn next_turn<'a>(&'a self, state: &'a mut GameState) -> StepResult {
        // increase turn number
        state.turn_number += 1;

        // switch current player
        let next_player = if state.turn_player == PlayerPos::P1 { PlayerPos::P2 } else { PlayerPos::P1 };
        state.turn_player = next_player;

        let step_result = if state.turn_number <= 2 {
            rec_call(self.run_from_main_phase(state))
        } else {
            rec_call(self.run_from_beginning_phase(state))
        };

        step_result
    }

    async fn run_from_beginning_phase<'a>(&'a self, state: &'a mut GameState) -> StepResult {
        state.phase = Phase::Beginning;

        let current_player = state.turn_player;
        Self::add_to_vigor(state, current_player, 1);
        // Todo: remove sakura tokens from enhancements, reshuffle deck, draw cards.

        rec_call(self.run_from_main_phase(state))
    }

    async fn run_from_main_phase<'a>(&'a self, state: &'a mut GameState) -> StepResult {
        state.phase = Phase::Main;


        rec_call(self.test_win())
    }

    async fn do_main_phase_actions<'a>(&'a self, state: &'a mut GameState, cont: Continuation<'a>) -> StepResult<'a> {
        let turn_player = state.turn_player;
        let turn_player_data = &self.players[turn_player];


        let action = turn_player_data.get_main_phase_action(
            &Self::get_player_viewable_state(&state, turn_player),
            &vec![MainPhaseAction::EndMainPhase],
        );

        todo!()
        // match action {
        //     EndMainPhase => {
        //         cont.0
        //     }
        //     _ => {
        //         rec_call(self.do_main_phase_actions(state, cont))
        //     }
        // }
    }

    async fn run_from_end_phase(&self, state: &mut GameState) -> StepResult {
        todo!()
    }

    async fn turn_end<'a>(&'a self, state: &'a mut GameState) -> StepResult {
        // Todo: move enhancements and in-use cards to the used pile.
        rec_call(self.next_turn(state))
    }

    fn get_player_viewable_state(state: &GameState, viewed_from: PlayerPos) -> ViewableState {
        let player_states = &state.player_states;

        let get_player_state = |player: PlayerPos| -> ViewablePlayerState {
            let player_state = &player_states[player];
            if player == viewed_from {
                ViewablePlayerState::Transparent(player_state)
            } else {
                ViewablePlayerState::Enemy(ViewableEnemyState::from(player_state))
            }
        };

        ViewableState {
            turn_player: state.turn_player,
            phase: &state.phase,
            turn_number: state.turn_number,
            player_states: ViewablePlayerStates::new(
                get_player_state(PlayerPos::P1),
                get_player_state(PlayerPos::P2),
            ),

        }
    }

    async fn test_win(&self) -> StepResult {
        StepResult::Result(GameResult::new(PlayerPos::P1))
    }

    fn add_to_vigor(state: &mut GameState, player: PlayerPos, diff: i32) {
        const MAX_VIGOR: i32 = 2;
        const MIN_VIGOR: i32 = 0;

        let vigor = &mut state.player_states[player].vigor;

        *vigor = cmp::min(MAX_VIGOR, cmp::max(MIN_VIGOR, *vigor + diff));
    }
}




