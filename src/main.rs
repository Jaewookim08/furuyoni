mod furuyoni;

fn main() {
    println!("Hello, world!");
    let mut a = furuyoni::game::Game::new();
    let res = futures::executor::block_on(
        a.run()
    );
    let winner_str = match res.winner {
        furuyoni::game::PlayerPos::P1 => { "P1" }
        furuyoni::game::PlayerPos::P2 => { "P2" }
    };
    println!("Game ended. Winner: {winner_str}");
}
