use crate::tableturf::card::Card;
use crate::tableturf::deck::{Deck, DrawRng};
use crate::tableturf::hand::{Hand, HandIndex};
use crate::tableturf::input::Placement;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlayerNum {
    P1,
    P2,
}

impl PlayerNum {
    pub fn idx(&self) -> usize {
        match self {
            PlayerNum::P1 => 0,
            PlayerNum::P2 => 1,
        }
    }
}

pub struct Player {
    hand: Hand,
    deck: Deck,
    pub special: u32,
}

impl Player {
    // Enforce the following constraints:
    // - Every card in hand is unavailable in the deck
    pub fn new(hand: Hand, deck: Deck, special: u32) -> Option<Self> {
        let card1 = hand.get(HandIndex::new(0).unwrap());
        let card2 = hand.get(HandIndex::new(1).unwrap());
        let card3 = hand.get(HandIndex::new(2).unwrap());
        let card4 = hand.get(HandIndex::new(3).unwrap());
        if deck.get(card1).is_available
            || deck.get(card2).is_available
            || deck.get(card3).is_available
            || deck.get(card4).is_available
        {
            return None;
        }
        Some(Player {
            hand,
            deck,
            special,
        })
    }

    pub fn hand(&self) -> Hand {
        self.hand
    }

    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    pub fn get_card(&self, hand_idx: HandIndex) -> Card {
        self.deck.get(self.hand.get(hand_idx)).card()
    }

    pub fn replace_card<R: DrawRng>(&mut self, idx: HandIndex, rng: &mut R) {
        // Don't replace the card if we're out of cards, since the game is over anyway.
        if let Some(deck_idx) = self.deck.draw_card(rng) {
            self.deck.set_unavailable(deck_idx);
            self.hand.set_deck_idx(idx, deck_idx);
        }
    }

    pub fn spend_special(&mut self, placement: &Placement, hand_idx: HandIndex) {
        if placement.is_special_activated() {
            let selected_card = self.deck.get(self.hand.get(hand_idx));
            self.special -= selected_card.special();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::card::{Card, CardState};
    use crate::tableturf::deck::{DeckIndex, DrawRng};

    struct MockRng;

    impl DrawRng for MockRng {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }
    }

    #[test]
    fn test_replace_card() {
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
        let deck = Deck::new([
            unavailable_card,
            unavailable_card,
            unavailable_card,
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
        ]);
        let mut player = Player::new(
            Hand::new([
                DeckIndex::new(0).unwrap(),
                DeckIndex::new(1).unwrap(),
                DeckIndex::new(2).unwrap(),
                DeckIndex::new(3).unwrap(),
            ]),
            deck,
            0,
        )
        .unwrap();
        player.replace_card(HandIndex::new(0).unwrap(), &mut MockRng);
        let deck_idx = player.hand.get(HandIndex::new(0).unwrap());
        assert_eq!(deck_idx.get(), 4);
        assert_eq!(player.deck().get(deck_idx).is_available, false);
    }
}
