use crate::networking::game_player_connection;
use furuyoni_lib::net::frames::{GameMessageFrame, GameRequest, GameRequestFrame, PlayerResponse};
use thiserror::Error;

pub struct GameCommunicationManager {
    sender: game_player_connection::Sender,
    receiver: game_player_connection::Receiver,
}

#[derive(Error, Debug)]
#[error("Send failed.")]
pub enum Error {
    SenderError(#[from] game_player_connection::SendError),
    ReceiverError(#[from] game_player_connection::RecvError),
}

impl GameCommunicationManager {
    pub fn new(
        sender: game_player_connection::Sender,
        receiver: game_player_connection::Receiver,
    ) -> Self {
        Self { sender, receiver }
    }

    pub async fn request(&mut self, request: GameRequest) -> Result<PlayerResponse, Error> {
        let id = 0; // Todo: random

        self.sender
            .send(GameMessageFrame::Request(GameRequestFrame {
                id,
                data: request,
            }))
            .await?;

        // Todo:
        let response = self.receiver.receive().await?;

        Ok(response.data)
    }
}
