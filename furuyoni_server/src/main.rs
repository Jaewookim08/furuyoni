#[macro_use]
extern crate derive_more;

extern crate furuyoni_lib;

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

    // let listener = TcpListener::bind("127.0.0.1:4255").await.unwrap();
    // let (socket, _) = listener.accept().await.unwrap();
    // let p1 = RemotePlayer::new(todo!());

    let mut game = game::Game::new(Box::new(CliPlayer {}), Box::new(IdlePlayer {}));
    let res = futures::executor::block_on(game.run());
    let winner_str = match res.winner {
        PlayerPos::P1 => "P1",
        PlayerPos::P2 => "P2",
    };
    println!("Game ended. Winner: {winner_str}");
}
