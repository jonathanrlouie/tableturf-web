use crate::tableturf::board::BoardSpace;
use crate::tableturf::player::PlayerNum;

pub const ROW_LEN: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InkSpace {
    Normal,
    Special,
}

impl InkSpace {
    pub fn into_board_space(self, player_num: PlayerNum) -> BoardSpace {
        match self {
            InkSpace::Normal => BoardSpace::Ink { player_num },
            InkSpace::Special => BoardSpace::Special {
                player_num,
                is_activated: false,
            },
        }
    }
}

pub type CardSpace = Option<InkSpace>;

pub type Grid = [[CardSpace; ROW_LEN]; ROW_LEN];

#[derive(Copy, Clone, PartialEq)]
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
            special
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

#[derive(Copy, Clone, PartialEq)]
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

    pub fn spaces(&self) -> Grid {
        self.card.spaces()
    }

    pub fn special(&self) -> u32 {
        self.card.special()
    }
}
