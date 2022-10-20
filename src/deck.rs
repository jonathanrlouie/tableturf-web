use rand::prelude::IteratorRandom;
use rand::thread_rng;
pub use self::deck_idx::{DECK_SIZE, DeckIndex};

use crate::card::CardState;

pub struct Deck(pub [CardState; DECK_SIZE]);

impl Deck {
    pub fn get(&self, idx: DeckIndex) -> CardState {
        self.0[idx.get()]
    }

    pub fn set_card_state(&mut self, idx: DeckIndex, card_state: CardState) {
        self.0[idx.get()] = card_state;
    }

    // TODO: Pass in the RNG for easier testing
    pub fn draw_card(&mut self) -> Option<DeckIndex> {
        let mut rng = thread_rng();
        let (idx, _) = self.0
            .iter()
            .filter(|cs| cs.is_available)
            .enumerate()
            .choose(&mut rng)?;
        DeckIndex::new(idx)
    }
}

mod deck_idx {
    pub const DECK_SIZE: usize = 15;

    #[derive(Copy, Clone)]
    pub struct DeckIndex(usize);

    impl DeckIndex {
        pub fn new(idx: usize) -> Option<Self> {
            if idx < DECK_SIZE {
                Some(DeckIndex(idx))
            } else {
                None
            }
        }

        pub fn get(&self) -> usize {
            self.0
        }
    }
}
