use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
#[error("MessageSender failed.")]
pub enum MessageSendError {
    ChannelClosed,
}

pub trait MessageSender<TMessage> {
    fn send(&self, message: TMessage) -> Result<(), MessageSendError>;
}

trait MyFunc {
    type Input;
    type Output;
}

pub struct MessageMap<TInnerSender, F> {
    sender: TInnerSender,
    f: F,
}

impl<TInnerSender, F> MessageMap<TInnerSender, F> {
    fn new(sender: TInnerSender, f: F) -> Self {
        Self { sender, f }
    }
}

impl<TMessageIn, TMessageOut, TInnerSender: MessageSender<TMessageOut>, F> MessageSender<TMessageIn>
for MessageMap<TInnerSender, F>
    where
        F: Fn(TMessageIn) -> TMessageOut,
{
    fn send(&self, message: TMessageIn) -> Result<(), MessageSendError> {
        self.sender.send((self.f)(message))
    }
}

pub trait IntoMessageMap<TM> {
    fn with_map<F>(self, f: F) -> MessageMap<Self, F>
        where
            Self: Sized;
}

impl<T, TM> IntoMessageMap<TM> for T
    where
        T: MessageSender<TM>,
{
    fn with_map<F>(self, f: F) -> MessageMap<Self, F>
        where
            Self: Sized,
    {
        MessageMap::new(self, f)
    }
}

impl<TMessage> MessageSender<TMessage>
for &mpsc::Sender<TMessage>
{
    /// This function fails when the inner channel is full. This limitation is to make this function
    /// non-async and therefore easier to use with locks.
    fn send(&self, message: TMessage) -> Result<(), MessageSendError> {
        self.try_send(message)
            .map_err(|e| match e {
                TrySendError::Full(_) => {
                    panic!("The mpsc channel should never be full.")
                }
                TrySendError::Closed(_) => MessageSendError::ChannelClosed,
            })?;

        Ok(())
    }
}

impl<TMessage> MessageSender<TMessage>
for mpsc::Sender<TMessage>
{
    fn send(&self, message: TMessage) -> Result<(), MessageSendError> {
        (&self).send(message)
    }
}