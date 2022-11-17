use async_trait::async_trait;
use crate::furuyoni;
use crate::furuyoni::game;
use crate::furuyoni::game::{MainPhaseAction, ViewableState};


pub struct CliPlayer {}

#[async_trait]
impl furuyoni::Player for CliPlayer {
    async fn get_main_phase_action(&self, state: &ViewableState<'_>, mut available_actions: Vec<MainPhaseAction>) -> MainPhaseAction {
        Self::print_state(state);
        println!("actions: {available_actions:?}");
        let index = Self::get_index_lower_than(available_actions.len()).unwrap();

        available_actions.swap_remove(index)
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
                Ok(v) => { break v; }
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
