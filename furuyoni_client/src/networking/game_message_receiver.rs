use furuyoni_lib::net::frames::{GameMessageFrame, PlayerMessageFrame, ResponseMainPhaseAction};
use furuyoni_lib::players::Player;

pub struct GameMessageHandler {
    player: Box<dyn Player>,
}

impl GameMessageHandler {
    pub fn new(player: Box<dyn Player>) -> Self {
        Self { player }
    }

    pub async fn handle(&mut self, message: GameMessageFrame) -> PlayerMessageFrame {
        match message {
            GameMessageFrame::RequestMainPhaseAction(req) => {
                let action = self
                    .player
                    .get_main_phase_action(
                        &req.state,
                        &req.playable_cards,
                        &req.performable_basic_actions,
                        &req.available_basic_action_costs,
                    )
                    .await;
                PlayerMessageFrame::ResponseMainPhaseAction(ResponseMainPhaseAction { action })
            }
        }
    }
}
