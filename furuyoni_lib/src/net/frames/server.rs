use crate::net::frames::base::{parse, write_serialized, InputFrame, OutputFrame};
use crate::net::frames::error::{ParseError, WriteError};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::io::AsyncWriteExt;

use super::game::GameToPlayerMessage;
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
