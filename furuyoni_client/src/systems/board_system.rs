use crate::game_logic::{BoardState, SelfPlayerPos};
use bevy::prelude::*;
use bevy::reflect::Reflect;
use furuyoni_lib::rules::cards::CardsPosition;
use furuyoni_lib::rules::states::{PetalsPosition, StateView};
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

impl PlayerRelativePos {
    pub fn into_absolute(self, me: PlayerPos) -> PlayerPos {
        match self {
            PlayerRelativePos::Me => me,
            PlayerRelativePos::Opponent => me.other(),
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

#[derive(Debug, Copy, Clone, Reflect)]
pub enum PetalsRelativePosition {
    Distance,
    Dust,
    Aura(PlayerRelativePos),
    Flare(PlayerRelativePos),
    Life(PlayerRelativePos),
}

impl PetalsRelativePosition {
    pub fn into_absolute(self, me: PlayerPos) -> PetalsPosition {
        match self {
            PetalsRelativePosition::Distance => PetalsPosition::Distance,
            PetalsRelativePosition::Dust => PetalsPosition::Dust,
            PetalsRelativePosition::Aura(p) => PetalsPosition::Aura(p.into_absolute(me)),
            PetalsRelativePosition::Flare(p) => PetalsPosition::Flare(p.into_absolute(me)),
            PetalsRelativePosition::Life(p) => PetalsPosition::Life(p.into_absolute(me)),
        }
    }
}

#[derive(Debug, Copy, Clone, Reflect)]
pub enum CardsRelativePosition {
    Hand(PlayerRelativePos),
    Playing(PlayerRelativePos),
    Deck(PlayerRelativePos),
    Enhancements(PlayerRelativePos),
    Played(PlayerRelativePos),
    Discards(PlayerRelativePos),
}

impl CardsRelativePosition {
    pub fn into_absolute(self, me: PlayerPos) -> CardsPosition {
        match self {
            CardsRelativePosition::Hand(p) => CardsPosition::Hand(p.into_absolute(me)),
            CardsRelativePosition::Playing(p) => CardsPosition::Playing(p.into_absolute(me)),
            CardsRelativePosition::Deck(p) => CardsPosition::Deck(p.into_absolute(me)),
            CardsRelativePosition::Enhancements(p) => {
                CardsPosition::Enhancements(p.into_absolute(me))
            }
            CardsRelativePosition::Played(p) => CardsPosition::Played(p.into_absolute(me)),
            CardsRelativePosition::Discards(p) => CardsPosition::Discards(p.into_absolute(me)),
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

fn get_string(me: PlayerPos, state: &StateView, picker: &StateStringPicker) -> String {
    let get_player = |rel_pos: &PlayerRelativePos| &state.player_states[rel_pos.into_absolute(me)];

    match picker {
        StateStringPicker::Turn => state.turn.to_string(),
        StateStringPicker::Vigor(rp) => get_player(rp).vigor.to_string(),
        StateStringPicker::PetalsCount(pos) => {
            state.get_petals(pos.into_absolute(me)).count.to_string()
        }
        StateStringPicker::CardsCount(pos) => state
            .get_cards_view(pos.into_absolute(me))
            .len()
            .to_string(),
    }
}
