use crate::game_logic::GameLogicError::InvalidRequest;
use crate::systems::board_plugin::BoardError;
use crate::systems::picker::{ PickBasicActionResult, PickMainPhaseActionResult };
use crate::systems::{ board_plugin, picker };
use bevy::prelude::*;
use bevy_tokio_tasks::*;
use furuyoni_lib::net::frames::{ GameToPlayerRequest, PlayerToGameResponse };
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::MessageSendError;
use furuyoni_lib::net::MessageRecvError;
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::player_actions::MainPhaseAction;
use furuyoni_lib::rules::GameResult;
use std::sync::Arc;
use thiserror::Error;

type PlayerToGameResponder = MessageChannel<PlayerToGameResponse, GameToPlayerRequest>;

#[derive(Debug, Error)]
pub(crate) enum GameLogicError {
    #[error("Failed to receive a request from the server: {0}")] RequestReceiveFailed(
        #[from] MessageRecvError,
    ),
    #[error("Failed to send back a response to the server : {0}")] ResponseSendFailed(
        #[from] MessageSendError,
    ),
    #[error("Received an invalid request from the server: {0:?}")] InvalidRequest(
        GameToPlayerRequest,
    ),
    #[error("Board error :{0}")] BoardError(#[from] BoardError),
}

pub(crate) async fn run_game(
    mut responder: PlayerToGameResponder,
    ctx: TaskContext
) -> Result<(), GameLogicError> {
    // wait for the initial state

    let state = match responder.receive().await? {
        GameToPlayerRequest::InitializeGameState(state) => state,
        r => {
            return Err(InvalidRequest(r));
        }
    };

    // wait for the game start message
    let me = match responder.receive().await? {
        GameToPlayerRequest::RequestGameStart { pos } => { pos }
        r => {
            return Err(InvalidRequest(r));
        }
    };

    // initialize in the main thread.
    ctx.run_on_main_thread(move |ctx| {
        board_plugin::initialize_board(ctx.world, state, me);
    }).await;

    // notify that the client has successfully started the game.
    responder.send(PlayerToGameResponse::AcknowledgeGameStart)?;

    // main logic loop.
    let result = loop {
        match responder.receive().await? {
            GameToPlayerRequest::NotifyEvent(event) => {
                board_plugin::apply_event(&ctx, event, me).await?;

                if let GameEvent::GameEnd { result } = event {
                    break result;
                }
            }
            GameToPlayerRequest::RequestMainPhaseAction(req) => {
                let allowed_costs = Arc::new(req.available_basic_action_costs);
                let allowed_actions = Arc::new(req.performable_basic_actions);

                let action = loop {
                    let picked_main_action = picker::pick_main_phase_action(
                        ctx.clone(),
                        allowed_costs.clone()
                    ).await;
                    match picked_main_action {
                        PickMainPhaseActionResult::PayBasicActionCost(cost) => {
                            let picked_basic_action = picker::pick_basic_action(
                                &ctx,
                                allowed_actions.clone()
                            ).await;

                            match picked_basic_action {
                                PickBasicActionResult::BasicAction(basic_action) => {
                                    break MainPhaseAction::PlayBasicAction {
                                        action: basic_action,
                                        cost,
                                    };
                                }
                                PickBasicActionResult::Cancel => {
                                    continue;
                                }
                            }
                        }
                        PickMainPhaseActionResult::EndMainPhase => {
                            break MainPhaseAction::EndMainPhase;
                        }
                    }
                };

                responder.send(PlayerToGameResponse::MainPhaseAction(action))?;
            }
            GameToPlayerRequest::CheckGameState(state) => {
                board_plugin::check_game_state(&ctx, state).await;
            }
            r => {
                return Err(InvalidRequest(r));
            }
        }
    };

    info!("Game ended.");
    match result {
        GameResult::Draw => {
            info!("Draw.");
        }
        GameResult::Winner(p) => {
            if me == p {
                info!("You won!");
            } else {
                info!("You lost...");
            }
        }
    }

    Ok(())
}
