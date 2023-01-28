use bevy::prelude::*;
use bevy::text::Text;
use furuyoni_lib::rules::{
    PlayerPos, ViewableOpponentState, ViewablePlayerState, ViewableSelfState, ViewableState,
};

use crate::systems::player::{GameState, SelfPlayerPos};
use bevy::prelude::Component;

pub fn display_board(
    state: Res<GameState>,
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

pub enum PlayerRelativePos {
    Me,
    Opponent,
}

pub enum StateStringPicker {
    Dust,
    Distance,
    Turn,
    PlayerValue {
        pos: PlayerRelativePos,
        picker: PlayerStringPicker,
    },
}

pub enum PlayerStringPicker {
    Life,
    Flare,
    Aura,
    Vigor,
}

#[derive(Component)]
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

fn get_string(self_pos: PlayerPos, state: &ViewableState, picker: &StateStringPicker) -> String {
    match picker {
        StateStringPicker::Dust => state.dust.to_string(),
        StateStringPicker::Distance => state.distance.to_string(),
        StateStringPicker::Turn => state.turn_number.to_string(),
        StateStringPicker::PlayerValue { pos, picker } => {
            let pos = match pos {
                PlayerRelativePos::Me => self_pos,
                PlayerRelativePos::Opponent => self_pos.other(),
            };
            let player = &state.player_states[pos];

            match player {
                ViewablePlayerState::SelfState(p) => get_string_from_self_player(p, picker),
                ViewablePlayerState::Opponent(p) => get_string_from_opponent_player(p, picker),
            }
        }
    }
}

fn get_string_from_self_player(state: &ViewableSelfState, picker: &PlayerStringPicker) -> String {
    match picker {
        PlayerStringPicker::Life => state.life.to_string(),
        PlayerStringPicker::Flare => state.flare.to_string(),
        PlayerStringPicker::Aura => state.aura.to_string(),
        PlayerStringPicker::Vigor => state.vigor.to_string(),
    }
}

fn get_string_from_opponent_player(
    state: &ViewableOpponentState,
    picker: &PlayerStringPicker,
) -> String {
    match picker {
        PlayerStringPicker::Life => state.life.to_string(),
        PlayerStringPicker::Flare => state.flare.to_string(),
        PlayerStringPicker::Aura => state.aura.to_string(),
        PlayerStringPicker::Vigor => state.vigor.to_string(),
    }
}
