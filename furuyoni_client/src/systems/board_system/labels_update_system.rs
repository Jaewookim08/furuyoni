use bevy::prelude::*;
use bevy::reflect::Reflect;
use furuyoni_lib::rules::{ states::StateView, PlayerPos };

use super::{
    relative_positions::{ CardsRelativePosition, PetalsRelativePosition, PlayerRelativePos },
    BoardState,
    SelfPlayerPos,
};


pub fn update_labels(
    state: Res<BoardState>,
    self_pos: Res<SelfPlayerPos>,
    mut query: Query<(&mut Text, &StateLabel)>
) {
    if state.is_changed() {
        for (mut text, state_label) in &mut query {
            text.sections[state_label.text_section_index].value = get_string(
                self_pos.0,
                &state.0,
                &state_label.picker
            );
        }
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct StateLabel {
    text_section_index: usize,
    picker: StateStringPicker,
}

impl StateLabel {
    pub fn new(text_section_index: usize, picker: StateStringPicker) -> Self {
        Self {
            text_section_index,
            picker,
        }
    }
}

// Todo: refactor using PetalsPos.
#[derive(Debug, Copy, Clone, Reflect, Default)]
pub enum StateStringPicker {
    #[default]
    Turn,
    Vigor(PlayerRelativePos),
    PetalsCount(PetalsRelativePosition),
    CardsCount(CardsRelativePosition),
}

fn get_string(me: PlayerPos, state: &StateView, picker: &StateStringPicker) -> String {
    let get_player = |rel_pos: &PlayerRelativePos| &state.player_states[rel_pos.into_absolute(me)];

    match picker {
        StateStringPicker::Turn => state.turn.to_string(),
        StateStringPicker::Vigor(rp) => get_player(rp).vigor.to_string(),
        StateStringPicker::PetalsCount(pos) => {
            state.petals(pos.into_absolute(me)).count.to_string()
        }
        StateStringPicker::CardsCount(pos) => {
            state.cards_view(pos.into_absolute(me)).len().to_string()
        }
    }
}
