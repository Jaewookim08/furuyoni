use crate::networking::{ServerConnectionReader, ServerConnectionWriter};
use furuyoni_lib::net::connection::WriteError;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameMessageFrame, GameNotification, GameRequest, PlayerMessageFrame,
    PlayerResponse, PlayerResponseFrame, PlayerToGameRequestFrame, ServerMessageFrame,
};
use furuyoni_lib::net::with_send_callback::WithCallback;
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
            Err(_) => {
                // Todo: Ignore error when the error is from a parsing failure.
                return Err(ReceivePostsError::FrameReadError);
            }
            Ok(client_message_frame) => match client_message_frame {
                ClientMessageFrame::PlayerMessage(message) => match message {
                    PlayerMessageFrame::Response(response) => {
                        player_response_tx
                            .try_send(response)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                    PlayerMessageFrame::Request(request) => {
                        player_request_tx
                            .try_send(request)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
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

        if let Err(_) = res {
            panic!("Todo");
        }

        let _ = request.callback.send(res.map_err(|e| e.into()));
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
