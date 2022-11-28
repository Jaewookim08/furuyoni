mod parse_helper;

use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::ViewableState;
use std::io::Cursor;

pub use parse_helper::Error;

pub enum GameMessageFrame {
    RequestMainPhaseAction(RequestMainPhaseAction),
}

pub enum PlayerMessageFrame {
    ResponseMainPhaseAction(ResponseMainPhaseAction),
}

pub struct RequestMainPhaseAction {
    state: ViewableState,
    playable_cards: Vec<PlayableCardSelector>,
    performable_basic_actions: Vec<BasicAction>,
    available_basic_action_costs: Vec<BasicActionCost>,
}

pub struct ResponseMainPhaseAction {
    action: MainPhaseAction,
}

impl PlayerMessageFrame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        let line = parse_helper::get_line(src)?;
        todo!()
    }
}
