use crate::networking::post_office;
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameNotification, GameRequest, PlayerResponse, PlayerResponseFrame,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

pub struct GameToPlayerSender {
    game_message_tx: mpsc::Sender<post_office::WithSendCallback<GameMessageFrame>>,
}

pub struct GameToPlayerReceiver {
    player_message_rx: mpsc::Receiver<PlayerResponseFrame>,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
enum SendError {
    PostOfficeError(#[from] post_office::SendError),
    ChannelSendError(
        #[from] mpsc::error::SendError<post_office::WithSendCallback<GameMessageFrame>>,
    ),
    ChannelReceiveError(#[from] oneshot::error::RecvError),
}

#[derive(Error, Debug)]
enum RecvError {
    #[error("Channel has been closed.")]
    ChannelClosed,
}

impl GameToPlayerSender {
    async fn send(&self, message: GameMessageFrame) -> Result<(), SendError> {
        let (send_result_tx, send_result_rx) = oneshot::channel();
        self.game_message_tx
            .send(post_office::WithSendCallback::new(send_result_tx, message))
            .await?;

        send_result_rx.await??;

        Ok(())
    }
}

impl GameToPlayerReceiver {
    async fn receive(&mut self) -> Result<PlayerResponseFrame, RecvError> {
        self.player_message_rx
            .recv()
            .await
            .ok_or(RecvError::ChannelClosed)
    }
}
