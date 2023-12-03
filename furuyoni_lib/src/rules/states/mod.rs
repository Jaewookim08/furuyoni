use serde::{Deserialize, Serialize};

mod petals;
mod players_data;
mod state_view;

pub use petals::*;
pub use players_data::PlayersData;
pub use state_view::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Beginning,
    Main,
    End,
}
