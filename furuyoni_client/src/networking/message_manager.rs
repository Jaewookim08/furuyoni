use crate::networking::GameMessageHandler;
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};
use tokio::sync::oneshot;

pub type ClientConnectionWriter<TWrite> = connection::ConnectionWriter<TWrite, ClientMessageFrame>;
pub type ClientConnectionReader<TRead> = connection::ConnectionReader<TRead, ServerMessageFrame>;

pub struct MessageManager {
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
    pub fn new(game_message_handler: GameMessageHandler) -> Self {
        Self {
            game_message_handler,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            todo!()
            // let frame = self.connection.read_frame().await?;
            // let response = self.game_message_handler.handle(frame).await;
            // self.connection.write_frame(&response).await?;
        }
    }
}
