use std::ops::{Index, IndexMut};

use crate::tableturf::card::CardState;

pub const DECK_SIZE: usize = 15;

pub trait DrawRng {
    fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, iter: I) -> Option<T>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeckIndex {
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
    D12,
    D13,
    D14,
    D15,
}

pub fn parse_idx(index: usize) -> Option<DeckIndex> {
    use DeckIndex::*;
    match index {
        0 => Some(D1),
        1 => Some(D2),
        2 => Some(D3),
        3 => Some(D4),
        4 => Some(D5),
        5 => Some(D6),
        6 => Some(D7),
        7 => Some(D8),
        8 => Some(D9),
        9 => Some(D10),
        10 => Some(D11),
        11 => Some(D12),
        12 => Some(D13),
        13 => Some(D14),
        14 => Some(D15),
        _ => None,
    }
}

#[derive(Copy, Clone)]
pub struct Deck([CardState; DECK_SIZE]);

impl Index<DeckIndex> for Deck {
    type Output = CardState;
    fn index(&self, index: DeckIndex) -> &Self::Output {
        match index {
            DeckIndex::D1 => &self.0[0],
            DeckIndex::D2 => &self.0[1],
            DeckIndex::D3 => &self.0[2],
            DeckIndex::D4 => &self.0[3],
            DeckIndex::D5 => &self.0[4],
            DeckIndex::D6 => &self.0[5],
            DeckIndex::D7 => &self.0[6],
            DeckIndex::D8 => &self.0[7],
            DeckIndex::D9 => &self.0[8],
            DeckIndex::D10 => &self.0[9],
            DeckIndex::D11 => &self.0[10],
            DeckIndex::D12 => &self.0[11],
            DeckIndex::D13 => &self.0[12],
            DeckIndex::D14 => &self.0[13],
            DeckIndex::D15 => &self.0[14],
        }
    }
}

impl IndexMut<DeckIndex> for Deck {
    fn index_mut(&mut self, index: DeckIndex) -> &mut Self::Output {
        match index {
            DeckIndex::D1 => &mut self.0[0],
            DeckIndex::D2 => &mut self.0[1],
            DeckIndex::D3 => &mut self.0[2],
            DeckIndex::D4 => &mut self.0[3],
            DeckIndex::D5 => &mut self.0[4],
            DeckIndex::D6 => &mut self.0[5],
            DeckIndex::D7 => &mut self.0[6],
            DeckIndex::D8 => &mut self.0[7],
            DeckIndex::D9 => &mut self.0[8],
            DeckIndex::D10 => &mut self.0[9],
            DeckIndex::D11 => &mut self.0[10],
            DeckIndex::D12 => &mut self.0[11],
            DeckIndex::D13 => &mut self.0[12],
            DeckIndex::D14 => &mut self.0[13],
            DeckIndex::D15 => &mut self.0[14],
        }
    }
}

impl Deck {
    pub fn new(deck: [CardState; DECK_SIZE]) -> Self {
        Deck(deck)
    }

    pub fn set_unavailable(&mut self, idx: DeckIndex) {
        self[idx].is_available = false;
    }

    pub fn draw_card<R: DrawRng>(&mut self, rng: &mut R) -> Option<DeckIndex> {
        let (idx, _) = rng.draw(self.0.iter().enumerate().filter(|(_, cs)| cs.is_available))?;
        parse_idx(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::card::Card;

    struct MockRng;

    impl DrawRng for MockRng {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
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
        let card = Card::new(0, spaces, 0);
        let available_card = CardState::new(card, true);
        let unavailable_card = CardState::new(card, false);
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
        assert_eq!(idx.unwrap(), DeckIndex::D1);

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
        assert_eq!(idx.unwrap(), DeckIndex::D2);

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
