use crate::networking::{MessageReceiver, MessageRecvError, MessageSendError, MessageSender};
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameRequest, GameRequestFrame, PlayerResponse, PlayerResponseFrame,
};
use thiserror::Error;

pub struct GameCommunicationManager {
    request_sender: MessageSender<GameMessageFrame>,
    response_receiver: MessageReceiver<PlayerResponseFrame>,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum Error {
    SenderError(#[from] MessageSendError<GameMessageFrame>),
    ReceiverError(#[from] MessageRecvError),
}

impl GameCommunicationManager {
    pub fn new(
        request_sender: MessageSender<GameMessageFrame>,
        response_receiver: MessageReceiver<PlayerResponseFrame>,
    ) -> Self {
        Self {
            request_sender,
            response_receiver,
        }
    }

    pub async fn request(&mut self, request: GameRequest) -> Result<PlayerResponse, Error> {
        let id = 0; // Todo: random

        self.request_sender
            .send(GameMessageFrame::Request(GameRequestFrame {
                id,
                data: request,
            }))
            .await?;

        // Todo:
        let response = self.response_receiver.receive().await?;

        Ok(response.data)
    }
}
