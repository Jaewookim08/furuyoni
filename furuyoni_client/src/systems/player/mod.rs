use crate::systems::board_system::{BoardRequest, BoardRequestQueue};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use furuyoni_lib::net::frames::{
    GameToPlayerNotification, GameToPlayerRequest, GameToPlayerRequestData, PlayerToGameResponse,
    ResponseMainPhaseAction,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::rules::{PlayerPos, ViewableState};
use futures_lite::future;
use tokio::sync::oneshot;

type PlayerToGameResponder = MessageChannel<GameToPlayerRequest, PlayerToGameResponse>;

pub struct PlayerPlugin;

#[derive(Resource, Debug)]
pub struct GameState(pub ViewableState);

#[derive(Resource, Debug)]
pub struct SelfPlayerPos(pub PlayerPos);

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum PlayerState {
    #[default]
    BeforeGameStart,
    GameStarted,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PlayerState>()
            .insert_resource(TaskRunning::Listen)
            .add_system(process_task.run_if(resource_exists::<ResponderResource>()));
    }
}

#[derive(Resource)]
pub struct ResponderResource(PlayerToGameResponder);

impl ResponderResource {
    pub fn new(
        responder: impl Responder<
                PlayerToGameResponseFrame,
                Request = GameToPlayerRequest,
                Error = MessageChannelResponseError,
            > + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            0: Box::new(responder),
        }
    }
}

#[derive(Resource)]
enum TaskRunning {
    Listen,
    ProcessDataRequest {
        request_id: u32,
        run: bevy::tasks::Task<Result<PlayerToGameResponse, ()>>,
    },
}

fn process_task(
    mut commands: Commands,
    curr_state: Res<State<PlayerState>>,
    next_state: ResMut<NextState<PlayerState>>,
    mut board_request_queue: ResMut<BoardRequestQueue>,
    mut task: ResMut<TaskRunning>,
    mut responder: ResMut<ResponderResource>,
) {
    let responder = &mut responder.0;
    let task = task.into_inner();
    match task {
        TaskRunning::Listen => {
            match responder.try_recv() {
                Ok(Some(req)) => {
                    let new_task = handle_request(
                        &mut commands,
                        &curr_state.0,
                        next_state.into_inner(),
                        board_request_queue.into_inner(),
                        req,
                    )
                    .expect("handling request failed");

                    *task = new_task;
                }
                Ok(None) => {
                    // No message. Do nothing.
                }
                Err(_) => {
                    todo!("Game->player channel has been closed.")
                }
            }
        }
        TaskRunning::ProcessDataRequest { request_id, run } => {
            if let Some(process_res) = future::block_on(future::poll_once(run)) {
                match process_res {
                    Ok(data) => {
                        responder
                            .response(WithRequestId::new(*request_id, data))
                            .expect("Sending response failed");

                        *task = TaskRunning::Listen;
                    }
                    Err(()) => todo!("processing data request failed"),
                }
            }
        }
    }
}

fn handle_request(
    commands: &mut Commands,
    curr_state: &PlayerState,
    next_state: &mut NextState<PlayerState>,
    board_request_queue: &mut BoardRequestQueue,
    req: GameToPlayerRequest,
) -> Result<TaskRunning, ()> {
    let pool = AsyncComputeTaskPool::get();

    match req {
        GameToPlayerRequest::RequestData(WithRequestId { request_id, data }) => match data {
            GameToPlayerRequestData::RequestMainPhaseAction(r) => {
                if curr_state != &PlayerState::GameStarted {
                    return Err(());
                }
                let (tx, rx) = oneshot::channel();
                board_request_queue
                    .0
                    .push_back(BoardRequest::GetMainPhaseAction {
                        query: r,
                        callback: tx,
                    });

                let new_task = pool.spawn(async move {
                    rx.await.map_err(|e| ()).map(|action| {
                        PlayerToGameResponse::MainPhaseAction(ResponseMainPhaseAction { action })
                    })
                });
                Ok(TaskRunning::ProcessDataRequest {
                    request_id,
                    run: new_task,
                })
            }
            GameToPlayerRequestData::RequestGameStart { pos, state } => {
                if curr_state != &PlayerState::BeforeGameStart {
                    return Err(());
                }

                commands.insert_resource(GameState { 0: state });
                commands.insert_resource(SelfPlayerPos { 0: pos });
                next_state.set(PlayerState::GameStarted);

                let new_task =
                    pool.spawn(async move { Ok(PlayerToGameResponse::AcknowledgeGameStart) });

                Ok(TaskRunning::ProcessDataRequest {
                    request_id,
                    run: new_task,
                })
            }
        },
        GameToPlayerRequest::Notify(notification) => {
            match notification {
                GameToPlayerNotification::Event(game_event) => {
                    board_request_queue
                        .0
                        .push_back(BoardRequest::PlayEvent(game_event));
                }
            }

            Ok(TaskRunning::Listen)
        }
    }
}
