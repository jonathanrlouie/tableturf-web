use crate::tableturf::deck::DeckIndex;
use std::ops::{Index, IndexMut};

pub const HAND_SIZE: usize = 4;

#[derive(Copy, Clone)]
pub enum HandIndex {
    H1,
    H2,
    H3,
    H4,
}

#[derive(Debug)]
pub struct Hand([DeckIndex; HAND_SIZE]);

impl Index<HandIndex> for Hand {
    type Output = DeckIndex;
    fn index(&self, index: HandIndex) -> &Self::Output {
        match index {
            HandIndex::H1 => &self.0[0],
            HandIndex::H2 => &self.0[1],
            HandIndex::H3 => &self.0[2],
            HandIndex::H4 => &self.0[3],
        }
    }
}

impl IndexMut<HandIndex> for Hand {
    fn index_mut(&mut self, index: HandIndex) -> &mut Self::Output {
        match index {
            HandIndex::H1 => &mut self.0[0],
            HandIndex::H2 => &mut self.0[1],
            HandIndex::H3 => &mut self.0[2],
            HandIndex::H4 => &mut self.0[3],
        }
    }
}

impl Hand {
    pub fn new(hand: [DeckIndex; HAND_SIZE]) -> Self {
        Hand(hand)
    }
}
