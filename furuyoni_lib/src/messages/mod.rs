mod parse_helper;

use crate::messages::parse_helper::InvalidMessage;
use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::ViewableState;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    InvalidMessage(InvalidMessage),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameMessageFrame {
    RequestMainPhaseAction(RequestMainPhaseAction),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerMessageFrame {
    ResponseMainPhaseAction(ResponseMainPhaseAction),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMainPhaseAction {
    state: ViewableState,
    playable_cards: Vec<PlayableCardSelector>,
    performable_basic_actions: Vec<BasicAction>,
    available_basic_action_costs: Vec<BasicActionCost>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMainPhaseAction {
    action: MainPhaseAction,
}

impl PlayerMessageFrame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        let line = parse_helper::get_line(src)?;
        todo!()
    }
}
