mod networking;

use crate::networking::{GameMessageHandler, MessageManager};
use furuyoni_lib::players::CliPlayer;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut socket = TcpStream::connect("127.0.0.1:4255").await?;
    let (read_half, write_half) = socket.split();

    todo!();
    // let connection = ::new(socket);
    // let game_message_handler = GameMessageHandler::new(Box::new(CliPlayer {}));
    //
    // let mut message_manager = MessageManager::new(connection, game_message_handler);
    //
    // message_manager.run().await.expect("Error");

    Ok(())
}
