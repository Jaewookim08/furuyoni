use crate::furuyoni::game;


pub trait Player {
    fn get_main_phase_action(&self, state: game::ViewableState, available_actions: Vec<game::MainPhaseAction>) -> game::MainPhaseAction;
}