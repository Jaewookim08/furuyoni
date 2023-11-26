use crate::networking::{ClientConnectionReader, ClientConnectionWriter};
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameToPlayerMessage, GameToPlayerRequest, GameToPlayerResponseFrame,
    LobbyToPlayerMessage, ServerMessageFrame,
};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum ReceivePostsError {
    #[error("Failed to send a message through a given channel.")]
    ChannelSendError,
    #[error("Failed to read a frame.")]
    FrameReadError,
}

pub async fn receive_posts<T: AsyncRead + Unpin>(
    mut reader: ClientConnectionReader<T>,
    game_request_tx: mpsc::Sender<GameToPlayerRequest>,
    game_response_tx: mpsc::Sender<GameToPlayerResponseFrame>,
) -> Result<(), ReceivePostsError> {
    loop {
        match reader.read_frame().await {
            Err(_) => {
                // Todo: Ignore error when the error is from a parsing failure.
                return Err(ReceivePostsError::FrameReadError);
            }
            Ok(message_frame) => match message_frame {
                ServerMessageFrame::GameMessage(msg) => match msg {
                    GameToPlayerMessage::Request(req) => {
                        game_request_tx
                            .try_send(req)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                    GameToPlayerMessage::Response(resp) => {
                        game_response_tx
                            .try_send(resp)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                },
                ServerMessageFrame::LobbyMessage(msg) => match msg {
                    LobbyToPlayerMessage::Request(req) => { todo!() }
                    LobbyToPlayerMessage::Response(res) => { todo!() }
                },
            },
        }
    }
}

pub async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut mailbox: mpsc::Receiver<ClientMessageFrame>,
    mut writer: ClientConnectionWriter<TWrite>,
) {
    while let Some(request) = mailbox.recv().await {
        let res = writer.write_frame(&request).await;

        if let Err(_) = res {
            panic!("Todo");
        }
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
