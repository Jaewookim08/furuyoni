use crate::net::frames::WithRequestId;
use crate::net::{MessageReceiver, MessageRecvError, MessageSendError, MessageSender};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Request failed.")]
pub enum RequestError<TRequestFrame> {
    SenderError(#[from] MessageSendError<TRequestFrame>),
    ReceiverError(#[from] MessageRecvError),
    #[error("The request id sent is not matched with the response.")]
    RequestIdMismatch,
}

#[async_trait]
pub trait Requester<TRequest, TResponse> {
    type TError;

    async fn request(&mut self, request: TRequest) -> Result<TResponse, Self::TError>;
}

pub async fn request_by_messages<TSendFrame, TRequest, TResponse>(
    sender: &MessageSender<TSendFrame>,
    receiver: &mut MessageReceiver<WithRequestId<TResponse>>,
    request_to_frame: impl Fn(WithRequestId<TRequest>) -> TSendFrame,
    request: TRequest,
) -> Result<TResponse, RequestError<TSendFrame>> {
    let id = rand::random();

    sender
        .send(request_to_frame(WithRequestId::new(id, request)))
        .await?;

    let response_frame = receiver.receive().await?;

    let response = response_frame
        .try_get(id)
        .ok_or(RequestError::RequestIdMismatch)?;

    Ok(response)
}

#[derive(Error, Debug)]
#[error("Notify failed.")]
pub enum NotifyError<TSendFrame> {
    SenderError(#[from] MessageSendError<TSendFrame>),
}

#[async_trait]
pub trait Notifier<TSendFrame, TNotification, TError = NotifyError<TSendFrame>> {
    async fn notify(&mut self, data: TNotification) -> Result<(), TError>;
}

pub async fn notify_by_message<TSendFrame, TNotification, TResponse>(
    sender: &MessageSender<TSendFrame>,
    notification_to_frame: impl Fn(TNotification) -> TSendFrame,
    notification: TNotification,
) -> Result<(), NotifyError<TSendFrame>> {
    sender.send(notification_to_frame(notification)).await?;

    Ok(())
}
