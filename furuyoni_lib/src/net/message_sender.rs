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
    ChannelSendError(#[from] mpsc::error::TrySendError<WithSendCallback<TMessage>>),
    ChannelReceiveError(#[from] oneshot::error::RecvError),
}

impl<TMessage> MessageSender<TMessage> {
    /// This function fails when the inner channel is full. This limitation is to make this function
    /// non-async and therefore easier to use with locks.
    pub fn send(&self, message: TMessage) -> Result<(), SendError<TMessage>> {
        let (send_result_tx, _) = oneshot::channel();
        self.message_tx
            .try_send(WithSendCallback::new(send_result_tx, message))?;

        // Do not wait for the callback to be called. The callback is a legacy and currently not used anywhere.

        Ok(())
    }

    pub fn new(message_tx: mpsc::Sender<WithSendCallback<TMessage>>) -> MessageSender<TMessage> {
        MessageSender { message_tx }
    }
}
