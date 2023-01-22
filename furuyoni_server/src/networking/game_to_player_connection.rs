use async_trait::async_trait;
use furuyoni_lib::net::frames::{
    GameMessageFrame, GameRequest, GameToPlayerRequestData, PlayerResponse, PlayerResponseFrame,
};
use furuyoni_lib::net::{
    request_by_messages, MessageReceiver, MessageRecvError, MessageSendError, MessageSender,
    RequestError, Requester,
};
use std::sync::Arc;

/// Note: This struct receives [Arc<MessageSender<GameMessageFrame>>] instead of a
/// MessageSender<GameRequest>. This is to guarantee ordering between the response and request of
/// the game, since they both can contain data of a same state.
pub struct GameToPlayerRequester {
    message_sender: Arc<MessageSender<GameMessageFrame>>,
    response_receiver: MessageReceiver<PlayerResponseFrame>,
}

#[async_trait]
impl Requester<GameToPlayerRequestData, PlayerResponse> for GameToPlayerRequester {
    type TError = RequestError<GameMessageFrame>;

    async fn request(
        &mut self,
        request: GameToPlayerRequestData,
    ) -> Result<PlayerResponse, Self::TError> {
        request_by_messages(
            &self.message_sender,
            &mut self.response_receiver,
            |req| GameMessageFrame::Request(GameRequest::RequestData(req)),
            request,
        )
        .await
    }
}

impl GameToPlayerRequester {
    pub fn new(
        request_sender: Arc<MessageSender<GameMessageFrame>>,
        response_receiver: MessageReceiver<PlayerResponseFrame>,
    ) -> Self {
        Self {
            message_sender: request_sender,
            response_receiver,
        }
    }
}
