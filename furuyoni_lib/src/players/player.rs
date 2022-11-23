use crate::player_actions::{BasicAction, BasicActionCost, MainPhaseAction, PlayableCardSelector};
use crate::rules::{PlayerPos, ViewableState};
use async_trait::async_trait;
use std::ops::{Index, IndexMut};

#[async_trait]
pub trait Player {
    async fn get_main_phase_action(
        &self,
        state: &ViewableState,
        playable_cards: &Vec<PlayableCardSelector>,
        doable_basic_actions: &Vec<BasicAction>,
        available_basic_action_costs: &Vec<BasicActionCost>,
    ) -> MainPhaseAction;
}

#[derive(Debug)]
pub struct PlayerData<TData> {
    p1_data: TData,
    p2_data: TData,
}
impl<T> PlayerData<T> {
    pub fn new(p1_data: T, p2_data: T) -> Self {
        Self { p1_data, p2_data }
    }
}

impl<T> Index<PlayerPos> for PlayerData<T> {
    type Output = T;

    fn index(&self, index: PlayerPos) -> &Self::Output {
        match index {
            PlayerPos::P1 => &self.p1_data,
            PlayerPos::P2 => &self.p2_data,
        }
    }
}

impl<T> IndexMut<PlayerPos> for PlayerData<T> {
    fn index_mut(&mut self, index: PlayerPos) -> &mut Self::Output {
        match index {
            PlayerPos::P1 => &mut self.p1_data,
            PlayerPos::P2 => &mut self.p2_data,
        }
    }
}
