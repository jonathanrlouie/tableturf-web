use crate::tableturf::deck::DeckIndex;

pub use self::hand_idx::{HandIndex, HAND_SIZE};

#[derive(Copy, Clone)]
pub struct Hand([DeckIndex; HAND_SIZE]);

impl Hand {
    pub fn new(hand: [DeckIndex; HAND_SIZE]) -> Self {
        Hand(hand)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_hand_index() {
        let invalid_idx = HandIndex::new(4);
        assert!(invalid_idx.is_none());

        let min_valid_idx = HandIndex::new(0);
        assert!(min_valid_idx.is_some());

        let max_valid_idx = HandIndex::new(3);
        assert!(max_valid_idx.is_some());
    }
}
