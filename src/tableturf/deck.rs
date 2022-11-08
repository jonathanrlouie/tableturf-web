use crate::tableturf::card::{Card, CardState};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use serde::Serialize;


pub const HAND_SIZE: usize = 4;

#[derive(Copy, Clone)]
pub enum HandIndex {
    H1,
    H2,
    H3,
    H4,
}

#[derive(Serialize, Debug)]
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
    fn new(hand: [DeckIndex; HAND_SIZE]) -> Self {
        Hand(hand)
    }
}

pub const DECK_SIZE: usize = 15;

pub trait DrawRng {
    fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, iter: I) -> Option<T>;
    fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Vec<DeckIndex>;
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

pub fn idx_to_usize(index: DeckIndex) -> usize {
    use DeckIndex::*;
    match index {
        D1 => 0,
        D2 => 1,
        D3 => 2,
        D4 => 3,
        D5 => 4,
        D6 => 5,
        D7 => 6,
        D8 => 7,
        D9 => 8,
        D10 => 9,
        D11 => 10,
        D12 => 11,
        D13 => 12,
        D14 => 13,
        D15 => 14,
    }
}

#[derive(Serialize, Copy, Clone, Debug)]
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

impl Deck {
    pub fn draw_hand<R: DrawRng>(deck: [Card; DECK_SIZE], rng: &mut R) -> Option<(Self, Hand)> {
        use DeckIndex::*;
        let indices = vec![
            D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15,
        ];
        let hand = rng.draw_hand(indices.into_iter());
        let deduped: HashSet<DeckIndex> = HashSet::from_iter(hand.clone().into_iter());
        if deduped.len() != 4 {
            return None;
        }
        let mut deck = [
            CardState::new(deck[0], true),
            CardState::new(deck[1], true),
            CardState::new(deck[2], true),
            CardState::new(deck[3], true),
            CardState::new(deck[4], true),
            CardState::new(deck[5], true),
            CardState::new(deck[6], true),
            CardState::new(deck[7], true),
            CardState::new(deck[8], true),
            CardState::new(deck[9], true),
            CardState::new(deck[10], true),
            CardState::new(deck[11], true),
            CardState::new(deck[12], true),
            CardState::new(deck[13], true),
            CardState::new(deck[14], true),
        ];
        deck[idx_to_usize(hand[0])].is_available = false;
        deck[idx_to_usize(hand[1])].is_available = false;
        deck[idx_to_usize(hand[2])].is_available = false;
        deck[idx_to_usize(hand[3])].is_available = false;
        Some((Deck(deck), Hand::new([hand[0], hand[1], hand[2], hand[3]])))
    }

    pub fn draw_card<R: DrawRng>(&mut self, rng: &mut R) -> Option<DeckIndex> {
        let (idx, _) = rng.draw(self.0.iter().enumerate().filter(|(_, cs)| cs.is_available))?;
        self.0[idx].is_available = false;
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

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Vec<DeckIndex> {
            let v: Vec<DeckIndex> = iter.collect();
            vec![v[0], v[1], v[2], v[3]]
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
