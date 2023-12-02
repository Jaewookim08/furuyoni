use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::states::ViewableState;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Sending message to the game observer has failed.")]
pub(crate) struct NotifyFailedError;

pub(crate) trait GameObserver {
    fn initialize_state(&mut self, _state: &ViewableState) -> Result<(), NotifyFailedError> {
        Ok(())
    }

    fn notify_event(&mut self, _event: &GameEvent) -> Result<(), NotifyFailedError> {
        Ok(())
    }
}
