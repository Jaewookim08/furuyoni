#[macro_use]
extern crate derive_more;

extern crate furuyoni_lib;

use furuyoni_lib::players::{CliPlayer, IdlePlayer};
use furuyoni_lib::rules::PlayerPos;

mod game;

fn main() {
    println!("Hello, world!");
    let game = game::Game::new(Box::new(CliPlayer {}), Box::new(IdlePlayer {}));
    let res = futures::executor::block_on(game.run());
    let winner_str = match res.winner {
        PlayerPos::P1 => "P1",
        PlayerPos::P2 => "P2",
    };
    println!("Game ended. Winner: {winner_str}");
}
