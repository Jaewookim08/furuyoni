use crate::game::states::GameState;
use crate::game::{filter_event, filter_state, GameError};
use crate::game_watcher::{GameObserver, NotifyFailedError};
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::ObservePosition;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub(crate) struct GameRecorder {
    initial_game_state: GameState,
    inner: Mutex<RecorderInner>,
}

#[derive(Debug)]
pub(crate) struct RecordedGame {
    pub initial_game_state: GameState,
    pub recorded_events: Vec<GameEvent>,
}

struct RecorderInner {
    current_state: GameState,
    recorded_events: Vec<GameEvent>,
    observers: Vec<ObserverWithPos>,
}

struct ObserverWithPos {
    position: ObservePosition,
    observer: Box<dyn GameObserver + Send>,
}

impl GameRecorder {
    pub(super) fn new(initial_game_state: GameState) -> Self {
        let current_state = initial_game_state.clone();
        Self {
            initial_game_state,
            inner: Mutex::new(RecorderInner {
                current_state,
                recorded_events: vec![],
                observers: vec![],
            }),
        }
    }
    pub fn add_observer(
        &self,
        position: ObservePosition,
        mut observer: impl GameObserver + Send + 'static,
    ) -> Result<(), NotifyFailedError> {
        let mut inner = self.inner.lock().unwrap();

        let state = filter_state(position, &inner.current_state);

        observer.initialize_state(&state)?;

        inner.observers.push(ObserverWithPos {
            position,
            observer: Box::new(observer),
        });
        Ok(())
    }

    pub fn into_recorded_game(self) -> RecordedGame {
        RecordedGame {
            initial_game_state: self.initial_game_state,
            recorded_events: self.inner.into_inner().unwrap().recorded_events,
        }
    }
}

pub(super) async fn run_recorder(
    mut rx: mpsc::Receiver<GameEvent>,
    recorder: Arc<GameRecorder>,
) -> Result<(), GameError> {
    while let Some(event) = rx.recv().await {
        let RecorderInner {
            current_state,
            recorded_events,
            observers,
        } = &mut *recorder.inner.lock().unwrap();

        recorded_events.push(event);

        for ObserverWithPos { position, observer } in observers {
            // ignore notify errors.
            // Todo: remove observer if error occurs?
            let _ = observer.notify_event(filter_event(&current_state, *position, event)?);
        }

        match event {
            GameEvent::StateUpdated(update) => current_state.apply_update(update)?,
            _ => {}
        }
    }
    Ok(())
}
