use std::cmp::Ordering;
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

impl InkSpace {
    fn into_board_space(self, player_num: PlayerNum) -> BoardSpace {
        match self {
            InkSpace::Normal => BoardSpace::Ink { player_num },
            InkSpace::Special => BoardSpace::Special {
                player_num,
                is_activated: false,
            },
        }
    }
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
    Ink {
        player_num: PlayerNum,
    },
    Special {
        player_num: PlayerNum,
        is_activated: bool,
    },
    Wall,
    OutOfBounds,
}

impl BoardSpace {
    pub fn is_ink(&self, num: PlayerNum) -> bool {
        match self {
            BoardSpace::Ink { player_num } | BoardSpace::Special { player_num, .. } => {
                *player_num == num
            }
            _ => false,
        }
    }

    pub fn is_special(&self, num: PlayerNum) -> bool {
        match self {
            BoardSpace::Special { player_num, .. } => {
                *player_num == num
            }
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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
    pub num: PlayerNum,
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
        let card_to_discard = &mut self.hand[idx];
        let discarded_card_idx = self
            .deck
            .iter()
            .position(|c| *c == CardState::Available(*card_to_discard));
        if let Some(discarded_card) = discarded_card_idx {
            self.deck[discarded_card] = CardState::Unavailable;
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

    pub fn spend_special(&mut self, placement: &Placement, card_idx: usize) {
        if placement.special_activated {
            let selected_card = self.hand[card_idx];
            self.special -= selected_card.special;
        }
    }

    pub fn update_special_gauge(&mut self, board: &mut Board) {
        let special_spaces = board.get_surrounded_inactive_specials(self.num);
        // activate surrounded special spaces
        for (x, y, _) in &special_spaces {
            board.0[*y][*x] = BoardSpace::Special {
                player_num: self.num,
                is_activated: true,
            }
        }
        self.special += special_spaces.len() as u32;
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

    pub fn get_surrounded_inactive_specials(&self, player_num: PlayerNum) -> Vec<(usize, usize, BoardSpace)> {
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

    pub fn set_ink(&mut self, ink_spaces: Vec<(i32, i32, BoardSpace)>) {
        for (x, y, s) in ink_spaces {
            (self.0)[y as usize][x as usize] = s;
        }
    }
}

pub struct Placement {
    // Inked spaces with absolute board positions
    pub ink_spaces: Vec<(i32, i32, InkSpace)>,
    pub special_activated: bool,
}

impl Placement {
    pub fn into_board_spaces(self, player_num: PlayerNum) -> Vec<(i32, i32, BoardSpace)> {
        self.ink_spaces.iter().map(|(x, y, s)| (*x, *y, s.into_board_space(player_num))).collect()
    }
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
        player.spend_special(&placement, card_idx);
        self.board.set_ink(placement.into_board_spaces(player_num));
    }

    pub fn place_both(
        &mut self,
        card_idx1: usize,
        card_idx2: usize,
        placement1: Placement,
        placement2: Placement,
    ) {
        // Spend special, if activated
        let player1 = &mut self.players[0];
        let priority1 = player1.hand[card_idx1].priority;
        player1.spend_special(&placement1, card_idx1);
        let player2 = &mut self.players[1];
        let priority2 = player2.hand[card_idx2].priority;
        player2.spend_special(&placement2, card_idx2);
        
        let overlap: Vec<(i32, i32, InkSpace, InkSpace)> = placement1.ink_spaces.iter()
            .filter_map(|(x1, y1, s1)| {
                placement2.ink_spaces.iter().find(|&&(x2, y2, _)| *x1 == x2 && *y1 == y2)
                    .map(|(_, _, s2)| (*x1, *y1, *s1, *s2))
            })
            .collect();
        
        if !overlap.is_empty() {
            let overlap_resolved = match priority1.cmp(&priority2) {
                Ordering::Greater => resolve_overlap(
                    overlap,
                    BoardSpace::Ink{ player_num: 1 },
                    BoardSpace::Special{ player_num: 1, is_activated: false }
                ),
                Ordering::Less => resolve_overlap(
                    overlap,
                    BoardSpace::Ink{ player_num: 0 },
                    BoardSpace::Special{ player_num: 0, is_activated: false }
                ),
                Ordering::Equal => resolve_overlap(
                    overlap,
                    BoardSpace::Wall,
                    BoardSpace::Wall
                )
            };
            // No need to try to find parts that don't overlap as long as
            // we set the overlapping ink last
            self.board.set_ink(placement1.into_board_spaces(0));
            self.board.set_ink(placement2.into_board_spaces(1));
            self.board.set_ink(overlap_resolved);
        } else {
            self.board.set_ink(placement1.into_board_spaces(0));
            self.board.set_ink(placement2.into_board_spaces(1));
        }
    }

    pub fn check_winner(&self) -> Outcome {
        let p1_inked_spaces = count_inked_spaces(&self.board, 0);
        let p2_inked_spaces = count_inked_spaces(&self.board, 1);

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
        BoardSpace::Special {
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
            .fold(0, |acc, _| acc + 1)
    })
}

fn resolve_overlap(
    overlap: Vec<(i32, i32, InkSpace, InkSpace)>,
    normal_collision_space: BoardSpace,
    special_collision_space: BoardSpace
) -> Vec<(i32, i32, BoardSpace)> {
    overlap.iter().map(|(x, y, s1, s2)| {
        (*x, *y, match (s1, s2) {
            (InkSpace::Normal, InkSpace::Normal) => normal_collision_space,
            (InkSpace::Special, InkSpace::Normal) => BoardSpace::Special{ player_num: 0, is_activated: false},
            (InkSpace::Normal, InkSpace::Special) => BoardSpace::Special{ player_num: 1, is_activated: false},
            (InkSpace::Special, InkSpace::Special) => special_collision_space
        })
    }).collect::<Vec<(i32, i32, BoardSpace)>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
