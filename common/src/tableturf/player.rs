use crate::tableturf::card::Card;
use crate::tableturf::deck::{Deck, DrawRng, Hand, HandIndex};
use crate::tableturf::input::Placement;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum PlayerNum {
    P1,
    P2,
}

#[derive(Debug)]
pub struct Players([Player; 2]);

impl Index<PlayerNum> for Players {
    type Output = Player;
    fn index(&self, index: PlayerNum) -> &Self::Output {
        match index {
            PlayerNum::P1 => &self.0[0],
            PlayerNum::P2 => &self.0[1],
        }
    }
}

impl IndexMut<PlayerNum> for Players {
    fn index_mut(&mut self, index: PlayerNum) -> &mut Self::Output {
        match index {
            PlayerNum::P1 => &mut self.0[0],
            PlayerNum::P2 => &mut self.0[1],
        }
    }
}

impl Players {
    pub fn new(players: [Player; 2]) -> Self {
        Players(players)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    hand: Hand,
    deck: Deck,
    player_num: PlayerNum,
    pub special: u32,
}

impl Player {
    pub fn new(hand: Hand, deck: Deck, player_num: PlayerNum, special: u32) -> Self {
        Player {
            hand,
            deck,
            player_num,
            special,
        }
    }

    pub fn player_num(&self) -> PlayerNum {
        self.player_num
    }

    pub fn hand(&self) -> &Hand {
        &self.hand
    }

    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    pub fn get_card(&self, hand_idx: HandIndex) -> &Card {
        self.deck.index(self.hand[hand_idx]).0
    }

    pub fn redraw_hand<R: DrawRng>(&mut self, rng: &mut R) {
        let (deck, hand) = Deck::draw_hand(self.deck.cards().clone(), rng);
        self.hand = hand;
        self.deck = deck;
    }

    pub fn replace_card<R: DrawRng>(&mut self, idx: HandIndex, rng: &mut R) {
        // Don't replace the card if we're out of cards, since the game is over anyway.
        if let Some(deck_idx) = self.deck.draw_card(rng) {
            self.hand[idx] = deck_idx;
        }
    }

    pub fn spend_special(&mut self, placement: &Placement, hand_idx: HandIndex) {
        if placement.is_special_activated() {
            let (card, _) = self.deck.index(self.hand[hand_idx]);
            self.special -= card.special();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::card::Card;
    use crate::tableturf::deck::{DeckIndex, DrawRng};

    struct MockRng;

    impl DrawRng for MockRng {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Hand {
            let v: Vec<DeckIndex> = iter.collect();
            Hand::new([v[0], v[1], v[2], v[3]]).unwrap()
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
        let card = Card::new("test".to_string(), 0, spaces, 0);
        let (deck, hand) = Deck::draw_hand(
            [
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card.clone(),
                card,
            ],
            &mut MockRng,
        );
        let mut player = Player::new(hand, deck, PlayerNum::P1, 0);
        player.replace_card(HandIndex::H1, &mut MockRng);
        let deck_idx = player.hand[HandIndex::H1];
        assert_eq!(deck_idx, DeckIndex::D5);
        assert_eq!(*player.deck().index(deck_idx).1, false);
    }
}
