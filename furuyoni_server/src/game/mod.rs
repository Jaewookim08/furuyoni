mod petals;
use petals::Petals;

use async_recursion::async_recursion;
use derive_more::Neg;
use furuyoni_lib::cards::Card;
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, HandSelector, MainPhaseAction, PlayBasicAction,
    PlayableCardSelector,
};
use furuyoni_lib::players::{Player, PlayerData};
use furuyoni_lib::rules::{
    Phase, PlayerPos, ViewableOpponentState, ViewablePlayerState, ViewablePlayerStates,
    ViewableSelfState, ViewableState,
};

use futures::future::BoxFuture;
use std::cmp;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::{Send, Sync};
use tokio::join;

type Players = PlayerData<Box<dyn Player + Send + Sync>>;

macro_rules! unwrap_or {
    ( $e:expr, $el:expr ) => {
        match $e {
            Some(x) => x,
            None => $el,
        }
    };
}

pub struct Game {
    players: Players,
    state: GameState,
}

pub struct GameResult {
    pub winner: PlayerPos,
}

#[derive(Debug, Copy, Clone, PartialEq, Neg)]
pub struct Vigor(i32);

struct GameState {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: Phase,
    distance: Petals,
    dust: Petals,
    player_states: PlayerStates,
}

impl From<&PlayerState> for ViewableOpponentState {
    fn from(player_state: &PlayerState) -> Self {
        ViewableOpponentState {
            hand_count: player_state.hand.len(),
            deck_count: player_state.deck.len(),
            enhancements: player_state.enhancements.clone(),
            played_pile: player_state.played_pile.clone(),
            discard_pile_count: player_state.discard_pile.len(),

            vigor: player_state.vigor.0,
            aura: player_state.aura.get_count(),
            life: player_state.life.get_count(),
            flare: player_state.flare.get_count(),
        }
    }
}

impl From<&PlayerState> for ViewableSelfState {
    fn from(player_state: &PlayerState) -> Self {
        ViewableSelfState {
            hands: player_state.hand.clone(),
            deck_count: player_state.deck.len(),
            enhancements: player_state.enhancements.clone(),
            played_pile: player_state.played_pile.clone(),
            discard_pile: player_state.discard_pile.clone(),

            vigor: player_state.vigor.0,
            aura: player_state.aura.get_count(),
            life: player_state.life.get_count(),
            flare: player_state.flare.get_count(),
        }
    }
}

type PlayerStates = PlayerData<PlayerState>;

trait Continuation<'a, TArgs>: FnOnce(TArgs) -> StepResult<'a> + Send + 'a {}

impl<'a, TArgs, T> Continuation<'a, TArgs> for T where T: FnOnce(TArgs) -> StepResult<'a> + Send + 'a
{}

#[derive(Debug)]
pub struct PlayerState {
    hand: Vec<Card>,
    deck: VecDeque<Card>,
    enhancements: Vec<Card>,
    played_pile: Vec<Card>,
    discard_pile: Vec<Card>,

    vigor: Vigor,
    aura: Petals,
    life: Petals,
    flare: Petals,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            hand: vec![],
            deck: VecDeque::default(),
            enhancements: vec![],
            played_pile: vec![],
            discard_pile: vec![],
            vigor: Vigor(0),
            aura: Petals::new(3),
            life: Petals::new(10),
            flare: Petals::new(0),
        }
    }
}

enum StepResult<'a> {
    TailCall(BoxFuture<'a, StepResult<'a>>),
    Result(GameResult),
}

fn rec_call<'a>(future: impl Future<Output = StepResult<'a>> + Send + 'a) -> StepResult<'a> {
    StepResult::TailCall(Box::pin(future))
}

fn rec_ret<'a>(result: GameResult) -> StepResult<'a> {
    StepResult::Result(result)
}

impl GameResult {
    pub fn new(winner: PlayerPos) -> Self {
        Self { winner }
    }
}

impl GameState {
    fn new(
        turn_number: u32,
        turn_player: PlayerPos,
        phase: Phase,
        distance: Petals,
        dust: Petals,
        player_states: PlayerStates,
    ) -> Self {
        GameState {
            turn_player,
            phase,
            distance,
            dust,
            turn_number,
            player_states,
        }
    }
}

impl Game {
    pub fn new(
        player_1: Box<dyn Player + Sync + Send>,
        player_2: Box<dyn Player + Sync + Send>,
    ) -> Self {
        let start_player = if rand::random::<bool>() {
            PlayerPos::P1
        } else {
            PlayerPos::P2
        };

        Game {
            players: Players::new(player_1, player_2),
            state: GameState::new(
                0,
                start_player,
                Phase::Main,
                Petals::new(10),
                Petals::new(0),
                default_player_states(),
            ),
        }
    }

    pub async fn run(&mut self) -> GameResult {
        run(self).await
    }
}

fn default_player_states() -> PlayerStates {
    let p1_state = PlayerState {
        deck: VecDeque::from([
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
        ]),
        ..Default::default()
    };

    let p2_state = PlayerState {
        deck: VecDeque::from([
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
            Card::Slash,
        ]),
        ..Default::default()
    };

    PlayerStates::new(p1_state, p2_state)
}

async fn run(game: &mut Game) -> GameResult {
    start_game(game).await.expect("todo");

    let mut next: BoxFuture<StepResult> = Box::pin(next_turn(game));

    let result = loop {
        let step_result = next.await;

        match step_result {
            StepResult::TailCall(future) => next = future,
            StepResult::Result(res) => {
                break res;
            }
        }
    };
    //

    result
}

async fn start_game(game: &mut Game) -> Result<(), ()> {
    async fn notify_start(
        p: &mut (impl Player + ?Sized + Send),
        state: &GameState,
        pos: PlayerPos,
    ) -> Result<(), ()> {
        p.start_game(&get_player_viewable_state(state, pos), pos)
            .await
    }

    let (a, b) = join!(
        notify_start(&mut *game.players.p1_data, &game.state, PlayerPos::P1),
        notify_start(&mut *game.players.p2_data, &game.state, PlayerPos::P2)
    );

    (a?, b?);

    Ok(())
}

async fn next_turn(game: &mut Game) -> StepResult {
    // increase turn number
    let state = &mut game.state;
    state.turn_number += 1;

    // switch current player
    let next_player = if state.turn_player == PlayerPos::P1 {
        PlayerPos::P2
    } else {
        PlayerPos::P1
    };
    state.turn_player = next_player;

    let step_result = if state.turn_number <= 2 {
        rec_call(run_from_main_phase(game))
    } else {
        rec_call(run_from_beginning_phase(game))
    };

    step_result
}

async fn run_from_beginning_phase(game: &mut Game) -> StepResult {
    let state = &mut game.state;
    state.phase = Phase::Beginning;

    let current_player = state.turn_player;
    add_to_vigor(state, current_player, Vigor(1));
    // Todo: remove sakura tokens from enhancements, reshuffle deck, draw cards.

    rec_call(run_from_main_phase(game))
}

async fn run_from_main_phase(game: &mut Game) -> StepResult {
    game.state.phase = Phase::Main;

    rec_call(do_main_phase_actions(game, |game| {
        rec_call(run_from_end_phase(game))
    }))
}

#[async_recursion]
async fn do_main_phase_actions<'a>(
    game: &'a mut Game,
    cont: impl Continuation<'a, &'a mut Game>,
) -> StepResult<'a> {
    const GET_ACTION_RETRY_TIMES: usize = 3;
    let state = &mut game.state;

    let turn_player = state.turn_player;
    let turn_player_data = &mut game.players[turn_player];

    let doable_basic_actions = vec![
        BasicAction::MoveForward,
        BasicAction::MoveBackward,
        BasicAction::Focus,
        BasicAction::Recover,
    ];
    let playable_cards = vec![PlayableCardSelector::Hand(HandSelector(0))];
    let available_costs = vec![BasicActionCost::Vigor];

    let mut cnt = 0;
    let action = loop {
        let action = turn_player_data
            .get_main_phase_action(
                &get_player_viewable_state(&state, turn_player),
                &playable_cards,
                &doable_basic_actions,
                &available_costs,
            )
            .await;

        if validate_main_phase_action(state, &action) {
            break action;
        }
        cnt += 1;
        if cnt >= GET_ACTION_RETRY_TIMES {
            todo!()
        }
    };

    let ret = match action {
        MainPhaseAction::EndMainPhase => cont(game),
        MainPhaseAction::PlayBasicAction(PlayBasicAction { action, cost }) => rec_call(
            pay_basic_action_cost(game, turn_player, cost, move |game| {
                rec_call(play_basic_action(game, turn_player, action, |game| {
                    rec_call(do_main_phase_actions(game, cont))
                }))
            }),
        ),
        MainPhaseAction::PlayCard(_) => cont(game),
    };

    fn validate_main_phase_action(_state: &GameState, _action: &MainPhaseAction) -> bool {
        true // Todo:
    }

    ret
}

#[async_recursion]
async fn run_from_end_phase(game: &mut Game) -> StepResult {
    rec_call(turn_end(game))
}

#[async_recursion]
async fn play_basic_action<'a>(
    game: &'a mut Game,
    player: PlayerPos,
    action: BasicAction,
    cont: impl Continuation<'a, &'a mut Game>,
) -> StepResult<'a> {
    let state = &mut game.state;
    let player_data = &mut state.player_states[player];

    match action {
        BasicAction::MoveForward => {
            player_data.aura += state.distance.take(1);
        }
        BasicAction::MoveBackward => {
            state.distance += player_data.aura.take(1);
        }
        BasicAction::Recover => {
            player_data.aura += state.dust.take(1);
        }
        BasicAction::Focus => {
            player_data.flare += player_data.aura.take(1);
        }
    }
    cont(game)
}

#[async_recursion]
async fn pay_basic_action_cost<'a>(
    game: &'a mut Game,
    player: PlayerPos,
    cost: BasicActionCost,
    cont: impl Continuation<'a, &'a mut Game>,
) -> StepResult<'a> {
    match cost {
        BasicActionCost::Hand(selector) => {
            let player_state = &mut game.state.player_states[player];
            let hand = &mut player_state.hand;

            if selector.0 > hand.len() {
                todo!("Call error continuation.")
            }
            let card = hand.remove(selector.0);

            player_state.discard_pile.push(card)
        }
        BasicActionCost::Vigor => add_to_vigor(&mut game.state, player, -Vigor(1)),
    }
    cont(game)
}

async fn turn_end<'a>(game: &'a mut Game) -> StepResult {
    // Todo: move enhancements and in-use cards to the used pile.
    rec_call(next_turn(game))
}

fn get_player_viewable_state(state: &GameState, viewed_from: PlayerPos) -> ViewableState {
    let player_states = &state.player_states;

    let get_player_state = |player: PlayerPos| -> ViewablePlayerState {
        let player_state = &player_states[player];
        if player == viewed_from {
            ViewablePlayerState::SelfState(ViewableSelfState::from(player_state))
        } else {
            ViewablePlayerState::Opponent(ViewableOpponentState::from(player_state))
        }
    };

    ViewableState {
        turn_player: state.turn_player,
        phase: state.phase,
        turn_number: state.turn_number,
        distance: state.distance.get_count(),
        dust: state.dust.get_count(),
        player_states: ViewablePlayerStates::new(
            get_player_state(PlayerPos::P1),
            get_player_state(PlayerPos::P2),
        ),
    }
}

async fn test_win<'a>() -> StepResult<'a> {
    rec_ret(GameResult::new(PlayerPos::P1))
}

fn add_to_vigor(state: &mut GameState, player: PlayerPos, diff: Vigor) {
    const MAX_VIGOR: i32 = 2;
    const MIN_VIGOR: i32 = 0;

    let vigor = &mut state.player_states[player].vigor;

    vigor.0 = cmp::min(MAX_VIGOR, cmp::max(MIN_VIGOR, vigor.0 + diff.0));
}
