use crate::game::{get_viewable_state, GameHandle, ObserverWithPos};
use crate::game_watcher::{GameObserver, NotifyFailedError};
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::ObservePosition;
use std::sync::{Arc, Mutex};

pub(crate) struct ObservableGame {
    handle: Arc<Mutex<GameHandle>>,
}

impl ObservableGame {
    pub(super) fn new(handle: Arc<Mutex<GameHandle>>) -> Self {
        Self { handle }
    }
    pub fn add_observable(
        &self,
        position: ObservePosition,
        mut observer: impl GameObserver + Send + 'static,
    ) -> Result<(), NotifyFailedError> {
        let mut handle = self.handle.lock().unwrap();

        let state = get_viewable_state(position, &handle.state);
        observer.initialize_state(&state)?;

        handle.observers.push(ObserverWithPos {
            position,
            observer: Box::new(observer),
        });
        Ok(())
    }
}

pub(crate) fn event_filter_information(position: ObservePosition, event: &GameEvent) -> GameEvent {
    // Todo:
    event.clone()
}
