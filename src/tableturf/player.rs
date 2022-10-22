use crate::tableturf::deck::{Deck, DeckIndex};
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
    pub hand: Hand,
    pub deck: Deck,
    pub special: u32,
}

impl Player {
    pub fn replace_card(&mut self, idx: HandIndex) {
        self.discard_card(idx);
        // Don't replace the card if we're out of cards, since the game is over anyway.
        if let Some(deck_idx) = self.draw_card() {
            self.hand.set_deck_idx(idx, deck_idx);
        }
    }

    fn discard_card(&mut self, idx: HandIndex) {
        let discarded_card_idx = self.hand.get(idx);
        self.deck.get(discarded_card_idx).is_available = false;
    }

    // TODO: Pass in the RNG for easier testing
    fn draw_card(&mut self) -> Option<DeckIndex> {
        self.deck.draw_card()
    }

    pub fn spend_special(&mut self, placement: &Placement, hand_idx: HandIndex) {
        if placement.special_activated {
            let selected_card = self.deck.get(self.hand.get(hand_idx));
            self.special -= selected_card.card.special;
        }
    }
}
