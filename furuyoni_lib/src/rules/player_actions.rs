use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct HandSelector(pub usize);

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum BasicAction {
    MoveForward,
    MoveBackward,
    Recover,
    Focus,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum PlayableCardSelector {
    Hand(HandSelector),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum BasicActionCost {
    Hand(HandSelector),
    Vigor,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum MainPhaseAction {
    PlayBasicAction {
        action: BasicAction,
        cost: BasicActionCost,
    },
    PlayCard(PlayableCardSelector),
    EndMainPhase,
}
