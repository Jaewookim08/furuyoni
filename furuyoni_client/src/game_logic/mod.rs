use crate::game_logic::GameLogicError::InvalidRequest;
use bevy::prelude::*;
use bevy_tokio_tasks::*;
use furuyoni_lib::net::frames::{GameToPlayerRequest, PlayerToGameResponse};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::MessageSendError;
use furuyoni_lib::net::MessageRecvError;
use furuyoni_lib::rules::{PlayerPos, ViewableState};
use thiserror::Error;
use tokio::net::TcpStream;

type PlayerToGameResponder = MessageChannel<PlayerToGameResponse, GameToPlayerRequest>;

#[derive(Resource)]
pub(crate) struct BoardState(pub ViewableState);

#[derive(Resource)]
pub(crate) struct SelfPlayerPos(pub PlayerPos);

#[derive(Debug, Error)]
pub(crate) enum GameLogicError {
    #[error("Failed to receive a request from the server.")]
    RequestReceiveFailed(#[from] MessageRecvError),
    #[error("Failed to send back a response to the server")]
    ResponseSendFailed(#[from] MessageSendError),
    #[error("Received an invalid request from the server.")]
    InvalidRequest(GameToPlayerRequest),
}

pub(crate) async fn run_game(
    mut responder: PlayerToGameResponder,
    mut ctx: TaskContext,
) -> Result<(), GameLogicError> {
    // wait for the game start message
    match responder.receive().await? {
        GameToPlayerRequest::RequestGameStart { pos, state } => {
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.insert_resource(BoardState { 0: state });
                ctx.world.insert_resource(SelfPlayerPos { 0: pos });
            })
            .await;

            responder.send(PlayerToGameResponse::AcknowledgeGameStart)?;
        }
        r => return Err(InvalidRequest(r)),
    };

    // main logic loop.
    loop {
        match responder.receive().await? {
            GameToPlayerRequest::NotifyEvent(e) => {
                // board.play_event()
                todo!()
            }
            GameToPlayerRequest::RequestMainPhaseAction(req) => {
                todo!()
            }
            r => return Err(InvalidRequest(r)),
        }
    }
}
