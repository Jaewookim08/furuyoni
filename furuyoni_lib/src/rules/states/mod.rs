use serde::{Deserialize, Serialize};

pub mod petals;
mod players_data;
mod state_view;

pub use players_data::PlayersData;
pub use state_view::*;
