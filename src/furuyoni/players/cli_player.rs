use crate::furuyoni;
use crate::furuyoni::game;
use crate::furuyoni::game::{
    BasicAction, BasicActionCost, MainPhaseAction, PlayBasicAction, PlayableCardSelector,
    ViewableState,
};
use async_trait::async_trait;

pub struct CliPlayer {}

#[async_trait]
impl furuyoni::Player for CliPlayer {
    async fn get_main_phase_action(
        &self,
        state: &ViewableState<'_>,
        playable_cards: &Vec<PlayableCardSelector>,
        doable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction {
        Self::print_state(state);

        println!("actions: {doable_basic_actions:?}");

        let index = Self::get_index_lower_than(doable_basic_actions.len() + 1).unwrap();

        if index == doable_basic_actions.len() {
            MainPhaseAction::EndMainPhase
        } else {
            MainPhaseAction::PlayBasicAction(PlayBasicAction::new(
                doable_basic_actions[index],
                BasicActionCost::Vigor,
            ))
        }
    }
}

impl CliPlayer {
    fn print_state(state: &ViewableState) {
        println!(" == state == ");
        println!("{state:?}");
        println!(" =========== ");
    }

    fn get_input<T: std::str::FromStr>() -> Result<T, std::io::Error> {
        let val = loop {
            let mut line = String::new();
            let b = std::io::stdin().read_line(&mut line)?;
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
