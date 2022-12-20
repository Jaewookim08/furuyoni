use crate::networking::{ClientConnectionReader, ClientConnectionWriter};
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameMessageFrame, GameNotification, GameRequest, GameRequestFrame,
    PlayerResponse, PlayerResponseFrame, ServerMessageFrame,
};

use std::sync::atomic::AtomicU32;
use std::sync::MutexGuard;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot, Mutex};

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

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum SendError {
    WriteError(#[from] connection::WriteError),
}

pub async fn receive_posts<T: AsyncRead + Unpin>(
    mut reader: ClientConnectionReader<T>,
    game_message_tx: mpsc::Sender<GameMessageFrame>,
) {
    loop {
        match reader.read_frame().await {
            Err(err) => {
                println!("Error occurred while reading a frame. Err: {:?}", err);
                return;
            }
            Ok(server_message) => match server_message {
                ServerMessageFrame::GameMessage(message) => {
                    game_message_tx.send(message);
                }
            },
        }
    }
}

pub async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut player_response_mailbox: mpsc::Receiver<WithSendCallback<PlayerResponseFrame>>,
    mut writer: ClientConnectionWriter<TWrite>,
) {
    while let Some(req) = player_response_mailbox.recv().await {
        let res = writer
            .write_frame(&ClientMessageFrame::PlayerResponse(req.data))
            .await;

        let _ = req.callback.send(res.map_err(|e| e.into()));
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
