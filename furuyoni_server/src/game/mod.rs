mod game_controlflow;
mod states;

use furuyoni_lib::rules::states::petals::Petals;

use crate::players::Player;
use derive_more::Neg;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, HandSelector, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::{PetalPosition, Phase, PlayerPos};

use crate::game::game_controlflow::GameControlFlow::{BreakPhase, Continue};
use crate::game::game_controlflow::{GameControlFlow, PhaseBreak};
use crate::game::states::*;
use crate::game_watcher::{GameObserver, NotifyFailedError};
use crate::players;
use furuyoni_lib::rules::cards::Card;
use furuyoni_lib::rules::events::{GameEvent, UpdateGameState};
use furuyoni_lib::rules::states::*;
use states::player_state::PlayerState;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::{Send, Sync};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use thiserror::Error;

const GET_ACTION_RETRY_TIMES: usize = 3;

#[derive(Error, Debug)]
pub(crate) enum GameError {
    #[error("Failed to communicate with a player.")]
    PlayerCommunicationFail(PlayerPos),
    #[error("An invalid action has been requested from the player.")]
    InvalidActionRequested(PlayerPos),
    #[error("{0}")]
    InvalidGameUpdate(#[from] InvalidGameUpdateError),
    #[error("{0}")]
    NotifyFailed(#[from] NotifyFailedError),
}

pub enum GameResult {
    Draw,
    Winner(PlayerPos),
}

type Players = PlayersData<Box<dyn Player + Send + Sync>>;

#[derive(Debug, Copy, Clone, PartialEq, Neg)]
pub struct Vigor(i32);

struct GameHandle {
    state: GameState,
    observers: Vec<Box<dyn GameObserver + Send>>,
}

pub(crate) struct Game {
    handle: Arc<Mutex<GameHandle>>,
}

pub(crate) struct ObservableGame {
    handle: Arc<Mutex<GameHandle>>,
}

impl Game {
    pub fn create_game() -> (Game, ObservableGame) {
        let shared = Arc::new(Mutex::new(GameHandle {
            state: initialize_game_states(),
            observers: Vec::new(),
        }));

        let game = Game {
            handle: shared.clone(),
        };

        let observable = ObservableGame { handle: shared };

        (game, observable)
    }

    pub async fn run(
        self,
        player_1: Box<dyn Player + Sync + Send>,
        player_2: Box<dyn Player + Sync + Send>,
    ) -> Result<GameResult, GameError> {
        let mut players = Players::new(player_1, player_2);

        // Todo: notify_game_start(&mut players, &phase_state, &board_state).await?;

        // Define phase modifying functions.
        fn next_phase(handle: &mut GameHandle, players: &mut Players) -> Result<(), GameError> {
            match handle.state.phase_state.phase {
                Phase::Beginning => {
                    update_state_and_notify(
                        handle,
                        players,
                        UpdateGameState::SetPhase(Phase::Main),
                    )?;
                }
                Phase::Main => {
                    update_state_and_notify(
                        handle,
                        players,
                        UpdateGameState::SetPhase(Phase::End),
                    )?;
                }
                Phase::End => next_turn(handle, players)?,
            }
            Ok(())
        }

        fn next_turn(handle: &mut GameHandle, players: &mut Players) -> Result<(), GameError> {
            update_state_and_notify(
                handle,
                players,
                UpdateGameState::SetTurn {
                    turn_player: handle.state.phase_state.turn_player.other(),
                    turn: handle.state.phase_state.turn + 1,
                },
            )?;

            update_state_and_notify(handle, players, UpdateGameState::SetPhase(Phase::Beginning))?;
            Ok(())
        }

        // phase loop
        loop {
            let current_phase = self.handle.lock().unwrap().state.phase_state.phase;
            let phase_result: GameControlFlow = match current_phase {
                Phase::Beginning => self.run_beginning_phase(&mut players).await?,
                Phase::Main => self.run_main_phase(&mut players).await?,
                Phase::End => self.run_end_phase(&mut players).await?,
            };

            let mut handle = self.handle.lock().unwrap();
            match phase_result {
                Continue => next_phase(&mut handle, &mut players)?,
                BreakPhase(phase_break) => match phase_break {
                    PhaseBreak::EndPhase => next_phase(&mut handle, &mut players)?,
                    PhaseBreak::EndTurn => next_turn(&mut handle, &mut players)?,
                    PhaseBreak::EndGame(game_result) => {
                        return Ok(game_result);
                    }
                },
            }
        }
    }

    async fn notify_game_start(&mut self, players: &mut Players) -> Result<(), GameError> {
        todo!()
        // async fn notify_start(
        //     p: &mut (impl Player + ?Sized + Send),
        //     phase_state: &PhaseState,
        //     board_state: &BoardState,
        //     pos: PlayerPos,
        // ) -> Result<(), GameError> {
        //     p.check_game_start(
        //         &get_player_viewable_state(phase_state, board_state, pos),
        //         pos,
        //     )
        //     .await
        //     .map_err(|_| GameError::PlayerCommunicationFail(pos))
        // }
        //
        // let (a, b) = join!(
        //     notify_start(
        //         &mut *players.p1_data,
        //         phase_state,
        //         board_state,
        //         PlayerPos::P1
        //     ),
        //     notify_start(
        //         &mut *players.p2_data,
        //         phase_state,
        //         board_state,
        //         PlayerPos::P2
        //     )
        // );
        //
        // (a?, b?);
        //
        // Ok(())
    }

    async fn run_beginning_phase(
        &self,
        players: &mut Players,
    ) -> Result<GameControlFlow, GameError> {
        let mut handle = self.handle.lock().unwrap();
        // Skip beginning phase for the first two turns.
        if handle.state.phase_state.turn <= 2 {
            return Ok(Continue);
        }

        // Add vigor
        let turn_player = handle.state.phase_state.turn_player;
        update_state_and_notify(
            &mut handle,
            players,
            UpdateGameState::AddToVigor {
                player: turn_player,
                diff: 1,
            },
        )?;

        // Todo: remove sakura tokens from enhancements, reshuffle deck, draw cards.
        Ok(Continue)
    }

    async fn run_main_phase(&self, players: &mut Players) -> Result<GameControlFlow, GameError> {
        self.handle_player_actions(players).await??;

        Ok(Continue)
    }

    async fn run_end_phase(&self, players: &mut Players) -> Result<GameControlFlow, GameError> {
        // Todo: move enhancements and in-use cards to the used pile.

        Ok(Continue)
    }

    async fn handle_player_actions(
        &self,
        players: &mut Players,
    ) -> Result<GameControlFlow, GameError> {
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
                let (turn_player_pos, viewable_state) = {
                    let handle = self.handle.lock().unwrap();
                    let turn_player_pos = handle.state.phase_state.turn_player;
                    (
                        turn_player_pos,
                        get_player_viewable_state(&handle.state, turn_player_pos),
                    )
                };

                let action = players[turn_player_pos]
                    .get_main_phase_action(
                        &viewable_state,
                        &playable_cards,
                        &doable_basic_actions,
                        &available_costs,
                    )
                    .await
                    .map_err(|_| GameError::PlayerCommunicationFail(turn_player_pos))?;
                // Todo: handle result.

                let mut handle = self.handle.lock().unwrap();

                if validate_main_phase_action(&handle.state.board_state, &action) {
                    break action;
                }
                cnt_try += 1;
                if cnt_try >= GET_ACTION_RETRY_TIMES {
                    return Err(GameError::InvalidActionRequested(turn_player_pos));
                }
            };

            let mut handle = self.handle.lock().unwrap();
            let turn_player_pos = handle.state.phase_state.turn_player;

            match action {
                MainPhaseAction::EndMainPhase => return Ok(Continue),
                MainPhaseAction::PlayBasicAction { action, cost } => {
                    pay_basic_action_cost(&mut handle, players, turn_player_pos, cost)?;
                    play_basic_action(&mut handle, players, turn_player_pos, action)?;
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
}
fn update_state_and_notify(
    handle: &mut GameHandle,
    players: &mut Players,
    update: UpdateGameState,
) -> Result<(), GameError> {
    handle.state.apply_update(&update)?;

    let e = GameEvent::StateUpdated(update);
    for p in PlayerPos::iter() {
        players[p].notify_event(&e)?;
    }
    for observer in handle.observers.iter_mut() {
        // ignore observer errors.
        // Todo: remove from list when error occurs.
        let _ = observer.notify_event(&e);
    }

    Ok(())
}

fn notify_all(event: GameEvent) -> Result<(), GameError> {
    Ok(())
}

fn initialize_game_states() -> GameState {
    // Select starting player.
    let start_player = if rand::random::<bool>() {
        PlayerPos::P1
    } else {
        PlayerPos::P2
    };

    // Initialize states.
    let phase_state = PhaseState::new(1, start_player, Phase::Beginning);

    let board_state = BoardState::new(
        Petals::new(10, Some(10)),
        Petals::new(0, None),
        default_player_states(),
    );

    GameState::new(phase_state, board_state)
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
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    action: BasicAction,
) -> Result<GameControlFlow, GameError> {
    notify_all(GameEvent::PerformBasicAction { player, action })?;

    let mut transfer_aura = |from, to| {
        update_state_and_notify(
            handle,
            players,
            UpdateGameState::TransferPetals {
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
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    cost: BasicActionCost,
) -> Result<(), GameError> {
    match cost {
        BasicActionCost::Hand(selector) => {
            update_state_and_notify(
                handle,
                players,
                UpdateGameState::DiscardCard { player, selector },
            )?;
        }
        BasicActionCost::Vigor => update_state_and_notify(
            handle,
            players,
            UpdateGameState::AddToVigor { player, diff: -1 },
        )?,
    }

    Ok(())
}

fn get_player_viewable_state(state: &GameState, viewed_from: PlayerPos) -> ViewableState {
    let GameStateInner {
        board_state,
        phase_state,
    } = state.deref();
    let player_states = &board_state.player_states;

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
        distance: board_state.distance.clone(),
        dust: board_state.dust.clone(),
        player_states: ViewablePlayerStates::new(
            get_player_state(PlayerPos::P1),
            get_player_state(PlayerPos::P2),
        ),
    }
}
