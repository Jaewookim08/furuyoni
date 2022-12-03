use crate::networking::{GameConnectionReader, GameConnectionWriter};
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameMessageFrame, GameNotification, GameRequest, GameRequestFrame,
    PlayerResponse, PlayerResponseFrame, ServerMessageFrame,
};
use rand::Rng;
use std::sync::atomic::AtomicU32;
use std::sync::MutexGuard;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Mutex};

pub struct PostOffice;

#[derive(Debug)]
pub struct WithSendCallback<T> {
    callback: oneshot::Sender<Result<(), SendError>>,
    data: T,
}

impl<T> WithSendCallback<T> {
    pub fn new(callback: oneshot::Sender<Result<(), SendError>>, data: T) -> Self {
        Self { callback, data }
    }
}

pub struct PostGameMessage {
    send_end_callback: oneshot::Sender<Result<(), SendError>>,
    data: PostGameMessageData,
}

pub enum PostGameMessageData {}

pub struct GameRequestWithCallback {
    callback: oneshot::Sender<Result<PlayerResponse, SendError>>,
    inner: GameRequest,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum SendError {
    WriteError(#[from] connection::WriteError),
}

impl PostOffice {
    pub async fn receive_posts(
        &self,
        mut reader: GameConnectionReader,
        player_message_tx: mpsc::Sender<PlayerResponseFrame>,
    ) {
        loop {
            match reader.read_frame().await {
                Err(err) => {
                    println!("Error occurred while reading a frame. Err: {:?}", err);
                    continue;
                }
                Ok(client_message_frame) => match client_message_frame {
                    ClientMessageFrame::PlayerResponse(response) => {
                        player_message_tx.send(response);
                    }
                },
            }
        }
    }

    pub async fn handle_send_requests(
        &self,
        mut send_game_message_mailbox: mpsc::Receiver<WithSendCallback<GameMessageFrame>>,
        mut writer: GameConnectionWriter,
    ) {
        while let Some(request) = send_game_message_mailbox.recv().await {
            let res = writer
                .write_frame(&ServerMessageFrame::GameMessage(request.data))
                .await;

            let _ = request.callback.send(res.map_err(|e| e.into()));
        }

        println!("[PostOffice] 'handle_send_requests' has ended.")
    }
}
