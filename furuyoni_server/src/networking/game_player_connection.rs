use crate::networking::post_office;
use crate::networking::post_office::WithSendCallback;
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameNotification, GameRequest, PlayerResponse, PlayerResponseFrame,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

pub struct Sender {
    game_message_tx: mpsc::Sender<WithSendCallback<GameMessageFrame>>,
}

pub struct Receiver {
    player_message_rx: mpsc::Receiver<PlayerResponseFrame>,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum SendError {
    PostOfficeError(#[from] post_office::SendError),
    ChannelSendError(#[from] mpsc::error::SendError<WithSendCallback<GameMessageFrame>>),
    ChannelReceiveError(#[from] oneshot::error::RecvError),
}

#[derive(Error, Debug)]
pub enum RecvError {
    #[error("Channel has been closed.")]
    ChannelClosed,
}

impl Sender {
    pub async fn send(&self, message: GameMessageFrame) -> Result<(), SendError> {
        let (send_result_tx, send_result_rx) = oneshot::channel();
        self.game_message_tx
            .send(WithSendCallback::new(send_result_tx, message))
            .await?;

        send_result_rx.await??;

        Ok(())
    }

    pub fn new(game_message_tx: mpsc::Sender<WithSendCallback<GameMessageFrame>>) -> Sender {
        Sender { game_message_tx }
    }
}

impl Receiver {
    pub async fn receive(&mut self) -> Result<PlayerResponseFrame, RecvError> {
        self.player_message_rx
            .recv()
            .await
            .ok_or(RecvError::ChannelClosed)
    }

    pub fn new(player_message_rx: mpsc::Receiver<PlayerResponseFrame>) -> Receiver {
        Receiver { player_message_rx }
    }
}
