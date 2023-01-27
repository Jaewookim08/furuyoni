use crate::systems::picker::{PickedEvent, RequestPick};
use crate::systems::player::PlayerState::Idle;
use crate::GameState;
use bevy::prelude::*;
use furuyoni_lib::net::frames::{
    GameRequest, GameToPlayerRequestData, PlayerResponse, PlayerResponseFrame,
    RequestMainPhaseAction, ResponseMainPhaseAction, WithRequestId,
};
use furuyoni_lib::net::message_channel::MessageChannelResponseError;
use furuyoni_lib::net::Responder;
use furuyoni_lib::player_actions::{BasicActionCost, MainPhaseAction, PlayBasicAction};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlayerState {
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
        app.add_loopless_state(PlayerState::Idle)
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
    while let Some(request) = responder.0.try_recv().map_err(|_| ())? {
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
        commands.insert_resource(NextState(Idle));
    }
}
