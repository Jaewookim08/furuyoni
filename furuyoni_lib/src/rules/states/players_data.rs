use crate::rules::PlayerPos;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayersData<TData> {
    pub p1_data: TData,
    pub p2_data: TData,
}
impl<T> PlayersData<T> {
    pub fn new(p1_data: T, p2_data: T) -> Self {
        Self { p1_data, p2_data }
    }
}

impl<T> Index<PlayerPos> for PlayersData<T> {
    type Output = T;

    fn index(&self, index: PlayerPos) -> &Self::Output {
        match index {
            PlayerPos::P1 => &self.p1_data,
            PlayerPos::P2 => &self.p2_data,
        }
    }
}

impl<T> Index<&PlayerPos> for PlayersData<T> {
    type Output = T;

    fn index(&self, index: &PlayerPos) -> &Self::Output {
        match index {
            PlayerPos::P1 => &self.p1_data,
            PlayerPos::P2 => &self.p2_data,
        }
    }
}

impl<T> IndexMut<PlayerPos> for PlayersData<T> {
    fn index_mut(&mut self, index: PlayerPos) -> &mut Self::Output {
        match index {
            PlayerPos::P1 => &mut self.p1_data,
            PlayerPos::P2 => &mut self.p2_data,
        }
    }
}

impl<T> IndexMut<&PlayerPos> for PlayersData<T> {
    fn index_mut(&mut self, index: &PlayerPos) -> &mut Self::Output {
        match index {
            PlayerPos::P1 => &mut self.p1_data,
            PlayerPos::P2 => &mut self.p2_data,
        }
    }
}
