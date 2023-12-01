use furuyoni_lib::rules::events::GameEvent;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Sending message to the game observer has failed.")]
pub(crate) struct NotifyFailedError;

pub(crate) trait GameObserver {
    fn notify_event(&mut self, _event: &GameEvent) -> Result<(), NotifyFailedError> {
        Ok(())
    }
}
