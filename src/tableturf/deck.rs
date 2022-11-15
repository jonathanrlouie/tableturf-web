use crate::tableturf::card::Card;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

pub const HAND_SIZE: usize = 4;

#[derive(Deserialize, Copy, Clone, Debug)]
pub enum HandIndex {
    H1,
    H2,
    H3,
    H4,
}

#[derive(Serialize, Clone, Debug)]
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
pub struct Deck {
    cards: [Card; DECK_SIZE],
    // true means the card can be drawn, false means it cannot be drawn
    card_states: [bool; DECK_SIZE],
}

impl Deck {
    pub fn draw_hand<R: DrawRng>(
        cards: [Card; DECK_SIZE],
        rng: &mut R,
    ) -> Result<(Self, Hand), String> {
        use DeckIndex::*;
        let indices = vec![
            D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D13, D14, D15,
        ];
        let hand = rng.draw_hand(indices.into_iter());
        let deduped: HashSet<DeckIndex> = HashSet::from_iter(hand.clone().into_iter());
        if deduped.len() != 4 {
            return Err("Drawn hand did not contain 4 cards".to_string());
        }
        let mut card_states = [
            true, true, true, true, true, true, true, true, true, true, true, true, true, true,
            true,
        ];
        card_states[idx_to_usize(hand[0])] = false;
        card_states[idx_to_usize(hand[1])] = false;
        card_states[idx_to_usize(hand[2])] = false;
        card_states[idx_to_usize(hand[3])] = false;
        Ok((
            Deck { cards, card_states },
            Hand::new([hand[0], hand[1], hand[2], hand[3]]),
        ))
    }

    pub fn index(&self, index: DeckIndex) -> (&Card, &bool) {
        match index {
            DeckIndex::D1 => (&self.cards[0], &self.card_states[0]),
            DeckIndex::D2 => (&self.cards[1], &self.card_states[1]),
            DeckIndex::D3 => (&self.cards[2], &self.card_states[2]),
            DeckIndex::D4 => (&self.cards[3], &self.card_states[3]),
            DeckIndex::D5 => (&self.cards[4], &self.card_states[4]),
            DeckIndex::D6 => (&self.cards[5], &self.card_states[5]),
            DeckIndex::D7 => (&self.cards[6], &self.card_states[6]),
            DeckIndex::D8 => (&self.cards[7], &self.card_states[7]),
            DeckIndex::D9 => (&self.cards[8], &self.card_states[8]),
            DeckIndex::D10 => (&self.cards[9], &self.card_states[9]),
            DeckIndex::D11 => (&self.cards[10], &self.card_states[10]),
            DeckIndex::D12 => (&self.cards[11], &self.card_states[11]),
            DeckIndex::D13 => (&self.cards[12], &self.card_states[12]),
            DeckIndex::D14 => (&self.cards[13], &self.card_states[13]),
            DeckIndex::D15 => (&self.cards[14], &self.card_states[14]),
        }
    }

    pub fn cards(self) -> [Card; DECK_SIZE] {
        self.cards
    }

    pub fn draw_card<R: DrawRng>(&mut self, rng: &mut R) -> Option<DeckIndex> {
        let (idx, _) = rng.draw(self.card_states.iter().enumerate().filter(|(_, cs)| **cs))?;
        self.card_states[idx] = false;
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
