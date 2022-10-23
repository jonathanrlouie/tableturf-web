pub use self::deck_idx::{DeckIndex, DECK_SIZE};

use crate::tableturf::card::CardState;

pub trait DrawRng {
    fn draw<T, I: Iterator<Item=T> + Sized>(&mut self, iter: I) -> Option<T>;
}

#[derive(Copy, Clone)]
pub struct Deck([CardState; DECK_SIZE]);

impl Deck {
    pub fn new(deck: [CardState; DECK_SIZE]) -> Self {
        Deck(deck)
    }

    pub fn get(&self, idx: DeckIndex) -> &CardState {
        &self.0[idx.get()]
    }

    pub fn set_card_state(&mut self, idx: DeckIndex, card_state: CardState) {
        self.0[idx.get()] = card_state;
    }

    pub fn set_unavailable(&mut self, idx: DeckIndex) {
        self.0[idx.get()].is_available = false;
    }

    pub fn draw_card<R: DrawRng>(&mut self, rng: &mut R) -> Option<DeckIndex> {
        let (idx, _) = rng.draw(self
            .0
            .iter()
            .enumerate()
            .filter(|(_, cs)| cs.is_available))?;
        DeckIndex::new(idx)
    }
}

mod deck_idx {
    pub const DECK_SIZE: usize = 15;

    #[derive(Copy, Clone)]
    pub struct DeckIndex(usize);

    impl DeckIndex {
        // Enforce that the deck index is in range 0..DECK_SIZE
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::card::Card;

    #[test]
    fn test_construct_deck_index() {
        let invalid_idx = DeckIndex::new(15);
        assert!(invalid_idx.is_none());

        let min_valid_idx = DeckIndex::new(0);
        assert!(min_valid_idx.is_some());

        let max_valid_idx = DeckIndex::new(14);
        assert!(max_valid_idx.is_some());
    }

    struct MockRng;

    impl DrawRng for MockRng {
        fn draw<T, I: Iterator<Item=T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }
    }

    #[test]
    fn test_draw_card() {
        let empty = None;
        let spaces = [
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
            [empty, empty, empty, empty, empty, empty, empty, empty],
        ];
        let card = Card::new(
            0,
            spaces,
            0
        );
        let available_card = CardState::new(
                card,
                true
            );
        let unavailable_card = CardState::new(
                card,
                false
            );
        let mut deck = Deck([
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
        ]);
        let idx = deck.draw_card(&mut MockRng);
        assert!(idx.is_some());
        assert_eq!(idx.unwrap().get(), 0);

        let mut unavailable_card_deck = Deck([
            unavailable_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
            available_card,
        ]);
        let idx = unavailable_card_deck.draw_card(&mut MockRng);
        assert!(idx.is_some());
        assert_eq!(idx.unwrap().get(), 1);

        let mut empty_deck = Deck([
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
            unavailable_card,
        ]);
        let no_card = empty_deck.draw_card(&mut MockRng);
        assert!(no_card.is_none());
    }
}
