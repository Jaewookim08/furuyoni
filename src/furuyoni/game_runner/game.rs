pub struct Game {
    state: GameState,
}



impl Game {
    pub fn new() -> Self {
        Game { state: GameState::default() }
    }

    pub fn hello(&self) {
        print!("Hello!!")
    }

    pub fn run(&mut self) {}
}


