use crate::game::Vigor;
use furuyoni_lib::rules::cards::Cards;
use furuyoni_lib::rules::states::Petals;
use furuyoni_lib::rules::states::{CardsView, PlayerStateView};
use furuyoni_lib::rules::{ObservePosition, PlayerPos};
use std::collections::VecDeque;

#[derive(Debug)]
pub(crate) struct PlayerState {
    pub hand: Cards,
    pub deck: Cards,
    pub enhancements: Cards,
    pub played_pile: Cards,
    pub discard_pile: Cards,

    pub vigor: Vigor,
    pub aura: Petals,
    pub life: Petals,
    pub flare: Petals,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            hand: vec![],
            deck: Vec::default(),
            enhancements: vec![],
            played_pile: vec![],
            discard_pile: vec![],
            vigor: Vigor(0),
            aura: Petals::new(3, Some(5)),
            life: Petals::new(10, Some(10)),
            flare: Petals::new(0, None),
        }
    }
}

impl PlayerState {
    pub fn as_viewed_from(
        &self,
        owner: PlayerPos,
        observed_from: ObservePosition,
    ) -> PlayerStateView {
        let (can_view_personals, can_view_all) = {
            match observed_from {
                ObservePosition::RelativeTo(p) => (p == owner, false),
                ObservePosition::MasterView => (true, true),
                ObservePosition::ByStander => (false, false),
            }
        };

        PlayerStateView {
            hand: CardsView::from(&self.hand, can_view_personals),
            deck: CardsView::from(&self.deck.clone(), can_view_all),
            enhancements: self.enhancements.clone(),
            played_pile: self.played_pile.clone(),
            discard_pile: CardsView::from(&self.discard_pile, can_view_personals),
            vigor: self.vigor.0,
            aura: self.aura.clone(),
            life: self.life.clone(),
            flare: self.flare.clone(),
        }
    }
}
