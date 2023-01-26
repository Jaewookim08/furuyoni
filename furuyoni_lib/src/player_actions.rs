use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct HandSelector(pub usize);
// Todo: Implement Index<HandSelector> for Hands vector?

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

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayBasicAction {
    pub action: BasicAction,
    pub cost: BasicActionCost,
}
impl PlayBasicAction {
    pub fn new(action: BasicAction, cost: BasicActionCost) -> Self {
        Self { action, cost }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MainPhaseAction {
    PlayBasicAction(PlayBasicAction),
    PlayCard(PlayableCardSelector),
    EndMainPhase,
}
