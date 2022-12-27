use furuyoni_lib::net::frames::{
    GameMessageFrame, GameRequest, GameRequestFrame, PlayerResponse, PlayerResponseFrame,
};
use furuyoni_lib::net::{MessageReceiver, MessageRecvError, MessageSendError, MessageSender};
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
            .send(GameMessageFrame::Request(GameRequestFrame::new(
                id, request,
            )))
            .await?;

        let response_frame = self.response_receiver.receive().await?;

        let response = response_frame.try_get(id).ok_or(Error::RequestIdMismatch)?;

        Ok(response)
    }
}
