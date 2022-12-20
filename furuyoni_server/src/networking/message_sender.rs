use crate::networking::post_office;
use crate::networking::post_office::WithSendCallback;
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameNotification, GameRequest, PlayerResponse, PlayerResponseFrame,
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

pub struct MessageSender<TMessage> {
    message_tx: mpsc::Sender<WithSendCallback<TMessage>>,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum SendError<TMessage> {
    PostOfficeError(#[from] post_office::SendError),
    ChannelSendError(#[from] mpsc::error::SendError<WithSendCallback<TMessage>>),
    ChannelReceiveError(#[from] oneshot::error::RecvError),
}

impl<TMessage> MessageSender<TMessage> {
    pub async fn send(&self, message: TMessage) -> Result<(), SendError<TMessage>> {
        let (send_result_tx, send_result_rx) = oneshot::channel();
        self.message_tx
            .send(WithSendCallback::new(send_result_tx, message))
            .await?;

        send_result_rx.await??;

        Ok(())
    }

    pub fn new(message_tx: mpsc::Sender<WithSendCallback<TMessage>>) -> MessageSender<TMessage> {
        MessageSender { message_tx }
    }
}
