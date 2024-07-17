use bevy::reflect::Reflect;
use furuyoni_lib::rules::cards::CardsPosition;
use furuyoni_lib::rules::states::PetalsPosition;
use furuyoni_lib::rules::PlayerPos;

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
