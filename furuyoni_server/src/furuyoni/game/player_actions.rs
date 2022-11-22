use crate::furuyoni::game;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HandSelector(pub usize);
// Todo: Implement Index<HandSelector> for Hands vector?

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlayableCardSelector {
    Hand(HandSelector),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BasicActionCost {
    Hand(HandSelector),
    Vigor,
}

#[derive(Debug)]
pub struct PlayBasicAction {
    pub(super) action: game::BasicAction,
    pub(super) cost: BasicActionCost,
}
impl PlayBasicAction {
    pub fn new(action: game::BasicAction, cost: BasicActionCost) -> Self {
        Self { action, cost }
    }
}

#[derive(Debug)]
pub enum MainPhaseAction {
    PlayBasicAction(PlayBasicAction),
    PlayCard(PlayableCardSelector),
    EndMainPhase,
}
