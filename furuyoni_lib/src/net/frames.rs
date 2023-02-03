use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::{PlayerPos, ViewableState};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Error, Write};
use std::string::FromUtf8Error;
use thiserror::Error;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub enum ParseError {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    InvalidMessage(InvalidMessage),
}

#[derive(Debug)]
pub struct InvalidMessage {
    pub err_str: String,
}
impl From<String> for InvalidMessage {
    fn from(str: String) -> Self {
        Self { err_str: str }
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(parse_error: serde_json::Error) -> Self {
        Self::InvalidMessage(InvalidMessage {
            err_str: parse_error.to_string(),
        })
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidMessage(InvalidMessage {
            err_str: err.to_string(),
        })
    }
}

#[derive(Error, Debug)]
#[error("Writing a frame failed.")]
pub enum WriteError {
    IOError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl From<serde_json::Error> for WriteError {
    fn from(parse_error: serde_json::Error) -> Self {
        Self::SerializationError(parse_error)
    }
}
impl From<std::io::Error> for WriteError {
    fn from(err: Error) -> Self {
        Self::IOError(err)
    }
}

#[async_trait]
pub trait OutputFrame {
    async fn write_to(
        &self,
        writer: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> Result<(), WriteError>;
}

pub trait InputFrame {
    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub trait Frame: OutputFrame + InputFrame {}
impl<T> Frame for T where T: OutputFrame + InputFrame {}

#[derive(Serialize, Deserialize, Debug)]
pub struct WithRequestId<T> {
    pub request_id: u32,
    pub data: T,
}
impl<T> WithRequestId<T> {
    pub fn new(request_id: u32, data: T) -> Self {
        Self { request_id, data }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessageFrame {
    GameMessage(GameMessageFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameMessageFrame {
    Request(GameRequest),
    Response(GameToPlayerResponseFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameRequest {
    RequestData(GameToPlayerRequestDataFrame),
    Notify(GameNotification),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameNotification {}

pub type GameToPlayerRequestDataFrame = WithRequestId<GameToPlayerRequestData>;

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerRequestData {
    RequestMainPhaseAction(RequestMainPhaseAction),
    RequestGameStart {
        state: ViewableState,
        pos: PlayerPos,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessageFrame {
    PlayerMessage(PlayerMessageFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerMessageFrame {
    Response(PlayerResponseFrame),
    Request(PlayerToGameRequestFrame),
}

pub type PlayerResponseFrame = WithRequestId<PlayerResponse>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerResponse {
    ResponseMainPhaseAction(ResponseMainPhaseAction),
    AcknowledgeGameStart,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMainPhaseAction {
    pub state: ViewableState,
    pub playable_cards: Vec<PlayableCardSelector>,
    pub performable_basic_actions: Vec<BasicAction>,
    pub available_basic_action_costs: Vec<BasicActionCost>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMainPhaseAction {
    pub action: MainPhaseAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToGameRequest {
    RequestState,
    Surrender,
}

pub type PlayerToGameRequestFrame = WithRequestId<PlayerToGameRequest>;

#[derive(Serialize, Deserialize, Debug)]
pub enum GameToPlayerResponse {
    State(ViewableState),
    Ack,
}

pub type GameToPlayerResponseFrame = WithRequestId<GameToPlayerResponse>;

#[async_trait]
impl OutputFrame for ServerMessageFrame {
    async fn write_to(
        &self,
        writer: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> Result<(), WriteError> {
        write_serialized(writer, &self).await
    }
}

impl InputFrame for ServerMessageFrame {
    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        parse::<Self>(src)
    }
}

#[async_trait]
impl OutputFrame for ClientMessageFrame {
    async fn write_to(
        &self,
        writer: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> Result<(), WriteError> {
        write_serialized(writer, &self).await
    }
}

impl InputFrame for ClientMessageFrame {
    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        parse::<Self>(src)
    }
}

fn parse<T: for<'a> Deserialize<'a>>(src: &mut Cursor<&[u8]>) -> Result<T, ParseError> {
    let line = get_line(src)?.to_vec();
    let str = String::from_utf8(line)?;
    let deserialized = serde_json::from_str::<T>(&str)?;
    Ok(deserialized)
}

async fn write_serialized(
    writer: &mut (impl AsyncWriteExt + Unpin),
    data: impl Serialize,
) -> Result<(), WriteError> {
    let serialized = serde_json::to_string(&data)?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.write_all(b"\r\n").await?;
    Ok(())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], ParseError> {
    let len = src.get_ref().len();
    if len <= 1 {
        return Err(ParseError::Incomplete);
    }

    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = len - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(ParseError::Incomplete)
}
