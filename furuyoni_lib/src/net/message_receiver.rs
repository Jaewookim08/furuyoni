use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

#[derive(Error, Debug)]
pub enum RecvError {
    #[error("Channel has been closed.")]
    ChannelClosed,
}

#[async_trait]
pub trait MessageReceiver {
    type Message;

    async fn receive(&mut self) -> Result<Self::Message, RecvError>;
    fn try_receive(&mut self) -> Result<Option<Self::Message>, RecvError>;
}

#[async_trait]
impl<TMessage: Send> MessageReceiver for mpsc::Receiver<TMessage> {
    type Message = TMessage;

    async fn receive(&mut self) -> Result<TMessage, RecvError> {
        self.recv().await.ok_or(RecvError::ChannelClosed)
    }

    fn try_receive(&mut self) -> Result<Option<Self::Message>, RecvError> {
        let res = self.try_recv();
        match res {
            Ok(m) => Ok(Some(m)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(RecvError::ChannelClosed),
        }
    }
}
