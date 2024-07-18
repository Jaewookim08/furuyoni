use furuyoni_lib::rules::{cards::Card, states::{ InvalidGameViewUpdateError, StateView }};
use bevy::prelude::*;
use furuyoni_lib::rules::PlayerPos;

mod relative_positions;
mod requests_handler;
mod spread_plugin;
mod labels_update_system;

use spread_plugin::SpreadPlugin;
use labels_update_system::update_labels;

pub(crate) use spread_plugin::Spread;
pub(crate) use labels_update_system::{ StateLabel, StateStringPicker };
pub(crate) use relative_positions::{
    CardsRelativePosition,
    PetalsRelativePosition,
    PlayerRelativePos,
};
pub(crate) use requests_handler::{ apply_event, check_game_state, initialize_board };
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum BoardError {
    #[error("Tried to do an invalid update to the game state: {0}")] InvalidUpdate(
        #[from] InvalidGameViewUpdateError,
    ),
}

pub(crate) struct BoardPlugin;

#[derive(Debug, Component)]
pub(crate) struct HandObject {
    relative_pos: PlayerRelativePos,
}

#[derive(Debug, Component)]
pub(crate) struct OpenCardObject {
    card: Card,
}

impl OpenCardObject {
    pub(crate) fn new(card: Card) -> Self {
        Self { card }
    }
}

#[derive(Debug, Component)]
pub(crate) struct CardObject {
    owner: PlayerPos,
}

impl CardObject {
    pub(crate) fn new(owner: PlayerPos) -> Self {
        Self { owner }
    }
}



impl HandObject {
    pub(crate) fn new(relative_pos: PlayerRelativePos) -> Self {
        Self { relative_pos }
    }
}

#[derive(Debug, Component)]
pub(crate) struct DeckObject {
    relative_pos: PlayerRelativePos,
}

impl DeckObject {
    pub(crate) fn new(relative_pos: PlayerRelativePos) -> Self {
        Self { relative_pos }
    }
}


#[derive(Debug, Component)]
pub(crate) struct CardInspectPosition;


#[derive(Resource)]
struct BoardState(pub StateView);

#[derive(Resource)]
struct SelfPlayerPos(pub PlayerPos);

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StateLabel>()
            .register_type::<StateStringPicker>()
            .register_type::<PlayerRelativePos>()
            .add_systems(
                PostUpdate,
                update_labels
                    .run_if(resource_exists::<BoardState>)
                    .run_if(resource_exists::<SelfPlayerPos>)
            )
            .add_plugins(SpreadPlugin);
    }
}
