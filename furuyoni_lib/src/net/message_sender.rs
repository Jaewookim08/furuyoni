use crate::net::with_send_callback::WithCallback;
use crate::net::{connection, with_send_callback};
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot};

#[derive(Error, Debug)]
#[error("MessageSender failed.")]
pub enum MessageSendError {
    ChannelClosed,
}

pub trait MessageSender<TMessage> {
    fn send_message(&self, message: TMessage) -> Result<(), MessageSendError>;
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
    fn send_message(&self, message: TMessageIn) -> Result<(), MessageSendError> {
        self.sender.send_message((self.f)(message))
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
    for &mpsc::Sender<WithCallback<TMessage, connection::WriteError>>
{
    /// This function fails when the inner channel is full. This limitation is to make this function
    /// non-async and therefore easier to use with locks.
    fn send_message(&self, message: TMessage) -> Result<(), MessageSendError> {
        let (send_result_tx, _) = oneshot::channel();
        self.try_send(WithCallback::new(send_result_tx, message))
            .map_err(|e| match e {
                TrySendError::Full(_) => {
                    panic!("The mpsc channel should never be full.")
                }
                TrySendError::Closed(_) => MessageSendError::ChannelClosed,
            })?;

        // Do not wait for the callback to be called. The callback is currently not used anywhere
        // and is just left for potential future cases where sender has to check if the message
        // has been correctly processed.

        Ok(())
    }
}

impl<TMessage> MessageSender<TMessage>
    for mpsc::Sender<WithCallback<TMessage, connection::WriteError>>
{
    fn send_message(&self, message: TMessage) -> Result<(), MessageSendError> {
        (&self).send_message(message)
    }
}

impl<TMessage> MessageSender<TMessage>
    for std::sync::Arc<mpsc::Sender<WithCallback<TMessage, connection::WriteError>>>
{
    fn send_message(&self, message: TMessage) -> Result<(), MessageSendError> {
        (**self).send_message(message)
    }
}
