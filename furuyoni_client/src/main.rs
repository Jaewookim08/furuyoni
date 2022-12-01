mod networking;

use crate::networking::{GameConnection, GameMessageHandler, MessageManager};
use furuyoni_lib::players::CliPlayer;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:4255").await?;
    let connection = GameConnection::new(socket);
    let game_message_handler = GameMessageHandler::new(Box::new(CliPlayer {}));

    let mut message_manager = MessageManager::new(connection, game_message_handler);

    message_manager.run().await.expect("Error");

    Ok(())
}
