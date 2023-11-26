mod petals;
mod player_state;
mod game_controlflow;

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
use player_state::PlayerState;
use thiserror::Error;
use crate::game::game_controlflow::{GameControlFlow, PhaseBreak};
use crate::game::game_controlflow::GameControlFlow::{BreakPhase, Continue};

const GET_ACTION_RETRY_TIMES: usize = 3;


#[derive(Error, Debug)]
pub enum GameError {
    #[error("Failed to communicate with a player.")]
    PlayerCommunicationFail(PlayerPos),
    #[error("An invalid action has been requested from the player.")]
    InvalidActionRequested(PlayerPos),
}

pub enum GameResult {
    Draw,
    Winner(PlayerPos),
}

type Players = PlayerData<Box<dyn Player + Send + Sync>>;

#[derive(Debug, Copy, Clone, PartialEq, Neg)]
pub struct Vigor(i32);

struct PhaseState {
    turn_number: u32,
    turn_player: PlayerPos,
    phase: Phase,
}

struct BoardState {
    distance: Petals,
    dust: Petals,
    player_states: PlayerStates,
}

type PlayerStates = PlayerData<PlayerState>;


pub async fn run_game(player_1: Box<dyn Player + Sync + Send>,
                      player_2: Box<dyn Player + Sync + Send>) -> Result<GameResult, GameError> {
    let mut players = Players::new(player_1, player_2);

    let (mut phase_state, mut board_state) = initialize_game_states();

    notify_game_start(&mut players, &phase_state, &board_state).await?;


    // Define phase modifying functions.
    fn next_phase(phase_state: &mut PhaseState) {
        match phase_state.phase {
            Phase::Beginning => { phase_state.phase = Phase::Main }
            Phase::Main => { phase_state.phase = Phase::End }
            Phase::End => { next_turn(phase_state) }
        }
    }

    fn next_turn(phase_state: &mut PhaseState) {
        *phase_state = PhaseState {
            turn_number: phase_state.turn_number + 1,
            turn_player: phase_state.turn_player.other(),
            phase: Phase::Beginning,
        }
    }

    // phase loop
    loop {
        let phase_result: GameControlFlow = match phase_state.phase {
            Phase::Beginning => { run_beginning_phase(&mut players, &phase_state, &mut board_state).await? }
            Phase::Main => { run_main_phase(&mut players, &phase_state, &mut board_state).await? }
            Phase::End => { run_end_phase(&mut players, &phase_state, &mut board_state).await? }
        };

        match phase_result {
            Continue => { next_phase(&mut phase_state) }
            BreakPhase(phase_break) => {
                match phase_break {
                    PhaseBreak::EndPhase => { next_phase(&mut phase_state) }
                    PhaseBreak::EndTurn => { next_turn(&mut phase_state) }
                    PhaseBreak::EndGame(game_result) => { return Ok(game_result); }
                }
            }
        }
    }
}


async fn run_beginning_phase(players: &mut Players, phase_state: &PhaseState, board_state: &mut BoardState)
                             -> Result<GameControlFlow, GameError> {
    // Skip beginning phase for the first two turns.
    if phase_state.turn_number <= 2 {
        return Ok(Continue);
    }

    // Add vigor
    add_to_vigor(&mut board_state.player_states[phase_state.turn_player], Vigor(1)).unwrap();
    // Todo: remove sakura tokens from enhancements, reshuffle deck, draw cards.

    Ok(Continue)
}

async fn run_main_phase(players: &mut Players, phase_state: &PhaseState, board_state: &mut BoardState)
                        -> Result<GameControlFlow, GameError> {
    handle_player_actions(players, phase_state, board_state).await??;

    Ok(Continue)
}

async fn run_end_phase(players: &mut Players, phase_state: &PhaseState, board_state: &mut BoardState)
                       -> Result<GameControlFlow, GameError> {
    // Todo: move enhancements and in-use cards to the used pile.

    Ok(Continue)
}

async fn handle_player_actions(players: &mut Players, phase_state: &PhaseState, board_state: &mut BoardState)
                               -> Result<GameControlFlow, GameError> {
    let turn_player_pos = phase_state.turn_player;
    let turn_player = &mut players[turn_player_pos];

    // main phase actions loop.
    loop {
        // Todo: implement selecting playable actions
        let doable_basic_actions = vec![
            BasicAction::MoveForward,
            BasicAction::MoveBackward,
            BasicAction::Focus,
            BasicAction::Recover,
        ];
        let playable_cards = vec![PlayableCardSelector::Hand(HandSelector(0))];
        let available_costs = vec![BasicActionCost::Vigor];

        // Todo: some reusable retry function.
        let mut cnt_try = 0;
        let action = loop {
            let action = turn_player
                .get_main_phase_action(
                    &get_player_viewable_state(&phase_state, &board_state, turn_player_pos),
                    &playable_cards,
                    &doable_basic_actions,
                    &available_costs,
                )
                .await;
            // Todo: handle result.

            if validate_main_phase_action(board_state, &action) {
                break action;
            }
            cnt_try += 1;
            if cnt_try >= GET_ACTION_RETRY_TIMES {
                return Err(GameError::InvalidActionRequested(turn_player_pos));
            }
        };

        match action {
            MainPhaseAction::EndMainPhase => return Ok(Continue),
            MainPhaseAction::PlayBasicAction(PlayBasicAction { action, cost }) => {
                pay_basic_action_cost(board_state, turn_player_pos, cost)?;
                play_basic_action(board_state, turn_player_pos, action)?;
                continue;
            }
            MainPhaseAction::PlayCard(_) => {
                todo!();
                continue;
            }
        };

        fn validate_main_phase_action(_state: &BoardState, _action: &MainPhaseAction) -> bool {
            true // Todo:
        }
    }
}

fn initialize_game_states() -> (PhaseState, BoardState) {
    // Select starting player.
    let start_player = if rand::random::<bool>() {
        PlayerPos::P1
    } else {
        PlayerPos::P2
    };

    // Initialize states.
    let phase_state = PhaseState {
        turn_number: 1,
        turn_player: start_player,
        phase: Phase::Beginning,
    };

    let board_state = BoardState {
        distance: Petals::new(10),
        dust: Petals::new(0),
        player_states: default_player_states(),
    };

    (phase_state, board_state)
}

async fn notify_game_start(players: &mut Players, phase_state: &PhaseState, board_state: &BoardState) -> Result<(), GameError> {
    async fn notify_start(
        p: &mut (impl Player + ?Sized + Send),
        phase_state: &PhaseState,
        board_state: &BoardState,
        pos: PlayerPos,
    ) -> Result<(), GameError> {
        p.notify_game_start(&get_player_viewable_state(phase_state, board_state, pos), pos)
            .await
            .map_err(|e| GameError::PlayerCommunicationFail(pos))
    }

    let (a, b) = join!(
        notify_start(&mut *players.p1_data, phase_state, board_state, PlayerPos::P1),
        notify_start(&mut *players.p2_data, phase_state, board_state, PlayerPos::P2)
    );

    (a?, b?);

    Ok(())
}


/// Return default player states. Only used for debugging.
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


fn play_basic_action(
    board_state: &mut BoardState,
    player: PlayerPos,
    action: BasicAction,
) -> Result<GameControlFlow, GameError> {
    let player_data = &mut board_state.player_states[player];

    // Todo: lambda로 map_err 한 줄로 줄이기?
    match action {
        BasicAction::MoveForward => {
            // Todo: Petals max 추가, transfer.
            player_data.aura += board_state.distance.take(1).map_err(|()| GameError::InvalidActionRequested(player))?;
        }
        BasicAction::MoveBackward => {
            board_state.distance += player_data.aura.take(1).map_err(|()| GameError::InvalidActionRequested(player))?;
        }
        BasicAction::Recover => {
            player_data.aura += board_state.dust.take(1).map_err(|()| GameError::InvalidActionRequested(player))?;
        }
        BasicAction::Focus => {
            player_data.flare += player_data.aura.take(1).map_err(|()| GameError::InvalidActionRequested(player))?;
        }
    }

    Ok(Continue)
}


fn pay_basic_action_cost(board_state: &mut BoardState, player: PlayerPos, cost: BasicActionCost)
                         -> Result<(), GameError> {
    let player_state = &mut board_state.player_states[player];
    match cost {
        BasicActionCost::Hand(selector) => {
            let hand = &mut player_state.hand;

            if selector.0 > hand.len() {
                return Err(GameError::InvalidActionRequested(player));
            }
            let card = hand.remove(selector.0);

            player_state.discard_pile.push(card)
        }
        BasicActionCost::Vigor => add_to_vigor(player_state, -Vigor(1)).map_err(|()| GameError::InvalidActionRequested(player))?,
    }

    Ok(())
}

fn get_player_viewable_state(phase_state: &PhaseState, board: &BoardState, viewed_from: PlayerPos) -> ViewableState {
    let player_states = &board.player_states;

    let get_player_state = |player: PlayerPos| -> ViewablePlayerState {
        let player_state = &player_states[player];
        if player == viewed_from {
            ViewablePlayerState::SelfState(ViewableSelfState::from(player_state))
        } else {
            ViewablePlayerState::Opponent(ViewableOpponentState::from(player_state))
        }
    };

    ViewableState {
        turn_player: phase_state.turn_player,
        phase: phase_state.phase,
        turn_number: phase_state.turn_number,
        distance: board.distance.get_count(),
        dust: board.dust.get_count(),
        player_states: ViewablePlayerStates::new(
            get_player_state(PlayerPos::P1),
            get_player_state(PlayerPos::P2),
        ),
    }
}


// Todo: impl Add for Vigor
fn add_to_vigor(player_state: &mut PlayerState, diff: Vigor) -> Result<(), ()> {
    const MAX_VIGOR: i32 = 2;
    const MIN_VIGOR: i32 = 0;

    let vigor = &mut player_state.vigor;

    let new = vigor.0 + diff.0;

    if new < 0 {
        return Err(());
    }

    vigor.0 = cmp::min(MAX_VIGOR, new);
    Ok(())
}
