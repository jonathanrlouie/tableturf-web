use serde::{Serialize, Deserialize};

pub const ROW_LEN: usize = 8;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum InkSpace {
    Normal,
    Special,
}

pub type CardSpace = Option<InkSpace>;

pub type Grid = [[CardSpace; ROW_LEN]; ROW_LEN];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Card {
    name: String,
    priority: u32,
    spaces: Grid,
    special: u32,
}

impl Card {
    pub fn new(name: String, priority: u32, spaces: Grid, special: u32) -> Self {
        Card {
            name,
            priority,
            spaces,
            special,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn spaces(&self) -> Grid {
        self.spaces
    }

    pub fn special(&self) -> u32 {
        self.special
    }
}
