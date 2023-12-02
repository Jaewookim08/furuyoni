use crate::game_watcher::GameObserver;
use async_trait::async_trait;
use furuyoni_lib::rules::player_actions::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector,
};
use furuyoni_lib::rules::states::*;

pub(crate) struct CliPlayer {}

#[async_trait]
impl super::Player for CliPlayer {
    async fn get_main_phase_action(
        &mut self,
        state: &StateView,
        _playable_cards: &Vec<PlayableCardSelector>,
        performable_basic_actions: &Vec<BasicAction>,
        _available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> Result<MainPhaseAction, ()> {
        Self::print_state(&state);

        println!("actions: {performable_basic_actions:?}");

        let index = Self::get_index_lower_than(performable_basic_actions.len() + 1).unwrap();

        if index == performable_basic_actions.len() {
            Ok(MainPhaseAction::EndMainPhase)
        } else {
            Ok(MainPhaseAction::PlayBasicAction {
                action: performable_basic_actions[index],
                cost: BasicActionCost::Vigor,
            })
        }
    }
}

impl GameObserver for CliPlayer {}
impl CliPlayer {
    fn print_state(state: &StateView) {
        println!(" == state == ");
        println!("{state:?}");
        println!(" =========== ");
    }

    fn get_input<T: std::str::FromStr>() -> Result<T, std::io::Error> {
        let val = loop {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line)?;
            let line = line.trim();

            match line.parse::<T>() {
                Ok(v) => {
                    break v;
                }
                Err(_) => {
                    println!("Parse failed. Please try again:");
                }
            }
        };
        Ok(val)
    }

    fn get_index_lower_than(upper_bound: usize) -> Result<usize, std::io::Error> {
        let val = loop {
            let a = Self::get_input::<usize>()?;
            if a < upper_bound {
                break a;
            } else {
                println!("Input should be smaller than {upper_bound}");
            }
        };

        assert!(val < upper_bound);
        Ok(val)
    }
}
