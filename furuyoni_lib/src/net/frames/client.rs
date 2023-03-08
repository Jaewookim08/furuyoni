use crate::net::frames::base::{InputFrame, OutputFrame, write_serialized, parse};
use crate::net::frames::error::{WriteError, ParseError};

use std::io::Cursor;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::io::AsyncWriteExt;

use super::PlayerToLobbyMessage;
use super::Game::PlayerToGameMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessageFrame {
    PlayerToLobbyMessage(PlayerToLobbyMessage),
    PlayerToGameMessage(PlayerToGameMessage),
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