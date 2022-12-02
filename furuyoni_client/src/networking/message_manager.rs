use crate::networking::GameMessageHandler;
use furuyoni_lib::net::connection;
use furuyoni_lib::net::connection::Connection;
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};
use tokio::sync::oneshot;

pub type GameConnection = Connection<ClientMessageFrame, ServerMessageFrame>;

pub struct MessageManager {
    connection: GameConnection,
    game_message_handler: GameMessageHandler,
}

#[derive(Debug)]
pub enum Error {
    ConnectionReadError(connection::ReadError),
    ConnectionWriteError(connection::WriteError),
}

impl From<connection::ReadError> for Error {
    fn from(err: connection::ReadError) -> Self {
        Error::ConnectionReadError(err)
    }
}
impl From<connection::WriteError> for Error {
    fn from(err: connection::WriteError) -> Self {
        Error::ConnectionWriteError(err)
    }
}

impl MessageManager {
    pub fn new(connection: GameConnection, game_message_handler: GameMessageHandler) -> Self {
        Self {
            connection,
            game_message_handler,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            let frame = self.connection.read_frame().await?;
            let response = self.game_message_handler.handle(frame).await;
            self.connection.write_frame(&response).await?;
        }
    }
}
