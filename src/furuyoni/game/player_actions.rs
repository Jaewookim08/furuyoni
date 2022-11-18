use crate::furuyoni::game;

#[derive(Debug, PartialEq)]
pub struct HandSelector(pub usize);
// Todo: Implement Index<HandSelector> for Hands vector?

#[derive(Debug)]
pub enum PlayableCardSelector {
    Hand(HandSelector),
}

#[derive(Debug, PartialEq)]
pub enum BasicActionCost {
    Hand(HandSelector),
    Vigor(game::Vigor),
}

#[derive(Debug)]
pub struct PlayBasicAction {
    pub(super) action: game::BasicAction,
    pub(super) cost: BasicActionCost,
}

#[derive(Debug)]
pub enum MainPhaseAction {
    PlayBasicAction(PlayBasicAction),
    PlayCard(PlayableCardSelector),
    EndMainPhase,
}
