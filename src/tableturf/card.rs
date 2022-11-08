use serde::Serialize;

pub const ROW_LEN: usize = 8;

#[derive(Serialize, Copy, Clone, Debug, PartialEq)]
pub enum InkSpace {
    Normal,
    Special,
}

pub type CardSpace = Option<InkSpace>;

pub type Grid = [[CardSpace; ROW_LEN]; ROW_LEN];

#[derive(Serialize, Copy, Clone, Debug, PartialEq)]
pub struct Card {
    priority: u32,
    spaces: Grid,
    special: u32,
}

impl Card {
    pub fn new(priority: u32, spaces: Grid, special: u32) -> Self {
        Card {
            priority,
            spaces,
            special,
        }
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

#[derive(Serialize, Copy, Clone, Debug, PartialEq)]
pub struct CardState {
    card: Card,
    pub is_available: bool,
}

impl CardState {
    pub fn new(card: Card, is_available: bool) -> Self {
        CardState { card, is_available }
    }

    pub fn card(&self) -> Card {
        self.card
    }

    pub fn priority(&self) -> u32 {
        self.card.priority()
    }

    pub fn special(&self) -> u32 {
        self.card.special()
    }
}
