use crate::net::with_send_callback;
use crate::net::with_send_callback::WithSendCallback;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

pub struct MessageSender<TMessage> {
    message_tx: mpsc::Sender<WithSendCallback<TMessage>>,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum SendError<TMessage> {
    SendError(#[from] with_send_callback::SendError),
    ChannelSendError(#[from] mpsc::error::SendError<WithSendCallback<TMessage>>),
    ChannelReceiveError(#[from] oneshot::error::RecvError),
}

impl<TMessage> MessageSender<TMessage> {
    pub async fn send(&self, message: TMessage) -> Result<(), SendError<TMessage>> {
        let (send_result_tx, _) = oneshot::channel();
        self.message_tx
            .send(WithSendCallback::new(send_result_tx, message))
            .await?;

        // Do not wait for the callback to be called. The callback is currently not used anywhere.

        Ok(())
    }

    pub fn new(message_tx: mpsc::Sender<WithSendCallback<TMessage>>) -> MessageSender<TMessage> {
        MessageSender { message_tx }
    }
}
