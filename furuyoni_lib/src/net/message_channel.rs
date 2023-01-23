use crate::net::frames::WithRequestId;
use crate::net::message_sender::MessageSender;
use crate::net::{MessageReceiver, RequestError, Requester};
use async_trait::async_trait;

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

        self.sender.send_message(WithRequestId::new(id, request))?;

        let response_frame = self.receiver.receive().await?;

        let response = response_frame
            .try_get(id)
            .ok_or(RequestError::RequestIdMismatch)?;

        Ok(response)
    }
}
