use crate::networking::{ClientConnectionReader, ClientConnectionWriter};
use furuyoni_lib::net::connection::WriteError;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameMessageFrame, GameRequest, GameToPlayerResponseFrame,
    ServerMessageFrame,
};
use furuyoni_lib::net::with_send_callback::WithCallback;
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
    game_request_tx: mpsc::Sender<GameRequest>,
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
                    GameMessageFrame::Request(req) => {
                        game_request_tx
                            .try_send(req)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                    GameMessageFrame::Response(resp) => {
                        game_response_tx
                            .try_send(resp)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                },
            },
        }
    }
}

pub async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut mailbox: mpsc::Receiver<WithCallback<ClientMessageFrame, WriteError>>,
    mut writer: ClientConnectionWriter<TWrite>,
) {
    while let Some(request) = mailbox.recv().await {
        let res = writer.write_frame(&request.data).await;

        if let Err(_) = res {
            panic!("Todo");
        }

        let _ = request.callback.send(res.map_err(|e| e.into()));
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
