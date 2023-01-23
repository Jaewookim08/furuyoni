use crate::networking::post_office::ReceivePostsError::{ChannelSendError, FrameReadError};
use crate::networking::{ServerConnectionReader, ServerConnectionWriter};
use furuyoni_lib::net::connection;
use furuyoni_lib::net::connection::WriteError;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameMessageFrame, GameNotification, GameRequest, PlayerMessageFrame,
    PlayerResponse, PlayerResponseFrame, PlayerToGameRequestFrame, ServerMessageFrame,
};
use furuyoni_lib::net::with_send_callback::WithCallback;
use rand::Rng;
use std::sync::atomic::AtomicU32;
use std::sync::MutexGuard;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot, Mutex};

#[derive(Error, Debug)]
pub enum ReceivePostsError {
    #[error("Failed to send a message through a given channel.")]
    ChannelSendError,
    #[error("Failed to read a frame.")]
    FrameReadError,
}

pub async fn receive_posts<T: AsyncRead + Unpin>(
    mut reader: ServerConnectionReader<T>,
    player_response_tx: mpsc::Sender<PlayerResponseFrame>,
    player_request_tx: mpsc::Sender<PlayerToGameRequestFrame>,
) -> Result<(), ReceivePostsError> {
    loop {
        match reader.read_frame().await {
            Err(err) => {
                println!("Error occurred while reading a frame. Err: {:?}", err);
                // Todo: Ignore error when the error is from a parsing failure.
                return Err(FrameReadError);
            }
            Ok(client_message_frame) => match client_message_frame {
                ClientMessageFrame::PlayerMessage(message) => match message {
                    PlayerMessageFrame::Response(response) => {
                        player_response_tx
                            .try_send(response)
                            .map_err(|_| ChannelSendError)?;
                    }
                    PlayerMessageFrame::Request(request) => {
                        player_request_tx
                            .try_send(request)
                            .map_err(|_| ChannelSendError)?;
                    }
                },
            },
        }
    }
}

pub async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut send_game_message_mailbox: mpsc::Receiver<WithCallback<GameMessageFrame, WriteError>>,
    mut writer: ServerConnectionWriter<TWrite>,
) {
    while let Some(request) = send_game_message_mailbox.recv().await {
        let res = writer
            .write_frame(&ServerMessageFrame::GameMessage(request.data))
            .await;

        let _ = request.callback.send(res.map_err(|e| e.into()));
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
