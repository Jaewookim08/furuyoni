use crate::net::frames::base::{parse, write_serialized, InputFrame, OutputFrame};
use crate::net::frames::error::{ParseError, WriteError};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::io::AsyncWriteExt;

use super::game::PlayerToGameMessage;
use super::PlayerToLobbyMessage;

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
