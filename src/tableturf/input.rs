use crate::tableturf::board::{Board, BoardPosition, BoardPositionError, BoardSpace};
use crate::tableturf::card::{Card, Grid, InkSpace, ROW_LEN};
use crate::tableturf::deck::HandIndex;
use crate::tableturf::player::{Player, PlayerNum};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Insufficient special. Current special: {special}. Required: {required}")]
    InsufficientSpecial {
        special: u32,
        required: u32,
    },
    InvalidPosition(BoardPositionError),
}

// Represents the number of counter-clockwise rotations applied to a Card
#[derive(Deserialize, Copy, Clone, Debug)]
pub enum Rotation {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Deserialize, Debug)]
pub enum Action {
    Pass,
    Place {
        // The x and y coordinates of the top-left corner of the card's grid
        x: i32,
        y: i32,
        special_activated: bool,
        rotation: Rotation,
    },
}

#[derive(Deserialize, Debug)]
pub struct RawInput {
    pub hand_idx: HandIndex,
    pub action: Action,
}

#[derive(Clone, Debug)]
pub struct ValidInput {
    hand_idx: HandIndex,
    input: Input,
}

#[derive(Clone, Debug)]
pub enum Input {
    Pass,
    Place(Placement),
}

#[derive(Clone, Debug)]
pub struct Placement {
    // Inked spaces with absolute board positions
    ink_spaces: Vec<(BoardPosition, InkSpace)>,
    special_activated: bool,
}

impl Placement {
    pub fn new(
        board_position: (i32, i32),
        special_activated: bool,
        rotation: Rotation,
        hand_idx: HandIndex,
        board: &Board,
        player: &Player,
        player_num: PlayerNum,
    ) -> Result<Placement, InputError> {
        let (board_x, board_y) = board_position;
        let selected_card = player.get_card(hand_idx);
        let grid = rotate_input(&selected_card, rotation);
        let result = grid
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(move |(x, cell)| cell.map(|s| (x, y, s)))
            })
            .map(|(x, y, s)| {
                board
                    .get_absolute_position(x, y, board_x, board_y)
                    .map(|bp| (bp, s))
            })
            .collect::<Result<Vec<(BoardPosition, InkSpace)>, BoardPositionError>>();
        let ink_spaces = result.map_err(|e| InputError::InvalidPosition(e))?;

        if special_activated {
            // Check that player has enough special and that the special isn't
            // overlapping any walls or special spaces.
            if player.special < selected_card.special() {
                return Err(InputError::InsufficientSpecial(
                    player.special,
                    selected_card.special(),
                ));
            }

            if special_collision(&ink_spaces[..], board) {
                return Err("special placement is overlapping walls or special spaces".to_string());
            }

            // Check that ink placement is adjacent to one of the player's special spaces
            if !placement_adjacent_to_special(&ink_spaces[..], board, player_num) {
                return Err("special placement not adjacent to a special square".to_string());
            }
        // Check that ink placement is over empty squares
        } else {
            if placement_collision(&ink_spaces[..], board) {
                return Err("ink placement not over empty tiles".to_string());
            }

            // Check that ink placement is adjacent to player's ink
            if !placement_adjacent_to_ink(&ink_spaces[..], board, player_num) {
                return Err("ink placement not adjacent to player's ink".to_string());
            }
        }
        Ok(Placement {
            ink_spaces,
            special_activated,
        })
    }

    pub fn ink_spaces(&self) -> &Vec<(BoardPosition, InkSpace)> {
        &self.ink_spaces
    }

    pub fn is_special_activated(&self) -> bool {
        self.special_activated
    }

    pub fn into_board_spaces(self, player_num: PlayerNum) -> Vec<(BoardPosition, BoardSpace)> {
        self.ink_spaces
            .iter()
            .map(|(bp, s)| (*bp, into_board_space(s, player_num)))
            .collect()
    }
}

impl ValidInput {
    // validates:
    // - board position
    // - card index in hand
    // - special availability
    pub fn new(
        input: RawInput,
        board: &Board,
        player: &Player,
        player_num: PlayerNum,
    ) -> Result<Self, InputError> {
        let hand_idx = input.hand_idx;
        match input.action {
            Action::Pass => Ok(Self {
                hand_idx,
                input: Input::Pass,
            }),
            Action::Place {
                x: board_x,
                y: board_y,
                special_activated,
                rotation,
            } => {
                let placement = Placement::new(
                    (board_x, board_y),
                    special_activated,
                    rotation,
                    hand_idx,
                    board,
                    player,
                    player_num,
                )?;

                Ok(Self {
                    hand_idx,
                    input: Input::Place(placement),
                })
            }
        }
    }

    pub fn get(self) -> Input {
        self.input
    }

    pub fn hand_idx(&self) -> HandIndex {
        self.hand_idx
    }
}

fn into_board_space(ink_space: &InkSpace, player_num: PlayerNum) -> BoardSpace {
    match ink_space {
        InkSpace::Normal => BoardSpace::Ink { player_num },
        InkSpace::Special => BoardSpace::Special {
            player_num,
            is_activated: false,
        },
    }
}

// Test if an entire placement of ink is adjacent to a player's inked space
fn placement_adjacent_to_ink(
    inked_spaces: &[(BoardPosition, InkSpace)],
    board: &Board,
    player_num: PlayerNum,
) -> bool {
    inked_spaces
        .iter()
        .any(|(bp, _)| bp.adjacent_to_ink(board, player_num))
}

// Test if an entire placement of ink is adjacent to one of the player's special spaces
fn placement_adjacent_to_special(
    inked_spaces: &[(BoardPosition, InkSpace)],
    board: &Board,
    player_num: PlayerNum,
) -> bool {
    inked_spaces
        .iter()
        .any(|(bp, _)| bp.adjacent_to_special(board, player_num))
}

// Test if an entire placement of ink overlaps ink or walls
fn placement_collision(inked_spaces: &[(BoardPosition, InkSpace)], board: &Board) -> bool {
    inked_spaces.iter().any(|(bp, _)| {
        !matches!(
            board.get_space(bp.x() as i32, bp.y() as i32),
            BoardSpace::Empty
        )
    })
}

// Test if an entire special placement of ink overlaps any special spaces or walls
fn special_collision(inked_spaces: &[(BoardPosition, InkSpace)], board: &Board) -> bool {
    inked_spaces.iter().any(|(bp, _)| {
        let board_space = board.get_space(bp.x() as i32, bp.y() as i32);
        matches!(board_space, BoardSpace::Special { .. })
            || matches!(board_space, BoardSpace::Wall)
            || matches!(board_space, BoardSpace::OutOfBounds)
    })
}

fn rotate_grid_ccw(grid: &mut Grid) {
    for i in 0..(ROW_LEN / 2) {
        for j in i..(ROW_LEN - i - 1) {
            let temp = grid[i][j];
            grid[i][j] = grid[j][ROW_LEN - 1 - i];
            grid[j][ROW_LEN - 1 - i] = grid[ROW_LEN - 1 - i][ROW_LEN - 1 - j];
            grid[ROW_LEN - 1 - i][ROW_LEN - 1 - j] = grid[ROW_LEN - 1 - j][i];
            grid[ROW_LEN - 1 - j][i] = temp;
        }
    }
}

fn rotate_input(card: &Card, rotation: Rotation) -> Grid {
    let mut grid = card.spaces();
    match rotation {
        Rotation::Zero => (),
        Rotation::One => {
            rotate_grid_ccw(&mut grid);
        }
        Rotation::Two => {
            rotate_grid_ccw(&mut grid);
            rotate_grid_ccw(&mut grid);
        }
        Rotation::Three => {
            rotate_grid_ccw(&mut grid);
            rotate_grid_ccw(&mut grid);
            rotate_grid_ccw(&mut grid);
        }
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::board::{Board, BoardPosition};
    use crate::tableturf::card::{Card, CardSpace, InkSpace};
    use crate::tableturf::deck::{Deck, DeckIndex, DrawRng, Hand, HandIndex};

    struct MockRng;
    struct MockRng2;

    impl DrawRng for MockRng {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Vec<DeckIndex> {
            let v: Vec<DeckIndex> = iter.collect();
            vec![v[13], v[1], v[2], v[3]]
        }
    }

    impl DrawRng for MockRng2 {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Vec<DeckIndex> {
            let v: Vec<DeckIndex> = iter.collect();
            vec![v[13], v[1], v[2], v[3]]
        }
    }

    #[test]
    fn test_rotate_input() {
        let e: CardSpace = None;
        let i: CardSpace = Some(InkSpace::Normal);
        let s: CardSpace = Some(InkSpace::Special);
        let splattershot = Card::new(
            8,
            [
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, i, i, s, e, e, e],
                [e, e, i, i, i, i, e, e],
                [e, e, i, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
            ],
            3,
        );
        let zero_rotations = rotate_input(&splattershot, Rotation::Zero);
        assert_eq!(
            zero_rotations,
            [
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, i, i, s, e, e, e],
                [e, e, i, i, i, i, e, e],
                [e, e, i, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
            ]
        );

        let one_rotation = rotate_input(&splattershot, Rotation::One);
        assert_eq!(
            one_rotation,
            [
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, i, e, e, e, e],
                [e, e, s, i, e, e, e, e],
                [e, e, i, i, e, e, e, e],
                [e, e, i, i, i, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
            ]
        );

        let two_rotations = rotate_input(&splattershot, Rotation::Two);
        assert_eq!(
            two_rotations,
            [
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, i, e, e],
                [e, e, i, i, i, i, e, e],
                [e, e, e, s, i, i, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
            ]
        );

        let three_rotations = rotate_input(&splattershot, Rotation::Three);
        assert_eq!(
            three_rotations,
            [
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, i, i, i, e, e],
                [e, e, e, e, i, i, e, e],
                [e, e, e, e, i, s, e, e],
                [e, e, e, e, i, e, e, e],
                [e, e, e, e, e, e, e, e],
                [e, e, e, e, e, e, e, e],
            ]
        );
    }

    fn custom_deck() -> [Card; 15] {
        let e: CardSpace = None;
        let i: CardSpace = Some(InkSpace::Normal);
        let s: CardSpace = Some(InkSpace::Special);
        [
            // Splattershot
            Card::new(
                8,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, i, i, s, e, e, e],
                    [e, e, i, i, i, i, e, e],
                    [e, e, i, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                3,
            ),
            // Custom card 1
            Card::new(
                6,
                [
                    [i, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                3,
            ),
            // Custom card 2
            Card::new(
                9,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, i],
                ],
                4,
            ),
            // Blaster
            Card::new(
                8,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, i, e, e, i, s, e, e],
                    [e, e, i, i, i, i, e, e],
                    [e, e, i, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                3,
            ),
            // Splat Dualies
            Card::new(
                8,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, i, i, i, i, e, e],
                    [e, e, i, s, e, e, e, e],
                    [e, i, i, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                3,
            ),
            // Flooder
            Card::new(
                14,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, i, s, i, i, i, e, e],
                    [e, i, e, i, e, i, e, e],
                    [e, i, e, i, e, i, e, e],
                    [e, i, e, i, e, i, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                5,
            ),
            // Splat Roller
            Card::new(
                9,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, i, i, s, i, i, e, e],
                    [e, e, e, i, i, i, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                4,
            ),
            // Tri-Stringer
            Card::new(
                11,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, i, s, i, i, i, e, e],
                    [e, i, e, i, e, e, e, e],
                    [e, i, i, e, e, e, e, e],
                    [e, i, e, e, e, e, e, e],
                    [e, i, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                4,
            ),
            // Chum
            Card::new(
                5,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, s, i, e, e, e, e],
                    [e, e, e, i, i, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                2,
            ),
            // Splat Charger
            Card::new(
                8,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [i, i, i, i, i, i, i, e],
                    [e, e, s, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                3,
            ),
            // Splatana Wiper
            Card::new(
                5,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, s, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                2,
            ),
            // SquidForce
            Card::new(
                10,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, i, i, i, i, i, e, e],
                    [e, e, e, i, s, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                4,
            ),
            // Heavy Splatling
            Card::new(
                12,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, i, i, e, e, e, e, e],
                    [e, i, i, e, e, e, e, e],
                    [e, i, i, e, e, e, e, e],
                    [e, e, i, i, e, e, e, e],
                    [e, e, e, i, s, e, e, e],
                    [e, e, e, e, i, i, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                5,
            ),
            // Splat Bomb
            Card::new(
                3,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, s, e, e, e],
                    [e, e, e, i, i, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                1,
            ),
            // Marigold
            Card::new(
                15,
                [
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, i, e, e, e, e],
                    [e, e, i, i, i, e, e, e],
                    [e, i, e, i, e, i, e, e],
                    [e, i, i, s, i, i, e, e],
                    [e, e, i, i, i, e, e, e],
                    [e, e, e, e, e, e, e, e],
                    [e, e, e, e, e, e, e, e],
                ],
                5,
            ),
        ]
    }

    fn draw_hand() -> (Deck, Hand) {
        Deck::draw_hand(custom_deck(), &mut MockRng).unwrap()
    }

    fn draw_hand2() -> (Deck, Hand) {
        Deck::draw_hand(custom_deck(), &mut MockRng2).unwrap()
    }

    #[test]
    fn test_construct_placement() {
        // Test placing ink on top of empty spaces
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, p1_ink],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let (deck, hand) = draw_hand2();
        let special = 5;
        let player = Player::new(hand, deck, special);
        let placement = Placement::new(
            (-3, -3),
            false,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_ok());
        let placement = placement.unwrap();
        assert_eq!(placement.ink_spaces.len(), 3);
        assert_eq!(
            placement.ink_spaces[0].0,
            BoardPosition::new(&board, 0, 0).unwrap()
        );
        assert_eq!(
            placement.ink_spaces[1].0,
            BoardPosition::new(&board, 1, 0).unwrap()
        );
        assert_eq!(
            placement.ink_spaces[2].0,
            BoardPosition::new(&board, 0, 1).unwrap()
        );

        // Test placing ink on top of an inked space
        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![empty, p1_ink, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            false,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_err());

        // Test placing special on top of an inked space
        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![empty, p1_special, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            true,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_ok());

        let (_draw, hand) = draw_hand();
        let player_no_special = Player::new(hand, deck, 0);
        // Test placing special with insufficient special meter
        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![empty, p1_special, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            true,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player_no_special,
            PlayerNum::P1,
        );
        assert!(placement.is_err());

        // Test placing special on top of a special space
        let board = Board::new(vec![
            vec![empty, p1_special, empty],
            vec![empty, p1_special, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            true,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_err());

        // Test placing ink without any ink nearby
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            false,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_err());

        // Test placing special without any special nearby
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, p1_ink, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            true,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_err());

        // Test placing ink with a special space nearby
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, p1_special, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let placement = Placement::new(
            (-3, -3),
            false,
            Rotation::Two,
            HandIndex::H1,
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(placement.is_ok());
    }

    #[test]
    fn test_placement_collision() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(!placement_collision(&ink_spaces, &board));

        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(placement_collision(&ink_spaces, &board));
    }

    #[test]
    fn test_placement_adjacent_to_ink() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, p1_ink],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(placement_adjacent_to_ink(
            &ink_spaces,
            &board,
            PlayerNum::P1
        ));

        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, p1_special],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(placement_adjacent_to_ink(
            &ink_spaces,
            &board,
            PlayerNum::P1
        ));

        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(!placement_adjacent_to_ink(
            &ink_spaces,
            &board,
            PlayerNum::P1
        ));
    }

    #[test]
    fn test_special_collision() {
        let empty = BoardSpace::Empty;
        let wall = BoardSpace::Wall;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(!special_collision(&ink_spaces, &board));

        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(!special_collision(&ink_spaces, &board));

        let board = Board::new(vec![
            vec![empty, wall, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(special_collision(&ink_spaces, &board));
    }

    #[test]
    fn test_placement_adjacent_to_special() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, p1_ink],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(!placement_adjacent_to_special(
            &ink_spaces,
            &board,
            PlayerNum::P1
        ));

        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, p1_special],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(placement_adjacent_to_special(
            &ink_spaces,
            &board,
            PlayerNum::P1
        ));

        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, empty, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let ink_spaces = vec![
            (BoardPosition::new(&board, 0, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 1, 0).unwrap(), InkSpace::Normal),
            (BoardPosition::new(&board, 0, 1).unwrap(), InkSpace::Special),
        ];
        assert!(!placement_adjacent_to_special(
            &ink_spaces,
            &board,
            PlayerNum::P1
        ));
    }

    #[test]
    fn test_construct_valid_input() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty],
            vec![empty, p1_ink, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let (deck, hand) = draw_hand2();
        let special = 5;
        let player = Player::new(hand, deck, special);
        let input = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Pass,
            },
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(input.is_ok());

        // Test placing a card just out of bounds in the top-left
        let input = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H3,
                action: Action::Place {
                    x: -8,
                    y: -8,
                    special_activated: false,
                    rotation: Rotation::Zero,
                },
            },
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(input.is_err());

        // Test placing a card as far to the top-left as possible
        let input = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H3,
                action: Action::Place {
                    x: -7,
                    y: -7,
                    special_activated: false,
                    rotation: Rotation::Zero,
                },
            },
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(input.is_ok());

        // Test placing a card as far to the bottom-right as possible
        let input = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H2,
                action: Action::Place {
                    x: 2,
                    y: 2,
                    special_activated: false,
                    rotation: Rotation::Zero,
                },
            },
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(input.is_ok());

        // Test placing a card just out of bounds in the bottom-right
        let input = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H2,
                action: Action::Place {
                    x: 3,
                    y: 2,
                    special_activated: false,
                    rotation: Rotation::Zero,
                },
            },
            &board,
            &player,
            PlayerNum::P1,
        );
        assert!(input.is_err());
    }
}
