use crate::net::message_sender::{MessageSendError, MessageSender};
use crate::net::{MessageReceiver, MessageRecvError};

pub struct MessageChannel<SendT, RecvT> {
    sender: Box<dyn MessageSender<SendT> + Send + Sync>,
    receiver: Box<dyn MessageReceiver<Message = RecvT> + Send + Sync>,
}

impl<SendT, RecvT> MessageChannel<SendT, RecvT> {
    pub fn new(
        sender: impl MessageSender<SendT> + Send + Sync + 'static,
        receiver: impl MessageReceiver<Message = RecvT> + Send + Sync + 'static,
    ) -> Self {
        Self {
            sender: Box::new(sender),
            receiver: Box::new(receiver),
        }
    }

    pub fn send(&self, message: SendT) -> Result<(), MessageSendError> {
        self.sender.send(message)
    }

    pub async fn receive(&mut self) -> Result<RecvT, MessageRecvError> {
        self.receiver.receive().await
    }
}
