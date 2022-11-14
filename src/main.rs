mod furuyoni;

fn main() {
    println!("Hello, world!");
    let mut a = furuyoni::game_runner::Game::new();
    let res = futures::executor::block_on(
        a.run()
    );
    println!("Game ended.");
}
