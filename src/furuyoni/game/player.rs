use crate::furuyoni::game;


pub trait Player {
    fn get_main_phase_action(&self, available_actions: Vec<game::MainPhaseAction>) -> game::MainPhaseAction;
    // fn synchronize(&self, game::)
}