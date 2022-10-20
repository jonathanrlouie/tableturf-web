use crate::tableturf::deck::DeckIndex;

pub use self::hand_idx::{HAND_SIZE, HandIndex};

pub struct Hand(pub [DeckIndex; HAND_SIZE]);

impl Hand {
    pub fn get(&self, idx: HandIndex) -> DeckIndex {
        self.0[idx.get()]
    }

    pub fn set_deck_idx(&mut self, hand_idx: HandIndex, deck_idx: DeckIndex) {
        self.0[hand_idx.get()] = deck_idx;
    }
}

mod hand_idx {
    pub const HAND_SIZE: usize = 4;

    #[derive(Copy, Clone)]
    pub struct HandIndex(usize);

    impl HandIndex {
        pub fn new(idx: usize) -> Option<Self> {
            if idx < HAND_SIZE {
                Some(HandIndex(idx))
            } else {
                None
            }
        }

        pub fn get(&self) -> usize {
            self.0
        }
    }
}
