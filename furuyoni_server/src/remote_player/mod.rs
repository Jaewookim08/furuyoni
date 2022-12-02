use crate::networking::GameConnection;
use async_trait::async_trait;
use furuyoni_lib::net::frames::{ClientMessageFrame, RequestMainPhaseAction, ServerMessageFrame};
use furuyoni_lib::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::players::Player;
use furuyoni_lib::rules::ViewableState;

pub struct RemotePlayer {
    connection: GameConnection,
}

impl RemotePlayer {
    pub fn new(connection: GameConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl Player for RemotePlayer {
    async fn get_main_phase_action(
        &mut self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        todo!()

        // let frame = ServerMessageFrame::RequestMainPhaseAction(RequestMainPhaseAction {
        //     state: state.clone(),
        //     playable_cards: playable_cards.clone(),
        //     performable_basic_actions: performable_basic_actions.clone(),
        //     available_basic_action_costs: available_basic_action_costs.clone(),
        // });
        // self.connection.write_frame(&frame).await.expect("Todo");
        // let response = self.connection.read_frame().await.expect("Todo");
        //
        // if let ClientMessageFrame::ResponseMainPhaseAction(response) = response {
        //     response.action
        // } else {
        //     todo!()
        // }
    }
}
