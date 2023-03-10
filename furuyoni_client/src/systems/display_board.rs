use bevy::prelude::*;
use bevy::text::Text;
use furuyoni_lib::rules::{
    PlayerPos, ViewableOpponentState, ViewablePlayerState, ViewableSelfState, ViewableState,
};

use crate::systems::player::{GameState, SelfPlayerPos};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StateLabel>()
            .register_type::<StateStringPicker>()
            .register_type::<PlayerValuePicker>()
            .register_type::<PlayerRelativePos>()
            .register_type::<PlayerValuePickerType>()
            .add_system(
                display_board
                    .run_if(resource_exists::<GameState>())
                    .run_if(resource_exists::<SelfPlayerPos>()),
            );
    }
}

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

#[derive(Debug, Reflect, FromReflect, Default)]
pub enum PlayerRelativePos {
    #[default]
    Me,
    Opponent,
}

#[derive(Debug, Reflect, FromReflect, Default)]
#[reflect(Default)]
pub struct PlayerValuePicker {
    pos: PlayerRelativePos,
    value_type: PlayerValuePickerType,
}

impl PlayerValuePicker {
    pub fn new(pos: PlayerRelativePos, value_type: PlayerValuePickerType) -> Self {
        Self { pos, value_type }
    }
}

#[derive(Debug, Reflect)]
pub enum StateStringPicker {
    Dust,
    Distance,
    Turn,
    PlayerValue(PlayerValuePicker),
}

#[derive(Debug, Reflect, FromReflect, Default)]
pub enum PlayerValuePickerType {
    #[default]
    Life,
    Flare,
    Aura,
    Vigor,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct StateLabel {
    text_section_index: usize,
    picker: StateStringPicker,
}

impl Default for StateLabel {
    fn default() -> Self {
        StateLabel {
            text_section_index: 0,
            picker: StateStringPicker::Dust,
        }
    }
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
        StateStringPicker::PlayerValue(PlayerValuePicker { pos, value_type }) => {
            let pos = match pos {
                PlayerRelativePos::Me => self_pos,
                PlayerRelativePos::Opponent => self_pos.other(),
            };
            let player = &state.player_states[pos];

            match player {
                ViewablePlayerState::SelfState(p) => get_string_from_self_player(p, value_type),
                ViewablePlayerState::Opponent(p) => get_string_from_opponent_player(p, value_type),
            }
        }
    }
}

fn get_string_from_self_player(
    state: &ViewableSelfState,
    picker: &PlayerValuePickerType,
) -> String {
    match picker {
        PlayerValuePickerType::Life => state.life.to_string(),
        PlayerValuePickerType::Flare => state.flare.to_string(),
        PlayerValuePickerType::Aura => state.aura.to_string(),
        PlayerValuePickerType::Vigor => state.vigor.to_string(),
    }
}

fn get_string_from_opponent_player(
    state: &ViewableOpponentState,
    picker: &PlayerValuePickerType,
) -> String {
    match picker {
        PlayerValuePickerType::Life => state.life.to_string(),
        PlayerValuePickerType::Flare => state.flare.to_string(),
        PlayerValuePickerType::Aura => state.aura.to_string(),
        PlayerValuePickerType::Vigor => state.vigor.to_string(),
    }
}
