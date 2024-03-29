use crate::networking::{ServerConnectionReader, ServerConnectionWriter};
use furuyoni_lib::net::frames::{ClientMessageFrame, PlayerToGameMessage, ServerMessageFrame, PlayerToLobbyMessage, PlayerToGameResponse, PlayerToGameRequest, PlayerToLobbyResponse, PlayerToLobbyRequest};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc};

#[derive(Error, Debug)]
pub enum ReceivePostsError {
    #[error("Failed to send a message through a given channel.")]
    ChannelSendError,
    #[error("Failed to read a frame.")]
    FrameReadError,
}

pub async fn receive_posts<T: AsyncRead + Unpin>(
    mut reader: ServerConnectionReader<T>,
    player_to_game_response_tx: mpsc::Sender<PlayerToGameResponse>,
    player_to_game_request_tx: mpsc::Sender<PlayerToGameRequest>,
    player_to_lobby_response_tx: mpsc::Sender<PlayerToLobbyResponse>,
    player_to_lobby_request_tx: mpsc::Sender<PlayerToLobbyRequest>
) -> Result<(), ReceivePostsError> {
    loop {
        match reader.read_frame().await {
            Err(_) => {
                // Todo: Ignore error when the error is from a parsing failure.
                return Err(ReceivePostsError::FrameReadError);
            }
            Ok(client_message_frame) => match client_message_frame {
                ClientMessageFrame::PlayerToGameMessage(message) => match message {
                    PlayerToGameMessage::Response(response) => {
                        player_to_game_response_tx
                            .try_send(response)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                    PlayerToGameMessage::Request(request) => {
                        player_to_game_request_tx
                            .try_send(request)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                },
                ClientMessageFrame::PlayerToLobbyMessage(message) => match message{
                    PlayerToLobbyMessage::Response(response) => {
                        player_to_lobby_response_tx
                            .try_send(response)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                    PlayerToLobbyMessage::Request(request) => {
                        player_to_lobby_request_tx
                            .try_send(request)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                }
            },
        }
    }
}

pub async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut send_message_mailbox: mpsc::Receiver<ServerMessageFrame>,
    mut writer: ServerConnectionWriter<TWrite>,
) {
    while let Some(request) = send_message_mailbox.recv().await {
        let res = writer
            .write_frame(&request)
            .await;

        if let Err(_) = res {
            panic!("Todo");
        }
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
