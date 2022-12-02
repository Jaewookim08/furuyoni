use furuyoni_lib::net::frames::{ClientMessageFrame, ResponseMainPhaseAction, ServerMessageFrame};
use furuyoni_lib::players::Player;

pub struct GameMessageHandler {
    player: Box<dyn Player>,
}

impl GameMessageHandler {
    pub fn new(player: Box<dyn Player>) -> Self {
        Self { player }
    }

    pub async fn handle(&mut self, message: ServerMessageFrame) -> ClientMessageFrame {
        todo!()
        // match message {
        //     ServerMessageFrame::RequestMainPhaseAction(req) => {
        //         let action = self
        //             .player
        //             .get_main_phase_action(
        //                 &req.state,
        //                 &req.playable_cards,
        //                 &req.performable_basic_actions,
        //                 &req.available_basic_action_costs,
        //             )
        //             .await;
        //         ClientMessageFrame::ResponseMainPhaseAction(ResponseMainPhaseAction { action })
        //     }
        // }
    }
}
