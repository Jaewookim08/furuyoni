use crate::furuyoni;
use crate::furuyoni::game::{MainPhaseAction, ViewableState};


pub struct IdlePlayer {}

impl furuyoni::Player for IdlePlayer {
    fn get_main_phase_action(&self, state: &ViewableState, available_actions: &Vec<MainPhaseAction>) -> MainPhaseAction {
        MainPhaseAction::EndMainPhase
    }
}