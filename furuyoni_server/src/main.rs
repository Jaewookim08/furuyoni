extern crate furuyoni_lib;

use crate::networking::{GameToPlayerConnection, ServerConnectionReader, ServerConnectionWriter};
use std::sync::Arc;

use furuyoni_lib::net::frames::{GameMessageFrame, PlayerResponseFrame};
use furuyoni_lib::net::{MessageReceiver, MessageSender};
use furuyoni_lib::players::{CliPlayer, IdlePlayer};
use furuyoni_lib::rules::PlayerPos;
use networking::post_office;
use remote_player::RemotePlayer;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

mod game;
mod networking;
mod remote_player;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:4255").await.unwrap();
    let (socket, _) = listener.accept().await.unwrap();

    let (player_message_receiver, game_message_sender, post_office_task) =
        spawn_post_office(socket);

    let sender = Arc::new(game_message_sender);
    let connection = GameToPlayerConnection::new(sender.clone(), player_message_receiver);

    let p1 = RemotePlayer::new(connection);

    let mut game = game::Game::new(Box::new(p1), Box::new(IdlePlayer {}));
    let res = futures::executor::block_on(game.run());
    let winner_str = match res.winner {
        PlayerPos::P1 => "P1",
        PlayerPos::P2 => "P2",
    };
    println!("Game ended. Winner: {winner_str}");

    post_office_task.abort();
}

fn spawn_post_office<'a>(
    mut stream: TcpStream,
) -> (
    MessageReceiver<PlayerResponseFrame>,
    MessageSender<GameMessageFrame>,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ServerConnectionReader::new(read_half);
    let writer = ServerConnectionWriter::new(write_half);

    let (player_message_tx, player_message_rx) = tokio::sync::mpsc::channel(20);
    let (game_message_tx, game_message_rx) = tokio::sync::mpsc::channel(20);

    let post_office_joinhandle = tokio::spawn(async {
        tokio::join!(
            post_office::receive_posts(reader, player_message_tx),
            post_office::handle_send_requests(game_message_rx, writer),
        );
        ()
    });

    let player_message_receiver = MessageReceiver::new(player_message_rx);
    let game_message_sender = MessageSender::new(game_message_tx);

    return (
        player_message_receiver,
        game_message_sender,
        post_office_joinhandle,
    );
}
