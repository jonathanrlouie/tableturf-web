use crate::board::BoardSpace;
use crate::player::PlayerNum;

pub const ROW_LEN: usize = 8;

#[derive(Copy, Clone, PartialEq)]
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
    pub priority: u32,
    pub spaces: Grid,
    pub special: u32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct CardState {
    pub card: Card,
    pub is_available: bool,
}
