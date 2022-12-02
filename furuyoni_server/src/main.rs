#[macro_use]
extern crate derive_more;

extern crate furuyoni_lib;

use crate::networking::{GameConnectionReader, GameConnectionWriter};
use furuyoni_lib::players::{CliPlayer, IdlePlayer};
use furuyoni_lib::rules::PlayerPos;
use remote_player::RemotePlayer;
use tokio::net::TcpListener;

mod game;
mod networking;
mod remote_player;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:4255").await.unwrap();
    let (mut socket, _) = listener.accept().await.unwrap();
    let (read_half, write_half) = socket.split();

    let p1 = RemotePlayer::new();

    let mut game = game::Game::new(Box::new(p1), Box::new(IdlePlayer {}));
    let res = futures::executor::block_on(game.run());
    let winner_str = match res.winner {
        PlayerPos::P1 => "P1",
        PlayerPos::P2 => "P2",
    };
    println!("Game ended. Winner: {winner_str}");
}
