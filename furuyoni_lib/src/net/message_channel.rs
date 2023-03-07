use crate::net::frames::base::WithRequestId;
use crate::net::message_sender::{MessageSendError, MessageSender};
use crate::net::responder::Responder;
use crate::net::{MessageReceiver, MessageRecvError, RequestError, Requester};
use async_trait::async_trait;
use thiserror::Error;

pub struct MessageChannel<Sender, Receiver> {
    sender: Sender,
    receiver: Receiver,
}

impl<Sender, Receiver> MessageChannel<Sender, Receiver> {
    pub fn new(sender: Sender, receiver: Receiver) -> Self {
        Self { sender, receiver }
    }
}

#[async_trait]
impl<Sender, SendMessage, Receiver, RecvMessage> Requester<SendMessage>
    for MessageChannel<Sender, Receiver>
where
    Sender: MessageSender<WithRequestId<SendMessage>> + Send,
    Receiver: MessageReceiver<Message = WithRequestId<RecvMessage>> + Send,
    SendMessage: Send + 'static,
{
    type Response = RecvMessage;
    type Error = RequestError;

    async fn request(&mut self, request: SendMessage) -> Result<Self::Response, Self::Error> {
        let id = rand::random();
        // Todo: move id checking logic into a separate struct.

        self.sender.send_message(WithRequestId::new(id, request))?;

        let WithRequestId { request_id, data } = self.receiver.receive().await?;

        if request_id != id {
            Err(RequestError::RequestIdMismatch)
        } else {
            Ok(data)
        }
    }
}

#[derive(Error, Debug)]
#[error("Responder failed.")]
pub enum MessageChannelResponseError {
    SenderError(#[from] MessageSendError),
    ReceiverError(#[from] MessageRecvError),
}

#[async_trait]
impl<Sender, Receiver, RecvMessage, SendMessage> Responder<SendMessage>
    for MessageChannel<Sender, Receiver>
where
    Sender: MessageSender<SendMessage> + Send,
    Receiver: MessageReceiver<Message = RecvMessage> + Send,
{
    type Request = RecvMessage;
    type Error = MessageChannelResponseError;

    async fn recv(&mut self) -> Result<RecvMessage, Self::Error> {
        self.receiver.receive().await.map_err(|e| e.into())
    }

    fn try_recv(&mut self) -> Result<Option<Self::Request>, Self::Error> {
        self.receiver.try_receive().map_err(|e| e.into())
    }

    fn response(&self, message: SendMessage) -> Result<(), Self::Error> {
        self.sender.send_message(message)?;
        Ok(())
    }
}
