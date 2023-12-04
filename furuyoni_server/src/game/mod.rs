mod game_controlflow;
mod observable_game;
mod states;

use furuyoni_lib::rules::states::Petals;

use crate::players::Player;
use derive_more::Neg;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, HandSelector, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::{ObservePosition, PlayerPos};

use crate::game::game_controlflow::GameControlFlow::{BreakPhase, Continue};
use crate::game::game_controlflow::{GameControlFlow, PhaseBreak};
use crate::game::observable_game::{event_filter_information, ObservableGame};
use crate::game::states::*;
use crate::game_watcher::{GameObserver, NotifyFailedError};
use furuyoni_lib::rules::cards::{Card, CardSelector, CardsPosition};
use furuyoni_lib::rules::events::{GameEvent, UpdateGameState};
use furuyoni_lib::rules::states::*;
use states::player_state::PlayerState;
use std::collections::VecDeque;
use std::future::Future;
use std::marker::{Send, Sync};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
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

struct ObserverWithPos {
    position: ObservePosition,
    observer: Box<dyn GameObserver + Send>,
}

struct GameHandle {
    state: GameState,
    observers: Vec<ObserverWithPos>,
}

pub(crate) struct Game {
    handle: Arc<Mutex<GameHandle>>,
}

impl Game {
    pub fn create_game() -> (Game, ObservableGame) {
        let handle = Arc::new(Mutex::new(GameHandle {
            state: initialize_game_states(),
            observers: Vec::new(),
        }));

        let game = Game {
            handle: handle.clone(),
        };

        let observable = ObservableGame::new(handle);

        (game, observable)
    }

    pub async fn run(
        mut self,
        player_1: Box<dyn Player + Sync + Send>,
        player_2: Box<dyn Player + Sync + Send>,
    ) -> Result<GameResult, GameError> {
        let mut players = Players::new(player_1, player_2);

        broadcast_viewable_state(self.handle.lock().unwrap().deref_mut(), &mut players)?;
        self.notify_game_start(&mut players).await?;

        // Define phase modifying functions.
        fn next_phase(handle: &mut GameHandle, players: &mut Players) -> Result<(), GameError> {
            match handle.state.phase {
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
                    turn_player: handle.state.turn_player.other(),
                    turn: handle.state.turn + 1,
                },
            )?;

            update_state_and_notify(handle, players, UpdateGameState::SetPhase(Phase::Beginning))?;
            Ok(())
        }

        // phase loop
        loop {
            let current_phase = self.handle.lock().unwrap().state.phase;
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
        async fn notify_start(
            p: &mut (impl Player + ?Sized + Send),
            pos: PlayerPos,
        ) -> Result<(), GameError> {
            p.request_game_start(pos)
                .await
                .map_err(|_| GameError::PlayerCommunicationFail(pos))
        }

        let PlayersData { p1_data, p2_data } = players;

        let (a, b) = join!(
            notify_start(p1_data.deref_mut(), PlayerPos::P1),
            notify_start(p2_data.deref_mut(), PlayerPos::P2)
        );

        (a?, b?);

        Ok(())
    }

    async fn run_beginning_phase(
        &self,
        players: &mut Players,
    ) -> Result<GameControlFlow, GameError> {
        let mut handle = self.handle.lock().unwrap();
        // Skip beginning phase for the first two turns.
        if handle.state.turn <= 2 {
            return Ok(Continue);
        }

        // Add vigor
        let turn_player = handle.state.turn_player;

        add_to_vigor(&mut handle, players, turn_player, 1)?;

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
        let turn_player = self.handle.lock().unwrap().state.turn_player;

        // main phase actions loop.
        loop {
            let (doable_basic_actions, playable_cards, available_costs) = {
                let handle = self.handle.lock().unwrap();
                let doable_basic_actions = [
                    BasicAction::MoveForward,
                    BasicAction::MoveBackward,
                    BasicAction::Focus,
                    BasicAction::Recover,
                ]
                .into_iter()
                .filter(|action| can_play_basic_action(&handle.state, turn_player, *action))
                .collect();

                let playable_cards = vec![]; // Todo:
                let available_costs = (0..handle.state.player_states[turn_player].hand.len())
                    .map(|i| BasicActionCost::Hand(HandSelector(i)))
                    .chain([BasicActionCost::Vigor].into_iter())
                    .filter(|cost| can_pay_basic_action_cost(&handle.state, turn_player, *cost))
                    .collect();
                (doable_basic_actions, playable_cards, available_costs)
            };

            // Todo: some reusable retry function.
            let mut cnt_try = 0;
            let action = loop {
                let viewable_state = {
                    let handle = self.handle.lock().unwrap();
                    get_viewable_state(ObservePosition::RelativeTo(turn_player), &handle.state)
                };

                let action = players[turn_player]
                    .get_main_phase_action(
                        &viewable_state,
                        &playable_cards,
                        &doable_basic_actions,
                        &available_costs,
                    )
                    .await
                    .map_err(|_| GameError::PlayerCommunicationFail(turn_player))?;
                // Todo: handle result.

                let mut handle = self.handle.lock().unwrap();

                if can_play_main_phase_action(&handle.state, turn_player, action) {
                    break action;
                }
                cnt_try += 1;
                if cnt_try >= GET_ACTION_RETRY_TIMES {
                    return Err(GameError::InvalidActionRequested(turn_player));
                }
            };

            let mut handle = self.handle.lock().unwrap();
            assert!(can_play_main_phase_action(
                &handle.state,
                turn_player,
                action
            ));

            play_main_phase_action(&mut handle, players, turn_player, action)??;
        }
    }
}
fn update_state_and_notify(
    handle: &mut GameHandle,
    players: &mut Players,
    update: UpdateGameState,
) -> Result<(), GameError> {
    handle.state.apply_update(update)?;

    notify_all(
        players,
        &mut handle.observers,
        GameEvent::StateUpdated(update),
    )?;
    Ok(())
}

fn broadcast_viewable_state(
    handle: &mut GameHandle,
    players: &mut Players,
) -> Result<(), GameError> {
    for p in PlayerPos::iter() {
        players[p].initialize_state(&get_viewable_state(
            ObservePosition::RelativeTo(p),
            &handle.state,
        ))?;
    }
    for ObserverWithPos { observer, position } in handle.observers.iter_mut() {
        // ignore observer errors.
        // Todo: remove from list when error occurs.
        let _ = observer.initialize_state(&get_viewable_state(*position, &handle.state));
    }
    Ok(())
}

fn notify_all(
    players: &mut Players,
    observers: &mut Vec<ObserverWithPos>,
    event: GameEvent,
) -> Result<(), GameError> {
    for p in PlayerPos::iter() {
        players[p].notify_event(event_filter_information(
            ObservePosition::RelativeTo(p),
            event,
        ))?;
    }
    for ObserverWithPos { observer, position } in observers.iter_mut() {
        // ignore observer errors.
        // Todo: remove from list when error occurs.
        let _ = observer.notify_event(event_filter_information(*position, event));
    }
    Ok(())
}

fn initialize_game_states() -> GameState {
    // Select starting player.
    let start_player = if rand::random::<bool>() {
        PlayerPos::P1
    } else {
        PlayerPos::P2
    };

    GameState::new(
        1,
        start_player,
        Phase::Beginning,
        Petals::new(10, Some(10)),
        Petals::new(0, None),
        default_player_states(),
    )
}

/// Return default player states. Only used for debugging.
fn default_player_states() -> PlayerStates {
    let p1_state = PlayerState {
        deck: Vec::from([
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
        deck: Vec::from([
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

fn get_master_interval(state: &GameState) -> i32 {
    2
}

fn can_play_basic_action(state: &GameState, player: PlayerPos, action: BasicAction) -> bool {
    let mut can_transfer_petals = |from, to| can_transfer_petals(state, from, to, 1);

    match action {
        BasicAction::MoveForward => {
            can_transfer_petals(PetalsPosition::Distance, PetalsPosition::Aura(player))
                && state.distance.count as i32 > get_master_interval(state)
        }
        BasicAction::MoveBackward => {
            can_transfer_petals(PetalsPosition::Aura(player), PetalsPosition::Distance)
        }
        BasicAction::Recover => {
            can_transfer_petals(PetalsPosition::Dust, PetalsPosition::Aura(player))
        }
        BasicAction::Focus => {
            can_transfer_petals(PetalsPosition::Aura(player), PetalsPosition::Flare(player))
        }
    }
}

fn play_basic_action(
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    action: BasicAction,
) -> Result<GameControlFlow, GameError> {
    notify_all(
        players,
        &mut handle.observers,
        GameEvent::PerformBasicAction { player, action },
    )?;

    let mut transfer_petals = |from, to| transfer_petals(handle, players, from, to, 1);
    match action {
        BasicAction::MoveForward => {
            transfer_petals(PetalsPosition::Distance, PetalsPosition::Aura(player))?;
        }
        BasicAction::MoveBackward => {
            transfer_petals(PetalsPosition::Aura(player), PetalsPosition::Distance)?;
        }
        BasicAction::Recover => {
            transfer_petals(PetalsPosition::Dust, PetalsPosition::Aura(player))?;
        }
        BasicAction::Focus => {
            transfer_petals(PetalsPosition::Aura(player), PetalsPosition::Flare(player))?;
        }
    }

    Ok(Continue)
}

fn can_pay_basic_action_cost(state: &GameState, player: PlayerPos, cost: BasicActionCost) -> bool {
    match cost {
        BasicActionCost::Hand(selector) => can_discard_card_from_hand(state, player, selector),
        BasicActionCost::Vigor => can_add_to_vigor(state, player, -1),
    }
}

fn pay_basic_action_cost(
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    cost: BasicActionCost,
) -> Result<(), GameError> {
    match cost {
        BasicActionCost::Hand(selector) => {
            discard_card_from_hand(handle, players, player, selector)?;
        }
        BasicActionCost::Vigor => add_to_vigor(handle, players, player, -1)?,
    }

    Ok(())
}

fn can_add_to_vigor(state: &GameState, player: PlayerPos, diff: i32) -> bool {
    state.player_states[player].vigor.0 + diff >= 0
}

fn add_to_vigor(
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    diff: i32,
) -> Result<(), GameError> {
    const MAX_VIGOR: i32 = 2;

    let vigor = handle.state.player_states[player].vigor;
    let real_diff = std::cmp::min(diff, MAX_VIGOR - vigor.0);

    update_state_and_notify(
        handle,
        players,
        UpdateGameState::AddToVigor {
            player,
            diff: real_diff,
        },
    )?;

    Ok(())
}

fn can_play_main_phase_action(
    state: &GameState,
    player: PlayerPos,
    action: MainPhaseAction,
) -> bool {
    match action {
        MainPhaseAction::EndMainPhase => true,
        MainPhaseAction::PlayBasicAction { action, cost } => {
            can_pay_basic_action_cost(state, player, cost)
                && can_play_basic_action(state, player, action)
        }
        MainPhaseAction::PlayCard(_) => false,
    }
}

fn play_main_phase_action(
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    action: MainPhaseAction,
) -> Result<GameControlFlow, GameError> {
    match action {
        MainPhaseAction::EndMainPhase => Ok(BreakPhase(PhaseBreak::EndPhase)),
        MainPhaseAction::PlayBasicAction { action, cost } => {
            pay_basic_action_cost(handle, players, player, cost)?;
            play_basic_action(handle, players, player, action)?;
            Ok(Continue)
        }
        MainPhaseAction::PlayCard(_) => {
            todo!();
        }
    }
}

fn get_viewable_state(viewed_from: ObservePosition, state: &GameState) -> StateView {
    let player_states = &state.player_states;

    StateView {
        turn_player: state.turn_player,
        phase: state.phase,
        turn: state.turn,
        distance: state.distance.clone(),
        dust: state.dust.clone(),
        player_states: PlayerStateViews::new(
            player_states[PlayerPos::P1].as_viewed_from(PlayerPos::P1, viewed_from),
            player_states[PlayerPos::P2].as_viewed_from(PlayerPos::P2, viewed_from),
        ),
    }
}

fn can_transfer_petals(
    state: &GameState,
    from: PetalsPosition,
    to: PetalsPosition,
    amount: u32,
) -> bool {
    if state.get_petals(from).count < amount {
        return false;
    }
    let to_petals = state.get_petals(to);
    if let Some(max) = to_petals.max && to_petals.count + amount > max {
        return false
    }

    true
}

fn transfer_petals(
    handle: &mut GameHandle,
    players: &mut Players,
    from: PetalsPosition,
    to: PetalsPosition,
    amount: u32,
) -> Result<(), GameError> {
    update_state_and_notify(
        handle,
        players,
        UpdateGameState::TransferPetals { from, to, amount },
    )
}

fn can_transfer_cards(state: &GameState, from: CardSelector, to: CardSelector) -> bool {
    if state.get_cards(from.position).len() <= from.index {
        return false;
    }
    let to_cards = state.get_cards(to.position);
    if to_cards.len() < to.index {
        return false;
    }

    true
}

fn transfer_cards(
    handle: &mut GameHandle,
    players: &mut Players,
    from: CardSelector,
    to: CardSelector,
) -> Result<(), GameError> {
    update_state_and_notify(handle, players, UpdateGameState::TransferCard { from, to })
}

fn can_discard_card_from_hand(
    state: &GameState,
    player: PlayerPos,
    hand_selector: HandSelector,
) -> bool {
    // Todo: poision, etc..
    // Todo: 내가 하는 것과 상대가 하는 것 구분해야 할 수도.

    let discard_pile_len = state.player_states[player].discard_pile.len();
    can_transfer_cards(
        state,
        CardSelector {
            position: CardsPosition::Hand(player),
            index: hand_selector.0,
        },
        CardSelector {
            position: CardsPosition::Discards(player),
            index: discard_pile_len,
        },
    );
    true
}

fn discard_card_from_hand(
    handle: &mut GameHandle,
    players: &mut Players,
    player: PlayerPos,
    hand_selector: HandSelector,
) -> Result<(), GameError> {
    let discard_pile_len = handle.state.player_states[player].discard_pile.len();
    transfer_cards(
        handle,
        players,
        CardSelector {
            position: CardsPosition::Hand(player),
            index: hand_selector.0,
        },
        CardSelector {
            position: CardsPosition::Discards(player),
            index: discard_pile_len,
        },
    )
}
