mod board_state;
mod game_controlflow;
mod petals;
mod phase_state;
mod player_state;

use petals::Petals;

use derive_more::Neg;
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, HandSelector, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::{PetalPosition, Phase, PlayerPos};

use crate::game::board_state::*;
use crate::game::game_controlflow::GameControlFlow::{BreakPhase, Continue};
use crate::game::game_controlflow::{GameControlFlow, PhaseBreak};
use crate::game::phase_state::*;
use furuyoni_lib::rules::cards::Card;
use furuyoni_lib::rules::events::{GameEvent, UpdateBoardState, UpdatePhaseState};
use furuyoni_lib::rules::states::*;
use player_state::PlayerState;
use std::cmp;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::{Send, Sync};
use thiserror::Error;
use tokio::join;

const GET_ACTION_RETRY_TIMES: usize = 3;

#[derive(Error, Debug)]
pub(crate) enum GameError {
    #[error("Failed to communicate with a player.")]
    PlayerCommunicationFail(PlayerPos),
    #[error("An invalid action has been requested from the player.")]
    InvalidActionRequested(PlayerPos),
    #[error("{0}")]
    InvalidBoardUpdate(#[from] InvalidBoardUpdateError),
}

pub enum GameResult {
    Draw,
    Winner(PlayerPos),
}

type Players = PlayersData<Box<dyn Player + Send + Sync>>;

#[derive(Debug, Copy, Clone, PartialEq, Neg)]
pub struct Vigor(i32);

type PlayerStates = PlayersData<PlayerState>;

pub(crate) async fn run_game(
    player_1: Box<dyn Player + Sync + Send>,
    player_2: Box<dyn Player + Sync + Send>,
) -> Result<GameResult, GameError> {
    let mut players = Players::new(player_1, player_2);

    let (mut phase_state, mut board_state) = initialize_game_states();

    notify_game_start(&mut players, &phase_state, &board_state).await?;

    // Define phase modifying functions.
    fn next_phase(phase_state: &mut PhaseState) -> Result<(), GameError> {
        match phase_state.phase {
            Phase::Beginning => {
                update_phase_and_notify(phase_state, UpdatePhaseState::SetPhase(Phase::Main));
            }
            Phase::Main => {
                update_phase_and_notify(phase_state, UpdatePhaseState::SetPhase(Phase::Main));
            }
            Phase::End => next_turn(phase_state)?,
        }
        Ok(())
    }

    fn next_turn(phase_state: &mut PhaseState) -> Result<(), GameError> {
        update_phase_and_notify(
            phase_state,
            UpdatePhaseState::SetTurn {
                turn_player: phase_state.turn_player.other(),
                turn: phase_state.turn + 1,
            },
        );

        update_phase_and_notify(phase_state, UpdatePhaseState::SetPhase(Phase::Beginning));
        Ok(())
    }

    // phase loop
    loop {
        let phase_result: GameControlFlow = match phase_state.phase {
            Phase::Beginning => {
                run_beginning_phase(&mut players, &phase_state, &mut board_state).await?
            }
            Phase::Main => run_main_phase(&mut players, &phase_state, &mut board_state).await?,
            Phase::End => run_end_phase(&mut players, &phase_state, &mut board_state).await?,
        };

        match phase_result {
            Continue => next_phase(&mut phase_state)?,
            BreakPhase(phase_break) => match phase_break {
                PhaseBreak::EndPhase => next_phase(&mut phase_state)?,
                PhaseBreak::EndTurn => next_turn(&mut phase_state)?,
                PhaseBreak::EndGame(game_result) => {
                    return Ok(game_result);
                }
            },
        }
    }
}

fn notify_all(event: GameEvent) -> Result<(), GameError> {
    Ok(())
}

fn update_phase_and_notify(phase_state: &mut PhaseState, update: UpdatePhaseState) {
    phase_state.apply_update(update);
    // Todo: watchers
}
fn update_board_and_notify(
    board_state: &mut BoardState,
    update: UpdateBoardState,
) -> Result<(), GameError> {
    board_state.apply_update(update)?;
    Ok(())
}

async fn run_beginning_phase(
    players: &mut Players,
    phase_state: &PhaseState,
    board_state: &mut BoardState,
) -> Result<GameControlFlow, GameError> {
    // Skip beginning phase for the first two turns.
    if phase_state.turn <= 2 {
        return Ok(Continue);
    }

    // Add vigor
    update_board_and_notify(
        board_state,
        UpdateBoardState::AddToVigor {
            player: phase_state.turn_player,
            diff: 1,
        },
    )?;

    // Todo: remove sakura tokens from enhancements, reshuffle deck, draw cards.

    Ok(Continue)
}

async fn run_main_phase(
    players: &mut Players,
    phase_state: &PhaseState,
    board_state: &mut BoardState,
) -> Result<GameControlFlow, GameError> {
    handle_player_actions(players, phase_state, board_state).await??;

    Ok(Continue)
}

async fn run_end_phase(
    players: &mut Players,
    phase_state: &PhaseState,
    board_state: &mut BoardState,
) -> Result<GameControlFlow, GameError> {
    // Todo: move enhancements and in-use cards to the used pile.

    Ok(Continue)
}

async fn handle_player_actions(
    players: &mut Players,
    phase_state: &PhaseState,
    board_state: &mut BoardState,
) -> Result<GameControlFlow, GameError> {
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
                .await
                .map_err(|_| GameError::PlayerCommunicationFail(turn_player_pos))?;
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
            MainPhaseAction::PlayBasicAction { action, cost } => {
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
    let phase_state = PhaseState::new(1, start_player, Phase::Beginning);

    let board_state = BoardState::new(Petals::new(10), Petals::new(0), default_player_states());

    (phase_state, board_state)
}

async fn notify_game_start(
    players: &mut Players,
    phase_state: &PhaseState,
    board_state: &BoardState,
) -> Result<(), GameError> {
    async fn notify_start(
        p: &mut (impl Player + ?Sized + Send),
        phase_state: &PhaseState,
        board_state: &BoardState,
        pos: PlayerPos,
    ) -> Result<(), GameError> {
        p.notify_game_start(
            &get_player_viewable_state(phase_state, board_state, pos),
            pos,
        )
        .await
        .map_err(|_| GameError::PlayerCommunicationFail(pos))
    }

    let (a, b) = join!(
        notify_start(
            &mut *players.p1_data,
            phase_state,
            board_state,
            PlayerPos::P1
        ),
        notify_start(
            &mut *players.p2_data,
            phase_state,
            board_state,
            PlayerPos::P2
        )
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
    notify_all(GameEvent::PerformBasicAction { player, action })?;

    let mut transfer_aura = |from, to| {
        update_board_and_notify(
            board_state,
            UpdateBoardState::TransferPetals {
                from,
                to,
                amount: 1,
            },
        )
    };
    match action {
        BasicAction::MoveForward => {
            transfer_aura(PetalPosition::Distance, PetalPosition::Aura(player))?;
        }
        BasicAction::MoveBackward => {
            transfer_aura(PetalPosition::Aura(player), PetalPosition::Distance)?;
        }
        BasicAction::Recover => {
            transfer_aura(PetalPosition::Dust, PetalPosition::Aura(player))?;
        }
        BasicAction::Focus => {
            transfer_aura(PetalPosition::Aura(player), PetalPosition::Flare(player))?;
        }
    }

    Ok(Continue)
}

fn pay_basic_action_cost(
    board_state: &mut BoardState,
    player: PlayerPos,
    cost: BasicActionCost,
) -> Result<(), GameError> {
    match cost {
        BasicActionCost::Hand(selector) => {
            update_board_and_notify(
                board_state,
                UpdateBoardState::DiscardCard { player, selector },
            )?;
        }
        BasicActionCost::Vigor => update_board_and_notify(
            board_state,
            UpdateBoardState::AddToVigor { player, diff: -1 },
        )?,
    }

    Ok(())
}

fn get_player_viewable_state(
    phase_state: &PhaseState,
    board: &BoardState,
    viewed_from: PlayerPos,
) -> ViewableState {
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
        turn_number: phase_state.turn,
        distance: board.distance.get_count(),
        dust: board.dust.get_count(),
        player_states: ViewablePlayerStates::new(
            get_player_state(PlayerPos::P1),
            get_player_state(PlayerPos::P2),
        ),
    }
}
