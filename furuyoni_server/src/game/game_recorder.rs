use crate::game::states::GameState;
use crate::game::{get_event_view, get_state_view};
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
    recorded_events: Vec<GameEvent>,
    observers: Vec<ObserverWithPos>,
}

struct ObserverWithPos {
    position: ObservePosition,
    observer: Box<dyn GameObserver + Send>,
}

impl GameRecorder {
    pub(super) fn new(initial_game_state: GameState) -> Self {
        Self {
            initial_game_state,
            inner: Mutex::new(RecorderInner {
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

        let mut state = get_state_view(position, &self.initial_game_state);

        for e in &inner.recorded_events {
            match e {
                GameEvent::StateUpdated(update) => state.apply_update(*update).expect("todo: "),
                _ => {}
            }
        }

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

pub(super) async fn run_recorder(mut rx: mpsc::Receiver<GameEvent>, recorder: Arc<GameRecorder>) {
    while let Some(event) = rx.recv().await {
        let mut inner = recorder.inner.lock().unwrap();
        inner.recorded_events.push(event);

        for ObserverWithPos { position, observer } in &mut inner.observers {
            // ignore notify errors.
            // Todo: remove observer if error occurs?
            let _ = observer.notify_event(get_event_view(*position, event));
        }
    }
}
