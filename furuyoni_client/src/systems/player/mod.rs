use crate::systems::picker::{PickedEvent, RequestPick};
use bevy::prelude::*;
use furuyoni_lib::net::frames::{
    GameToPlayerNotification, GameToPlayerRequest, GameToPlayerRequestData, PlayerToGameResponse,
    PlayerToGameResponseFrame, RequestMainPhaseAction, ResponseMainPhaseAction, WithRequestId,
};
use furuyoni_lib::net::message_channel::MessageChannelResponseError;
use furuyoni_lib::net::Responder;
use furuyoni_lib::player_actions::{BasicActionCost, MainPhaseAction, PlayBasicAction};
use furuyoni_lib::rules::{
    Phase, PlayerPos, ViewableOpponentState, ViewablePlayerState, ViewablePlayerStates,
    ViewableSelfState, ViewableState,
};
pub struct PlayerPlugin;

#[derive(Resource)]
pub struct PlayerToGameResponder(
    Box<
        dyn Responder<
                PlayerToGameResponseFrame,
                Request = GameToPlayerRequest,
                Error = MessageChannelResponseError,
            > + Send
            + Sync,
    >,
);
impl PlayerToGameResponder {
    pub fn new(
        responder: Box<
            dyn Responder<
                    PlayerToGameResponseFrame,
                    Request = GameToPlayerRequest,
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

#[derive(Resource, Debug)]
pub struct SelfPlayerPos(pub PlayerPos);

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum PlayerState {
    #[default]
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
        app.add_state::<PlayerState>()
            .add_system(player_listener)
            .add_system(
                start_pick_main_phase_action
                    .in_schedule(OnEnter(PlayerState::SelectingMainPhaseAction)),
            )
            .add_system(
                wait_for_main_phase_action.in_set(OnUpdate(PlayerState::SelectingMainPhaseAction)),
            );
    }
}

fn player_listener(
    commands: Commands,
    curr_state: Res<State<PlayerState>>,
    responder: ResMut<PlayerToGameResponder>,
    next_state: ResMut<NextState<PlayerState>>,
) {
    let res = run_player_listener(commands, curr_state, responder, next_state);

    match res {
        Ok(_) => {}
        Err(_) => {
            panic!("Todo")
        }
    }
}

fn run_player_listener(
    mut commands: Commands,
    curr_state: Res<State<PlayerState>>,
    mut responder: ResMut<PlayerToGameResponder>,
    mut next_state: ResMut<NextState<PlayerState>>,
) -> Result<(), ()> {
    // Messages should at be processed at max once in a frame, to give time for state changes.
    if let Some(request) = responder.0.try_recv().map_err(|_| ())? {
        match request {
            GameToPlayerRequest::RequestData(WithRequestId { request_id, data }) => match data {
                GameToPlayerRequestData::RequestMainPhaseAction(r) => {
                    if curr_state.0 != PlayerState::Idle {
                        return Err(());
                    }

                    commands.insert_resource(MainPhaseActionPickRequest {
                        request: r,
                        request_id,
                    });
                    next_state.set(PlayerState::SelectingMainPhaseAction);
                }
                GameToPlayerRequestData::RequestGameStart { pos, state } => {
                    if curr_state.0 != PlayerState::BeforeStart {
                        return Err(());
                    }

                    next_state.set(PlayerState::Idle);
                    commands.insert_resource(GameState { 0: state });
                    commands.insert_resource(SelfPlayerPos { 0: pos });

                    responder
                        .0
                        .response(PlayerToGameResponseFrame::new(
                            request_id,
                            PlayerToGameResponse::AcknowledgeGameStart,
                        ))
                        .map_err(|_| ())?;
                }
            },
            GameToPlayerRequest::Notify(nt) => match nt {
                GameToPlayerNotification::Event(_) => {}
            },
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
    mut next_state: ResMut<NextState<PlayerState>>,
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
            .response(PlayerToGameResponseFrame {
                request_id: req.request_id,
                data: PlayerToGameResponse::MainPhaseAction(ResponseMainPhaseAction { action }),
            })
            .expect("Todo");

        commands.remove_resource::<MainPhaseActionPickRequest>();
        next_state.set(PlayerState::Idle);
    }
}
