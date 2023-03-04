
use crate::net::message_sender::{MessageSendError};
use crate::net::{MessageRecvError};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Request failed.")]
pub enum RequestError {
    SenderError(#[from] MessageSendError),
    ReceiverError(#[from] MessageRecvError),
    #[error("The request id sent is not matched with the response.")]
    RequestIdMismatch,
}

#[async_trait]
pub trait Requester<Request> {
    type Response;
    type Error: std::error::Error;

    async fn request(&mut self, request: Request) -> Result<Self::Response, Self::Error>;
}

#[derive(Error, Debug)]
#[error("Notify failed.")]
pub enum NotifyError {
    SenderError(#[from] MessageSendError),
}

#[async_trait]
pub trait Notifier<TNotification> {
    type Error: std::error::Error;

    async fn notify(&mut self, data: TNotification) -> Result<(), Self::Error>;
}
