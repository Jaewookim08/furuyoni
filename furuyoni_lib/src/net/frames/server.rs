use crate::net::frames::base::{InputFrame, OutputFrame, write_serialized, parse};
use crate::net::frames::error::{WriteError, ParseError};

use std::io::Cursor;
use serde::{Serialize, Deserialize};
use tokio::io::AsyncWriteExt;
use async_trait::async_trait;

use super::Game::GameToPlayerMessage;
use super::lobby::LobbyToPlayerMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessageFrame {
    LobbyMessage(LobbyToPlayerMessage),
    GameMessage(GameToPlayerMessage),
}

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