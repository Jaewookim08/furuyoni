use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Petals {
    pub count: u32,
    pub max: Option<u32>,
}

impl Petals {
    pub fn new(n: u32, max: Option<u32>) -> Self {
        Self { count: n, max }
    }
}
