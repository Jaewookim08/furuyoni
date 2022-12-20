use crate::networking::{MessageReceiver, MessageRecvError, MessageSendError, MessageSender};
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameRequest, GameRequestFrame, PlayerResponse, PlayerResponseFrame,
};
use thiserror::Error;

pub struct GameToPlayerConnection {
    request_sender: MessageSender<GameMessageFrame>,
    response_receiver: MessageReceiver<PlayerResponseFrame>,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum Error {
    SenderError(#[from] MessageSendError<GameMessageFrame>),
    ReceiverError(#[from] MessageRecvError),
    #[error("The request id sent is not matched with the response.")]
    RequestIdMismatch,
}

impl GameToPlayerConnection {
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
        let id = rand::random();

        self.request_sender
            .send(GameMessageFrame::Request(GameRequestFrame {
                id,
                data: request,
            }))
            .await?;

        let response = self.response_receiver.receive().await?;

        if response.responding_request_id != id {
            Err(Error::RequestIdMismatch)
        } else {
            Ok(response.data)
        }
    }
}
