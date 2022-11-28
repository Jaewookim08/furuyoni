mod parse_helper;

use crate::messages::parse_helper::InvalidMessage;
use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::ViewableState;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    InvalidMessage(InvalidMessage),
}

impl From<serde_json::Error> for Error {
    fn from(parse_error: serde_json::Error) -> Self {
        Self::InvalidMessage(InvalidMessage {
            err_str: parse_error.to_string(),
        })
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidMessage(InvalidMessage {
            err_str: err.to_string(),
        })
    }
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

fn parse<T: for<'a> Deserialize<'a>>(src: &mut Cursor<&[u8]>) -> Result<T, Error> {
    let line = parse_helper::get_line(src)?.to_vec();
    let str = String::from_utf8(line)?;
    let deserialized = serde_json::from_str::<T>(&str)?;
    Ok(deserialized)
}

impl PlayerMessageFrame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        parse::<Self>(src)
    }
}

impl GameMessageFrame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        parse::<Self>(src)
    }
}
