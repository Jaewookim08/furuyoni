use crate::net::connection;
use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum SendError {
    WriteError(#[from] connection::WriteError),
}

#[derive(Debug)]
pub struct WithSendCallback<T> {
    pub callback: oneshot::Sender<Result<(), SendError>>,
    pub data: T,
}

impl<T> WithSendCallback<T> {
    pub fn new(callback: oneshot::Sender<Result<(), SendError>>, data: T) -> Self {
        Self { callback, data }
    }
}
