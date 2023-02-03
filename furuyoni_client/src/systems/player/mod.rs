use crate::systems::picker::{PickedEvent, RequestPick};
use bevy::prelude::*;
use furuyoni_lib::net::frames::{
    GameRequest, GameToPlayerRequestData, PlayerResponse, PlayerResponseFrame,
    RequestMainPhaseAction, ResponseMainPhaseAction, WithRequestId,
};
use furuyoni_lib::net::message_channel::MessageChannelResponseError;
use furuyoni_lib::net::Responder;
use furuyoni_lib::player_actions::{BasicActionCost, MainPhaseAction, PlayBasicAction};
use furuyoni_lib::rules::{
    Phase, PlayerPos, ViewableOpponentState, ViewablePlayerState, ViewablePlayerStates,
    ViewableSelfState, ViewableState,
};
use iyes_loopless::prelude::*;

pub struct PlayerPlugin;

#[derive(Resource)]
pub struct PlayerToGameResponder(
    Box<
        dyn Responder<
                PlayerResponseFrame,
                Request = GameRequest,
                Error = MessageChannelResponseError,
            > + Send
            + Sync,
    >,
);
impl PlayerToGameResponder {
    pub fn new(
        responder: Box<
            dyn Responder<
                    PlayerResponseFrame,
                    Request = GameRequest,
                    Error = MessageChannelResponseError,
                > + Send
                + Sync,
        >,
    ) -> Self {
        Self(responder)
    }
}

#[derive(Resource, Debug)]
pub struct GameState(pub ViewableState);

// Todo: remove. GameState should not be constructed in client.
impl Default for GameState {
    fn default() -> Self {
        Self {
            0: ViewableState {
                turn_number: 0,
                turn_player: PlayerPos::P1,
                phase: Phase::Beginning,
                distance: 0,
                dust: 0,
                player_states: ViewablePlayerStates::new(
                    ViewablePlayerState::SelfState(ViewableSelfState {
                        hands: vec![],
                        deck_count: 0,
                        enhancements: vec![],
                        played_pile: vec![],
                        discard_pile: vec![],
                        vigor: 0,
                        aura: 0,
                        life: 0,
                        flare: 0,
                    }),
                    ViewablePlayerState::Opponent(ViewableOpponentState {
                        hand_count: 0,
                        deck_count: 0,
                        enhancements: vec![],
                        played_pile: vec![],
                        discard_pile_count: 0,
                        vigor: 0,
                        aura: 0,
                        life: 0,
                        flare: 0,
                    }),
                ),
            },
        }
    }
}

#[derive(Resource, Debug)]
pub struct SelfPlayerPos(pub PlayerPos);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlayerState {
    BeforeStart,
    Idle,
    SelectingMainPhaseAction,
}

// This struct is not integrated into PlayerState because
// the state needs to be Copy and Clone for some reasons.
#[derive(Resource)]
struct MainPhaseActionPickRequest {
    request: RequestMainPhaseAction,
    request_id: u32,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(PlayerState::BeforeStart)
            .add_system(player_listener)
            .add_enter_system(
                PlayerState::SelectingMainPhaseAction,
                start_pick_main_phase_action,
            )
            .add_system(
                wait_for_main_phase_action.run_in_state(PlayerState::SelectingMainPhaseAction),
            );
    }
}

fn player_listener(
    commands: Commands,
    curr_state: Res<CurrentState<PlayerState>>,
    responder: ResMut<PlayerToGameResponder>,
) {
    let res = run_player_listener(commands, curr_state, responder);

    match res {
        Ok(_) => {}
        Err(_) => {
            panic!("Todo")
        }
    }
}

fn run_player_listener(
    mut commands: Commands,
    curr_state: Res<CurrentState<PlayerState>>,
    mut responder: ResMut<PlayerToGameResponder>,
) -> Result<(), ()> {
    // Messages should at be processed at max once in a frame, to give time for state changes.
    if let Some(request) = responder.0.try_recv().map_err(|_| ())? {
        match request {
            GameRequest::RequestData(WithRequestId { request_id, data }) => match data {
                GameToPlayerRequestData::RequestMainPhaseAction(r) => {
                    if curr_state.0 != PlayerState::Idle {
                        return Err(());
                    }

                    commands.insert_resource(NextState(PlayerState::SelectingMainPhaseAction));
                    commands.insert_resource(MainPhaseActionPickRequest {
                        request: r,
                        request_id,
                    });
                }
                GameToPlayerRequestData::RequestGameStart { pos, state } => {
                    if curr_state.0 != PlayerState::BeforeStart {
                        return Err(());
                    }

                    commands.insert_resource(NextState(PlayerState::Idle));
                    commands.insert_resource(GameState { 0: state });
                    commands.insert_resource(SelfPlayerPos { 0: pos });

                    responder
                        .0
                        .response(PlayerResponseFrame::new(
                            request_id,
                            PlayerResponse::AcknowledgeGameStart,
                        ))
                        .map_err(|_| ())?;
                }
            },
            GameRequest::Notify(nt) => match nt {},
        }
    }
    Ok(())
}

fn start_pick_main_phase_action(
    mut game_state: ResMut<GameState>,
    mut event_writer: EventWriter<RequestPick>,
    req: Res<MainPhaseActionPickRequest>,
) {
    let req = &req.request;

    game_state.0 = req.state.clone();

    event_writer.send(RequestPick::new(
        req.performable_basic_actions.iter().cloned().collect(),
        true,
    ));
}

fn wait_for_main_phase_action(
    mut commands: Commands,
    mut event_reader: EventReader<PickedEvent>,
    responder: ResMut<PlayerToGameResponder>,
    req: Res<MainPhaseActionPickRequest>,
) {
    if let Some(ev) = event_reader.iter().next() {
        let action = match ev {
            PickedEvent::BasicAction(ba) => MainPhaseAction::PlayBasicAction(PlayBasicAction {
                action: *ba,
                cost: BasicActionCost::Vigor,
            }),
            PickedEvent::Skip => MainPhaseAction::EndMainPhase,
        };

        responder
            .0
            .response(PlayerResponseFrame {
                request_id: req.request_id,
                data: PlayerResponse::ResponseMainPhaseAction(ResponseMainPhaseAction { action }),
            })
            .expect("Todo");

        commands.remove_resource::<MainPhaseActionPickRequest>();
        commands.insert_resource(NextState(PlayerState::Idle));
    }
}
