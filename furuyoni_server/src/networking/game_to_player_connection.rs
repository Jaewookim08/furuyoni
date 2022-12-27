use async_trait::async_trait;
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameRequest, GameRequestFrame, PlayerResponse, PlayerResponseFrame,
};
use furuyoni_lib::net::{
    request_by_messages, MessageReceiver, MessageRecvError, MessageSendError, MessageSender,
    RequestError, Requester,
};

pub struct GameToPlayerConnection {
    request_sender: MessageSender<GameMessageFrame>,
    response_receiver: MessageReceiver<PlayerResponseFrame>,
}

#[async_trait]
impl Requester<GameRequest, PlayerResponse> for GameToPlayerConnection {
    type TError = RequestError<GameMessageFrame>;

    async fn request(&mut self, request: GameRequest) -> Result<PlayerResponse, Self::TError> {
        request_by_messages(
            &self.request_sender,
            &mut self.response_receiver,
            |req| GameMessageFrame::Request(req),
            request,
        )
        .await
    }
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
}
