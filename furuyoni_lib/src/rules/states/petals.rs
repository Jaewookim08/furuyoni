use crate::rules::PlayerPos;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Petals {
    pub count: u32,
    // Todo: Consider removing max.
    pub max: Option<u32>,
}

impl Petals {
    pub fn new(n: u32, max: Option<u32>) -> Self {
        Self { count: n, max }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub enum PetalsPosition {
    Distance,
    Dust,
    Aura(PlayerPos),
    Flare(PlayerPos),
    Life(PlayerPos),
    // Todo: 부여패.
}
