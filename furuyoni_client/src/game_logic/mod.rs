use crate::game_logic::GameLogicError::InvalidRequest;
use crate::systems::picker;
use bevy::prelude::*;
use bevy_tokio_tasks::*;
use furuyoni_lib::net::frames::{GameToPlayerRequest, PlayerToGameResponse};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::MessageSendError;
use furuyoni_lib::net::MessageRecvError;
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::player_actions::{BasicActionCost, MainPhaseAction};
use furuyoni_lib::rules::states::*;
use furuyoni_lib::rules::PlayerPos;
use std::sync::Arc;
use thiserror::Error;

type PlayerToGameResponder = MessageChannel<PlayerToGameResponse, GameToPlayerRequest>;

#[derive(Resource)]
pub(crate) struct BoardState(pub StateView);

#[derive(Resource)]
pub(crate) struct SelfPlayerPos(pub PlayerPos);

#[derive(Debug, Error)]
pub(crate) enum GameLogicError {
    #[error("Failed to receive a request from the server: {0}")]
    RequestReceiveFailed(#[from] MessageRecvError),
    #[error("Failed to send back a response to the server : {0}")]
    ResponseSendFailed(#[from] MessageSendError),
    #[error("Received an invalid request from the server: {0:?}")]
    InvalidRequest(GameToPlayerRequest),
    #[error("Tried to do an invalid update to the game state: {0}")]
    InvalidUpdate(#[from] InvalidGameViewUpdateError),
}

pub(crate) async fn run_game(
    mut responder: PlayerToGameResponder,
    mut ctx: TaskContext,
) -> Result<(), GameLogicError> {
    // wait for the initial state
    match responder.receive().await? {
        GameToPlayerRequest::InitializeGameState(state) => {
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.insert_resource(BoardState { 0: state });
            })
            .await;
        }
        r => return Err(InvalidRequest(r)),
    }

    // wait for the game start message
    match responder.receive().await? {
        GameToPlayerRequest::RequestGameStart { pos } => {
            ctx.run_on_main_thread(move |ctx| {
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
            GameToPlayerRequest::NotifyEvent(event) => {
                // Todo: move to board.
                match event {
                    GameEvent::StateUpdated(update) => {
                        ctx.run_on_main_thread(move |ctx| -> Result<(), GameLogicError> {
                            let mut state = ctx.world.get_resource_mut::<BoardState>().unwrap();
                            state.0.apply_update(&update)?;
                            Ok(())
                        })
                        .await?;
                    }
                    GameEvent::PerformBasicAction { .. } => {}
                }
            }
            GameToPlayerRequest::RequestMainPhaseAction(req) => {
                let allowed_actions = Arc::new(req.performable_basic_actions);
                let picked = picker::pick_basic_action(ctx.clone(), allowed_actions, true).await;

                // skip checking the validity of the picked value. The server will do it for us.
                let main_phase_action = match picked {
                    None => MainPhaseAction::EndMainPhase,
                    Some(action) => {
                        MainPhaseAction::PlayBasicAction {
                            action,
                            cost: /* todo */BasicActionCost::Vigor,
                        }
                    }
                };
                responder.send(PlayerToGameResponse::MainPhaseAction(main_phase_action))?;
            }
            GameToPlayerRequest::CheckGameState(state) => {
                ctx.run_on_main_thread(move |ctx| {
                    let resource = ctx
                        .world
                        .get_resource::<BoardState>()
                        .expect("Resource BoardState is missing.");
                    if resource.0 != state {
                        eprintln!("Error: state mismatch.");
                        eprintln!("server state: {:?}", state);
                        eprintln!("client state: {:?}", resource.0);
                        todo!("handle state mismatch: resynchronize...")
                    }
                })
                .await;
            }
            r => return Err(InvalidRequest(r)),
        }
    }
}
