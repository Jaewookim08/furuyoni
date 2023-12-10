mod game_controlflow;
mod game_recorder;
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
use crate::game::game_recorder::{run_recorder, GameRecorder};
use crate::game::states::*;
use crate::game_watcher::{GameObserver, NotifyFailedError};
use furuyoni_lib::rules::attack::{AttackDamage, Damage};
use furuyoni_lib::rules::cards::{Card, CardSelector, CardsPosition};
use furuyoni_lib::rules::events::{GameEvent, UpdateGameState};
use furuyoni_lib::rules::states::*;
use states::player_state::PlayerState;
use std::future::Future;
use std::marker::{Send, Sync};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use thiserror::Error;
use tokio::join;
use tokio::sync::mpsc;

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
    #[error("{0}")]
    EventFilterError(#[from] EventFilterError),
}

pub enum GameResult {
    Draw,
    Winner(PlayerPos),
}

type Players = PlayersData<Box<dyn Player + Send + Sync>>;

#[derive(Debug, Copy, Clone, PartialEq, Neg)]
pub struct Vigor(i32);

pub(crate) struct GameSetup {
    game: Game,
    event_rx: mpsc::Receiver<GameEvent>,
    recorder: Arc<GameRecorder>,
}

struct Game {
    state: GameState,
    players: Players,
    event_tx: Option<mpsc::Sender<GameEvent>>,
}
pub fn create_game(
    player_1: Box<dyn Player + Sync + Send>,
    player_2: Box<dyn Player + Sync + Send>,
) -> (GameSetup, Arc<GameRecorder>) {
    let (tx, rx) = mpsc::channel(20);

    let state = initialize_game_states();
    let recorder = Arc::new(GameRecorder::new(state.clone()));
    let game = Game {
        state,
        players: Players::new(player_1, player_2),
        event_tx: Some(tx),
    };

    let setup = GameSetup {
        game,
        event_rx: rx,
        recorder: recorder.clone(),
    };
    (setup, recorder)
}

impl GameSetup {
    pub async fn run(mut self) -> Result<GameResult, GameError> {
        // broadcast state.
        let GameSetup {
            game,
            event_rx,
            recorder,
        } = self;

        let recorder_task = tokio::spawn(run_recorder(event_rx, recorder));

        let result = game.run().await;

        let recorder_result = recorder_task.await.unwrap();
        // Todo:

        result
    }
}

impl Game {
    pub async fn run(mut self) -> Result<GameResult, GameError> {
        // broadcast state.
        for (p, player) in self.players.iter_mut() {
            player.initialize_state(&filter_state(ObservePosition::RelativeTo(p), &self.state))?;
        }

        self.notify_game_start().await?;

        // Define phase modifying functions. The phase state should only be modified using these functions.
        fn next_phase(game: &mut Game) -> Result<(), GameError> {
            match game.state.phase {
                Phase::Beginning => {
                    game.update_state_and_notify(UpdateGameState::SetPhase(Phase::Main))?;
                }
                Phase::Main => {
                    game.update_state_and_notify(UpdateGameState::SetPhase(Phase::End))?;
                }
                Phase::End => next_turn(game)?,
            }
            Ok(())
        }

        fn next_turn(game: &mut Game) -> Result<(), GameError> {
            game.update_state_and_notify(UpdateGameState::SetTurn {
                turn_player: game.state.turn_player.other(),
                turn: game.state.turn + 1,
            })?;

            game.update_state_and_notify(UpdateGameState::SetPhase(Phase::Beginning))?;
            Ok(())
        }

        // phase loop
        loop {
            let phase_result = match self.state.phase {
                Phase::Beginning => self.run_beginning_phase().await?,
                Phase::Main => self.run_main_phase().await?,
                Phase::End => self.run_end_phase().await?,
            };

            match phase_result {
                Continue => next_phase(&mut self)?,
                BreakPhase(phase_break) => match phase_break {
                    PhaseBreak::EndPhase => next_phase(&mut self)?,
                    PhaseBreak::EndTurn => next_turn(&mut self)?,
                    PhaseBreak::EndGame(game_result) => {
                        return Ok(game_result);
                    }
                },
            }
        }
    }

    fn update_state_and_notify(&mut self, update: UpdateGameState) -> Result<(), GameError> {
        self.notify_all(GameEvent::StateUpdated(update))?;
        // The order between notifying and applying should be preserved.
        self.state.apply_update(update)?;
        Ok(())
    }

    fn notify_all(&mut self, event: GameEvent) -> Result<(), GameError> {
        for (pos, player) in self.players.iter_mut() {
            player.notify_event(filter_event(
                &self.state,
                ObservePosition::RelativeTo(pos),
                event,
            )?)?;
        }
        if let Some(tx) = &self.event_tx {
            match tx.try_send(event) {
                Err(_) => {
                    eprintln!("Failed to send event to the game recorder.");
                    drop(self.event_tx.take());
                }
                Ok(()) => {}
            }
        }

        Ok(())
    }

    async fn notify_game_start(&mut self) -> Result<(), GameError> {
        // Todo: iterator to task 한 다음 전부 await하는 그런 거 없나.
        async fn notify_start(
            p: &mut (impl Player + ?Sized + Send),
            pos: PlayerPos,
        ) -> Result<(), GameError> {
            p.request_game_start(pos)
                .await
                .map_err(|_| GameError::PlayerCommunicationFail(pos))
        }

        let PlayersData { p1_data, p2_data } = &mut self.players;

        // Todo: tokio joinset
        let (a, b) = join!(
            notify_start(p1_data.deref_mut(), PlayerPos::P1),
            notify_start(p2_data.deref_mut(), PlayerPos::P2)
        );

        (a?, b?);

        Ok(())
    }

    async fn run_beginning_phase(&mut self) -> Result<GameControlFlow, GameError> {
        // Skip beginning phase for the first two turns.
        if self.state.turn <= 2 {
            return Ok(Continue);
        }

        // Add vigor
        let turn_player = self.state.turn_player;

        self.add_to_vigor(turn_player, 1)?;

        for _ in 0..2 {
            self.try_draw_card(turn_player).await??;
        }

        // Todo: remove sakura tokens from enhancements, reshuffle deck, ...
        Ok(Continue)
    }

    async fn run_main_phase(&mut self) -> Result<GameControlFlow, GameError> {
        self.handle_player_actions().await??;

        Ok(Continue)
    }

    async fn run_end_phase(&mut self) -> Result<GameControlFlow, GameError> {
        // Todo: move enhancements and in-use cards to the used pile.

        Ok(Continue)
    }

    async fn handle_player_actions(&mut self) -> Result<GameControlFlow, GameError> {
        // main phase actions loop.
        loop {
            let turn_player = self.state.turn_player;

            let doable_basic_actions = [
                BasicAction::MoveForward,
                BasicAction::MoveBackward,
                BasicAction::Focus,
                BasicAction::Recover,
            ]
            .into_iter()
            .filter(|action| self.can_play_basic_action(turn_player, *action))
            .collect();

            let playable_cards = vec![]; // Todo:
            let available_costs = (0..self.state.player_states[turn_player].hand.len())
                .map(|i| BasicActionCost::Hand(HandSelector(i)))
                .chain([BasicActionCost::Vigor].into_iter())
                .filter(|cost| self.can_pay_basic_action_cost(turn_player, *cost))
                .collect();

            // Todo: some reusable retry function.
            let mut cnt_try = 0;
            let action = loop {
                let viewable_state =
                    filter_state(ObservePosition::RelativeTo(turn_player), &self.state);

                let action = self.players[turn_player]
                    .main_phase_action(
                        &viewable_state,
                        &playable_cards,
                        &doable_basic_actions,
                        &available_costs,
                    )
                    .await
                    .map_err(|_| GameError::PlayerCommunicationFail(turn_player))?;

                if self.can_play_main_phase_action(turn_player, action) {
                    break action;
                }
                cnt_try += 1;
                if cnt_try >= GET_ACTION_RETRY_TIMES {
                    return Err(GameError::InvalidActionRequested(turn_player));
                }
            };

            debug_assert!(self.can_play_main_phase_action(turn_player, action));

            self.play_main_phase_action(turn_player, action)??;
        }
    }

    async fn try_draw_card(&mut self, player: PlayerPos) -> Result<GameControlFlow, GameError> {
        let from = CardSelector::Last(CardsPosition::Deck(player));
        let to = CardSelector::PushLast(CardsPosition::Hand(player));

        if self.can_transfer_card(from, to) {
            self.transfer_card(from, to)?;
        } else {
            self.apply_attack_damage(
                player,
                AttackDamage {
                    aura_damage: Some(1),
                    life_damage: Some(1),
                },
            )
            .await??;
        }

        Ok(Continue)
    }

    async fn apply_attack_damage(
        &mut self,
        to: PlayerPos,
        damage: AttackDamage,
    ) -> Result<GameControlFlow, GameError> {
        // todo: ask user to select where to get the damage.

        self.apply_damage_try_best(PetalsPosition::Life(to), damage.life_damage.unwrap())
            .await??;
        Ok(Continue)
    }

    async fn apply_damage_try_best(
        &mut self,
        petals_pos: PetalsPosition,
        amount: u32,
    ) -> Result<GameControlFlow, GameError> {
        let amount = std::cmp::min(self.state.petals(petals_pos).count, amount);

        let move_to = match petals_pos {
            PetalsPosition::Aura(_) | PetalsPosition::Flare(_) => PetalsPosition::Dust,
            PetalsPosition::Life(p) => PetalsPosition::Flare(p),
            _ => panic!(),
        };

        self.transfer_petals(petals_pos, move_to, amount)?;
        Ok(Continue)
    }

    fn add_to_vigor(&mut self, player: PlayerPos, diff: i32) -> Result<(), GameError> {
        const MAX_VIGOR: i32 = 2;

        let vigor = self.state.player_states[player].vigor;
        let real_diff = std::cmp::min(diff, MAX_VIGOR - vigor.0);

        self.update_state_and_notify(UpdateGameState::AddToVigor {
            player,
            diff: real_diff,
        })?;

        Ok(())
    }

    fn play_main_phase_action(
        &mut self,
        player: PlayerPos,
        action: MainPhaseAction,
    ) -> Result<GameControlFlow, GameError> {
        match action {
            MainPhaseAction::EndMainPhase => Ok(BreakPhase(PhaseBreak::EndPhase)),
            MainPhaseAction::PlayBasicAction { action, cost } => {
                self.pay_basic_action_cost(player, cost)?;
                self.play_basic_action(player, action)?;
                Ok(Continue)
            }
            MainPhaseAction::PlayCard(_) => {
                todo!();
            }
        }
    }

    fn pay_basic_action_cost(
        &mut self,
        player: PlayerPos,
        cost: BasicActionCost,
    ) -> Result<(), GameError> {
        match cost {
            BasicActionCost::Hand(selector) => {
                self.discard_card_from_hand(player, selector)?;
            }
            BasicActionCost::Vigor => self.add_to_vigor(player, -1)?,
        }

        Ok(())
    }
    fn discard_card_from_hand(
        &mut self,
        player: PlayerPos,
        hand_selector: HandSelector,
    ) -> Result<(), GameError> {
        self.transfer_card(
            CardSelector::Index {
                position: CardsPosition::Hand(player),
                index: hand_selector.0,
            },
            CardSelector::PushLast(CardsPosition::Discards(player)),
        )
    }

    fn transfer_card(&mut self, from: CardSelector, to: CardSelector) -> Result<(), GameError> {
        self.update_state_and_notify(UpdateGameState::TransferCard { from, to })
    }

    fn play_basic_action(
        &mut self,
        player: PlayerPos,
        action: BasicAction,
    ) -> Result<GameControlFlow, GameError> {
        self.notify_all(GameEvent::PerformBasicAction { player, action })?;

        let mut transfer_petals = |from, to| self.transfer_petals(from, to, 1);
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

    fn transfer_petals(
        &mut self,
        from: PetalsPosition,
        to: PetalsPosition,
        amount: u32,
    ) -> Result<GameControlFlow, GameError> {
        self.update_state_and_notify(UpdateGameState::TransferPetals { from, to, amount })?;

        Ok(Continue)
    }

    fn master_interval(&self) -> i32 {
        2
    }

    fn can_play_basic_action(&self, player: PlayerPos, action: BasicAction) -> bool {
        let mut can_transfer_petals = |from, to| self.can_transfer_petals(from, to, 1);

        match action {
            BasicAction::MoveForward => {
                can_transfer_petals(PetalsPosition::Distance, PetalsPosition::Aura(player))
                    && self.state.distance.count as i32 > self.master_interval()
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

    fn can_pay_basic_action_cost(&self, player: PlayerPos, cost: BasicActionCost) -> bool {
        match cost {
            BasicActionCost::Hand(selector) => self.can_discard_card_from_hand(player, selector),
            BasicActionCost::Vigor => self.can_add_to_vigor(player, -1),
        }
    }

    fn can_add_to_vigor(&self, player: PlayerPos, diff: i32) -> bool {
        self.state.player_states[player].vigor.0 + diff >= 0
    }

    fn can_play_main_phase_action(&self, player: PlayerPos, action: MainPhaseAction) -> bool {
        match action {
            MainPhaseAction::EndMainPhase => true,
            MainPhaseAction::PlayBasicAction { action, cost } => {
                self.can_pay_basic_action_cost(player, cost)
                    && self.can_play_basic_action(player, action)
            }
            MainPhaseAction::PlayCard(_) => false,
        }
    }

    fn can_transfer_petals(&self, from: PetalsPosition, to: PetalsPosition, amount: u32) -> bool {
        if self.state.petals(from).count < amount {
            return false;
        }
        let to_petals = self.state.petals(to);
        if let Some(max) = to_petals.max
            && to_petals.count + amount > max
        {
            return false;
        }

        true
    }

    fn can_transfer_card(&self, from: CardSelector, to: CardSelector) -> bool {
        let from_cards = self.state.cards(from.cards_position());
        let from_index = from.index(from_cards.len());
        if from_cards.len() <= from_index {
            return false;
        }
        let to_cards = self.state.cards(to.cards_position());
        let to_index = to.index(to_cards.len());

        if to_cards.len() < to_index {
            return false;
        }

        true
    }

    fn can_discard_card_from_hand(&self, player: PlayerPos, hand_selector: HandSelector) -> bool {
        // Todo: poision, etc..
        // Todo: 내가 하는 것과 상대가 하는 것 구분해야 할 수도.

        self.can_transfer_card(
            CardSelector::Index {
                position: CardsPosition::Hand(player),
                index: hand_selector.0,
            },
            CardSelector::PushLast(CardsPosition::Discards(player)),
        );
        true
    }
}

#[derive(Debug, Error)]
#[error("Failed to filter an event: {0:?}")]
struct EventFilterError(GameEvent);
fn filter_event(
    state: &GameState,
    position: ObservePosition,
    event: GameEvent,
) -> Result<GameEvent, EventFilterError> /* Todo: to Result<Option<GameEvent>, ?> */
{
    let (can_view_p1, can_view_p2, can_view_master) = {
        match position {
            ObservePosition::RelativeTo(p) => (p == PlayerPos::P1, p == PlayerPos::P2, false),
            ObservePosition::MasterView => (true, true, true),
            ObservePosition::ByStander => (false, false, false),
        }
    };
    let can_view_p = PlayersData::new(can_view_p1, can_view_p2);

    let e = match event {
        e @ GameEvent::StateUpdated(update) => GameEvent::StateUpdated(match update {
            u @ UpdateGameState::SetTurn { .. }
            | u @ UpdateGameState::SetPhase(_)
            | u @ UpdateGameState::TransferPetals { .. }
            | u @ UpdateGameState::AddToVigor { .. } => u,
            u @ UpdateGameState::TransferCard { from, to } => {
                let is_from_open = match from.cards_position() {
                    CardsPosition::Deck(_) => can_view_master,

                    CardsPosition::Discards(p) | CardsPosition::Hand(p) => can_view_p[p],

                    CardsPosition::Playing(_)
                    | CardsPosition::Enhancements(_)
                    | CardsPosition::Played(_) => true,
                };

                if is_from_open {
                    u
                } else {
                    UpdateGameState::TransferCardFromHidden {
                        from: from.cards_position(),
                        to,
                        card: state.select_card(from).ok_or(EventFilterError(e))?,
                    }
                }
            }
            UpdateGameState::TransferCardFromHidden { .. } => {
                panic!();
            }
        }),
        e @ GameEvent::PerformBasicAction { .. } => e,
    };

    Ok(e)
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

fn filter_state(viewed_from: ObservePosition, state: &GameState) -> StateView {
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
