use rand::prelude::IteratorRandom;
use rand::thread_rng;

pub const ROW_LEN: usize = 8;
pub const HAND_SIZE: usize = 4;
pub const DECK_SIZE: usize = 15;

#[derive(Copy, Clone, PartialEq)]
pub enum InkSpace {
    Normal,
    Special,
}

pub type CardSpace = Option<InkSpace>;

pub type Grid = [[CardSpace; ROW_LEN]; ROW_LEN];

#[derive(Copy, Clone, PartialEq)]
pub struct Card {
    pub priority: u32,
    pub spaces: Grid,
    pub special: u32,
}

pub type PlayerNum = usize;

#[derive(Copy, Clone)]
pub enum BoardSpace {
    Empty,
    PlayerInk {
        player_num: PlayerNum,
    },
    PlayerSpecial {
        player_num: PlayerNum,
        is_activated: bool,
    },
    Wall,
    OutOfBounds,
}

impl BoardSpace {
    pub fn is_ink(&self, num: PlayerNum) -> bool {
        match self {
            BoardSpace::PlayerInk { player_num } | BoardSpace::PlayerSpecial { player_num, .. } => {
                *player_num == num
            }
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub enum CardState {
    Available(Card),
    Unavailable,
}

pub type Hand = [Card; HAND_SIZE];
pub type Deck = [CardState; DECK_SIZE];

pub struct Player {
    pub hand: Hand,
    pub deck: Deck,
    pub special: u32,
}

impl Player {
    pub fn replace_card(&mut self, idx: usize) {
        self.discard_card(idx);
        // Don't replace the card if we're out of cards, since the game is over anyway.
        if let Some(card) = self.draw_card() {
            self.hand[idx] = card;
        }
    }

    fn discard_card(&mut self, idx: usize) {
        let card_to_discard = self.hand[idx];
        let discarded_card_option = self
            .deck
            .iter_mut()
            .find(|c| **c == CardState::Available(card_to_discard));
        if let Some(discarded_card) = discarded_card_option {
            *discarded_card = CardState::Unavailable;
        }
    }

    // TODO: Pass in the RNG for easier testing
    fn draw_card(&mut self) -> Option<Card> {
        let mut rng = thread_rng();
        let new_card = self
            .deck
            .iter()
            .filter_map(|cs| match cs {
                CardState::Available(card) => Some(card),
                CardState::Unavailable => None,
            })
            .choose(&mut rng)?;
        Some(*new_card)
    }
}

// This cannot be an array, because custom boards might be loaded at runtime
pub struct Board(pub Vec<Vec<BoardSpace>>);

impl Board {
    // Need to take signed integers because we may need to check out-of-bounds
    pub fn get_space(&self, x: i32, y: i32) -> BoardSpace {
        self.try_get_space(x, y).unwrap_or(BoardSpace::OutOfBounds)
    }

    fn try_get_space(&self, x: i32, y: i32) -> Option<BoardSpace> {
        let x = usize::try_from(x).ok()?;
        let y = usize::try_from(y).ok()?;

        let row = self.0.get(y)?;
        let space = row.get(x)?;
        Some(*space)
    }

    pub fn get_inactive_specials(&self, player_num: PlayerNum) -> Vec<(usize, usize, BoardSpace)> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(y, r)| {
                r.iter()
                    .enumerate()
                    .filter(move |(x, s)| {
                        is_inactive_special(s, player_num)
                            && is_surrounded(*x as i32, y as i32, self)
                    })
                    .map(move |(x, s)| (x, y, *s))
            })
            .collect()
    }
}

pub struct Placement {
    // Inked spaces with absolute board positions
    pub ink_spaces: Vec<(i32, i32, BoardSpace)>,
    pub special_activated: bool,
}

enum Outcome {
    P1Win,
    P2Win,
    Draw,
}

pub struct GameState {
    pub board: Board,
    pub players: [Player; 2],
    pub turns_left: u32,
}

impl GameState {
    pub fn place(&mut self, card_idx: usize, placement: Placement, player_num: PlayerNum) {
        let player = &mut self.players[player_num];
        let selected_card = player.hand[card_idx];
        if placement.special_activated {
            player.special -= selected_card.special;
        }
        for (x, y, s) in placement.ink_spaces {
            (self.board.0)[y as usize][x as usize] = s;
        }
        let special_spaces = self.board.get_inactive_specials(player_num);
        // activate surrounded special spaces
        for (x, y, s) in &special_spaces {
            self.board.0[*y][*x] = BoardSpace::PlayerSpecial {
                player_num,
                is_activated: true,
            }
        }
        player.special += special_spaces.len() as u32;
    }

    pub fn place_both(
        &mut self,
        card_idx1: usize,
        card_idx2: usize,
        p1_input: Placement,
        p2_input: Placement,
    ) {
    }

    pub fn check_winner(&self) -> Outcome {
        let p1_inked_spaces = count_inked_spaces(&self.board, 0);
        let p2_inked_spaces = count_inked_spaces(&self.board, 1);

        use std::cmp::Ordering;
        match p1_inked_spaces.cmp(&p2_inked_spaces) {
            Ordering::Greater => Outcome::P1Win,
            Ordering::Less => Outcome::P2Win,
            Ordering::Equal => Outcome::Draw,
        }
    }
}

// Get the spaces surrounding the given position
pub fn surrounding_spaces(x: i32, y: i32, board: &Board) -> [BoardSpace; 8] {
    let nw_space = board.get_space(x - 1, y - 1);
    let n_space = board.get_space(x, y - 1);
    let ne_space = board.get_space(x + 1, y - 1);
    let w_space = board.get_space(x - 1, y);
    let e_space = board.get_space(x + 1, y);
    let sw_space = board.get_space(x - 1, y + 1);
    let s_space = board.get_space(x, y + 1);
    let se_space = board.get_space(x + 1, y + 1);

    [
        nw_space, n_space, ne_space, w_space, e_space, sw_space, s_space, se_space,
    ]
}

fn is_surrounded(x: i32, y: i32, board: &Board) -> bool {
    surrounding_spaces(x, y, board)
        .iter()
        .all(|s| !matches!(s, BoardSpace::Empty))
}

fn is_inactive_special(s: &BoardSpace, num: PlayerNum) -> bool {
    match s {
        BoardSpace::PlayerSpecial {
            player_num,
            is_activated,
        } => *player_num == num && !is_activated,
        _ => false,
    }
}

fn count_inked_spaces(board: &Board, player_num: PlayerNum) -> u32 {
    board.0.iter().fold(0, |acc, row| {
        acc + row
            .iter()
            .filter(|s| s.is_ink(player_num))
            .fold(0, |acc, c| acc + 1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
