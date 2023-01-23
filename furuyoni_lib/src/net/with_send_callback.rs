use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Error, Debug)]
#[error("Callback returned an error")]
pub enum CallbackError<TInner> {
    Inner(#[from] TInner),
}

#[derive(Debug)]
pub struct WithCallback<TData, TError> {
    pub callback: oneshot::Sender<Result<(), CallbackError<TError>>>,
    pub data: TData,
}

impl<TData, TError> WithCallback<TData, TError> {
    pub fn new(callback: oneshot::Sender<Result<(), CallbackError<TError>>>, data: TData) -> Self {
        Self { callback, data }
    }
}
