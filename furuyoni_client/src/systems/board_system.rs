use crate::game_logic::{BoardState, SelfPlayerPos};
use crate::ReflectComponent;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use furuyoni_lib::rules::states::StateView;
use furuyoni_lib::rules::PlayerPos;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StateLabel>()
            .register_type::<StateStringPicker>()
            .register_type::<PlayerRelativePos>()
            .add_systems(
                Update,
                display_board
                    .run_if(resource_exists::<BoardState>())
                    .run_if(resource_exists::<SelfPlayerPos>()),
            );
    }
}

fn display_board(
    state: Res<BoardState>,
    self_pos: Res<SelfPlayerPos>,
    mut query: Query<(&mut Text, &StateLabel)>,
) {
    if state.is_changed() {
        for (mut text, state_label) in &mut query {
            text.sections[state_label.text_section_index].value =
                get_string(self_pos.0, &state.0, &state_label.picker);
        }
    }
}

#[derive(Debug, Reflect, Default, Copy, Clone)]
pub enum PlayerRelativePos {
    #[default]
    Me,
    Opponent,
}

// Todo: refactor using PetalsPos.
#[derive(Debug, Reflect, Default)]
pub enum StateStringPicker {
    #[default]
    Turn,
    Distance,
    Dust,
    Aura(PlayerRelativePos),
    Flare(PlayerRelativePos),
    Life(PlayerRelativePos),
    Vigor(PlayerRelativePos),
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

fn get_string(self_pos: PlayerPos, state: &StateView, picker: &StateStringPicker) -> String {
    let get_player = |rel_pos: &PlayerRelativePos| {
        let pos = match rel_pos {
            PlayerRelativePos::Me => self_pos,
            PlayerRelativePos::Opponent => self_pos.other(),
        };
        &state.player_states[pos]
    };

    match picker {
        StateStringPicker::Turn => state.turn.to_string(),
        StateStringPicker::Vigor(rel_pos) => get_player(rel_pos).vigor.to_string(),
        StateStringPicker::Distance => state.distance.count.to_string(),
        StateStringPicker::Dust => state.dust.count.to_string(),
        StateStringPicker::Aura(rel_pos) => get_player(rel_pos).aura.count.to_string(),
        StateStringPicker::Flare(rel_pos) => get_player(rel_pos).flare.count.to_string(),
        StateStringPicker::Life(rel_pos) => get_player(rel_pos).life.count.to_string(),
    }
}
