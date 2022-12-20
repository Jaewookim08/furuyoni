use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum RecvError {
    #[error("Channel has been closed.")]
    ChannelClosed,
}

pub struct MessageReceiver<TMessage> {
    message_rx: mpsc::Receiver<TMessage>,
}

impl<TMessage> MessageReceiver<TMessage> {
    pub async fn receive(&mut self) -> Result<TMessage, RecvError> {
        self.message_rx.recv().await.ok_or(RecvError::ChannelClosed)
    }

    pub fn new(message_rx: mpsc::Receiver<TMessage>) -> MessageReceiver<TMessage> {
        MessageReceiver { message_rx }
    }
}
