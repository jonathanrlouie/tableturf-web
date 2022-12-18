use crate::tableturf::player::PlayerNum;
use serde::{Serialize, Deserialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum Dimension {
    Width(usize),
    Height(usize),
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dimension::Width(w) => write!(f, "width {}", w),
            Dimension::Height(h) => write!(f, "height {}", h),
        }
    }
}

#[derive(Error, Debug)]
pub enum BoardError {
    #[error("Board with no rows given")]
    NoRows,
    #[error("Board of {dimension} exceeds the maximum of {max}")]
    TooLarge {
        dimension: Dimension,
        max: Dimension,
    },
    #[error("Board contains empty rows")]
    EmptyRows,
    #[error("Not all board rows have the same length")]
    MismatchedRowLengths,
}

#[derive(Debug)]
pub enum Coordinate {
    X,
    Y,
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coordinate::X => write!(f, "x"),
            Coordinate::Y => write!(f, "y"),
        }
    }
}

#[derive(Error, Debug)]
pub enum BoardPositionError {
    #[error("{0} coordinate {1} exceeds board {2}")]
    OutOfBounds(Coordinate, usize, Dimension),
    #[error("Final {coordinate} coordinate with base {base} and offset {offset} overflowed")]
    Overflow {
        coordinate: Coordinate,
        base: usize,
        offset: usize,
    },
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
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
            BoardSpace::Special { player_num, .. } => *player_num == num,
            _ => false,
        }
    }

    pub fn is_inactive_special(&self, num: PlayerNum) -> bool {
        match self {
            BoardSpace::Special {
                player_num,
                is_activated,
            } => *player_num == num && !is_activated,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoardPosition(usize, usize);

impl BoardPosition {
    // Ensure that the given position meets the following criteria:
    // - x coordinate does not exceed Board row length
    // - y coordinate does not exceed Board column height
    pub fn new(board: &Board, x: usize, y: usize) -> Result<Self, BoardPositionError> {
        let width = board.width();
        let height = board.height();
        if x >= width {
            return Err(BoardPositionError::OutOfBounds(
                Coordinate::X,
                x,
                Dimension::Width(width),
            ));
        }
        if y >= height {
            return Err(BoardPositionError::OutOfBounds(
                Coordinate::Y,
                y,
                Dimension::Height(height),
            ));
        }
        Ok(BoardPosition(x, y))
    }

    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }

    // Get the spaces surrounding the given position
    pub fn surrounding_spaces(&self, board: &Board) -> [BoardSpace; 8] {
        let x = self.0;
        let y = self.1;
        let x_minus_1 = usize::checked_sub(x, 1);
        let y_minus_1 = usize::checked_sub(y, 1);
        let x_plus_1 = usize::checked_add(x, 1);
        let y_plus_1 = usize::checked_add(y, 1);
        let nw_space = match (x_minus_1, y_minus_1) {
            (Some(x), Some(y)) => board.get_space(x, y),
            _ => BoardSpace::OutOfBounds
        };

        let n_space = match y_minus_1 {
            Some(y) => board.get_space(x, y),
            None => BoardSpace::OutOfBounds
        };

        let ne_space = match (x_plus_1, y_minus_1) {
            (Some(x), Some(y)) => board.get_space(x, y),
            _ => BoardSpace::OutOfBounds
        };

        let w_space = match x_minus_1 {
            Some(x) => board.get_space(x, y),
            None => BoardSpace::OutOfBounds
        };

        let e_space = match x_plus_1 {
            Some(x) => board.get_space(x, y),
            None => BoardSpace::OutOfBounds
        };

        let sw_space = match (x_minus_1, y_plus_1) {
            (Some(x), Some(y)) => board.get_space(x, y),
            _ => BoardSpace::OutOfBounds
        };

        let s_space = match y_plus_1 {
            Some(y) => board.get_space(x, y),
            None => BoardSpace::OutOfBounds
        };

        let se_space = match (x_plus_1, y_plus_1) {
            (Some(x), Some(y)) => board.get_space(x, y),
            _ => BoardSpace::OutOfBounds
        };

        [
            nw_space, n_space, ne_space, w_space, e_space, sw_space, s_space, se_space,
        ]
    }

    pub fn is_surrounded(&self, board: &Board) -> bool {
        self.surrounding_spaces(board)
            .iter()
            .all(|s| !matches!(s, BoardSpace::Empty))
    }

    // Test if a single space is adjacent to a player's special space
    pub fn adjacent_to_special(&self, board: &Board, player_num: PlayerNum) -> bool {
        self.surrounding_spaces(board)
            .iter()
            .any(|s| s.is_special(player_num))
    }

    // Test if a single space is adjacent to a player's inked space
    pub fn adjacent_to_ink(&self, board: &Board, player_num: PlayerNum) -> bool {
        self.surrounding_spaces(board)
            .iter()
            .any(|s| s.is_ink(player_num))
    }
}

pub const MAX_BOARD_WIDTH: usize = 26;
pub const MAX_BOARD_HEIGHT: usize = 26;

// This cannot be an array, because custom boards might be loaded at runtime.
// Note that boards are padded due to some cards being difficult to place at
// the boundaries of the board.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Board {
    width: usize,
    height: usize,
    spaces: Vec<BoardSpace>
}

const PADDING: usize = 7;
const DOUBLE_PADDING: usize = PADDING * 2;

impl Board {
    // Ensure that the given board meets the following criteria:
    // - all rows are the same length
    // - row length does not exceed the max board width
    // - column height does not exceed the max board height
    // - board contains at least one row
    // - rows contain at least one space
    // spaces is an unpadded board
    pub fn new(spaces: Vec<Vec<BoardSpace>>) -> Result<Self, BoardError> {
        if spaces.is_empty() {
            return Err(BoardError::NoRows);
        }
        let height = spaces.len();
        if height > MAX_BOARD_HEIGHT {
            return Err(BoardError::TooLarge {
                dimension: Dimension::Height(height),
                max: Dimension::Height(MAX_BOARD_HEIGHT),
            });
        }
        let row = &spaces[0];
        if row.is_empty() {
            return Err(BoardError::EmptyRows);
        }
        let width = row.len();
        if width > MAX_BOARD_WIDTH {
            return Err(BoardError::TooLarge {
                dimension: Dimension::Width(width),
                max: Dimension::Width(MAX_BOARD_WIDTH),
            });
        }
        if spaces.iter().any(|row| row.len() != width) {
            return Err(BoardError::MismatchedRowLengths);
        }

        // Need to add padding because some cards may be too difficult to place at board edges
        let start_padding = (0..(width + DOUBLE_PADDING) * PADDING).map(|_| BoardSpace::OutOfBounds);
        let end_padding = start_padding.clone();
        let padded_rows = spaces.iter().map(|row| pad_row(row.iter().cloned())).flatten();
        let padded_spaces = start_padding.chain(padded_rows).chain(end_padding);
        Ok(Board {
            width: width + DOUBLE_PADDING,
            height: height + DOUBLE_PADDING,
            spaces: padded_spaces.collect()
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn spaces(&self) -> &Vec<BoardSpace> {
        &self.spaces
    }

    pub fn set_space(&mut self, position: &BoardPosition, space: BoardSpace) {
        self.spaces[position.y() * self.width + position.x()] = space;
    }

    pub fn get_space(&self, x: usize, y: usize) -> BoardSpace {
        self.try_get_space(x, y).unwrap_or(BoardSpace::OutOfBounds)
    }

    fn try_get_space(&self, x: usize, y: usize) -> Option<BoardSpace> {
        let temp = usize::checked_mul(y, self.width)?;
        let idx = usize::checked_add(temp, x)?;
        let space = self.spaces.get(idx)?;
        Some(*space)
    }

    pub fn get_surrounded_inactive_specials(
        &self,
        player_num: PlayerNum,
    ) -> Vec<(BoardPosition, BoardSpace)> {
        self.spaces
            .iter()
            .enumerate()
            .filter_map(move |(idx, s)| self.get_surrounded_inactive_special(player_num, idx, *s))
            .collect()
    }

    fn get_surrounded_inactive_special(
        &self,
        player_num: PlayerNum,
        idx: usize,
        space: BoardSpace
    ) -> Option<(BoardPosition, BoardSpace)> {
        let x = idx % self.width;
        let y = idx / self.width;
        let board_pos = BoardPosition::new(self, x, y).unwrap();
        if space.is_inactive_special(player_num) && board_pos.is_surrounded(self) {
            Some((board_pos, space))
        } else {
            None
        }
    }

    pub fn set_ink(&mut self, ink_spaces: Vec<(BoardPosition, BoardSpace)>) {
        for (bp, s) in ink_spaces {
            self.set_space(&bp, s);
        }
    }

    pub fn count_inked_spaces(&self, player_num: PlayerNum) -> u32 {
        self.spaces.iter()
            .filter(|s| s.is_ink(player_num))
            .fold(0, |acc, _| acc + 1)
    }

    // Calculate the absolute board position for a given base position and offsets
    pub fn get_absolute_position(
        &self,
        // The number of spaces to the right of the board_x base position
        x_offset: usize,
        // The number of spaces down from the board_y base position
        y_offset: usize,
        board_x: usize,
        board_y: usize,
    ) -> Result<BoardPosition, BoardPositionError> {
        let x = usize::checked_add(board_x, x_offset).ok_or(BoardPositionError::Overflow {
            coordinate: Coordinate::X,
            base: board_x,
            offset: x_offset,
        })?;
        let y = usize::checked_add(board_y, y_offset).ok_or(BoardPositionError::Overflow {
            coordinate: Coordinate::Y,
            base: board_y,
            offset: y_offset,
        })?;
        BoardPosition::new(self, x, y)
    }
}

fn pad_row(row: impl Iterator<Item = BoardSpace>) -> impl Iterator<Item = BoardSpace> {
    let initial_padding = (0..PADDING).map(|_| BoardSpace::OutOfBounds);
    let end_padding = initial_padding.clone();
    initial_padding.chain(row).chain(end_padding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_board() {
        let no_rows = Board::new(vec![]);
        assert!(no_rows.is_err());

        let empty_row = Board::new(vec![vec![]]);
        assert!(empty_row.is_err());

        let wall = BoardSpace::Wall;
        let empty = BoardSpace::Empty;
        let uneven_rows = Board::new(vec![vec![wall], vec![wall, wall]]);
        assert!(uneven_rows.is_err());

        let too_wide_board = Board::new(vec![vec![
            wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
        ]]);
        assert!(too_wide_board.is_err());

        let too_tall_board = Board::new(vec![
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
            vec![wall],
        ]);
        assert!(too_tall_board.is_err());

        let min_valid_board = Board::new(vec![vec![wall]]);
        assert!(min_valid_board.is_ok());

        let valid_board = Board::new(vec![vec![wall, wall], vec![wall, empty]]);
        assert!(valid_board.is_ok());

        let max_valid_board = Board::new(vec![
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
            vec![
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
                wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            ],
        ]);
        assert!(max_valid_board.is_ok());
    }

    #[test]
    fn test_construct_board_position() {
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![vec![empty, empty], vec![empty, empty]]).unwrap();
        let outside_row = BoardPosition::new(&board, 16, 7);
        assert!(outside_row.is_err());
        let outside_col = BoardPosition::new(&board, 7, 16);
        assert!(outside_col.is_err());
        let outside_row_and_col = BoardPosition::new(&board, 16, 16);
        assert!(outside_row_and_col.is_err());
        let valid_pos = BoardPosition::new(&board, 15, 15);
        assert!(valid_pos.is_ok());
    }

    #[test]
    fn test_surrounding_spaces() {
        let oob = BoardSpace::OutOfBounds;
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![vec![empty, empty], vec![empty, empty]]).unwrap();
        let spaces = BoardPosition::new(&board, 7, 7)
            .unwrap()
            .surrounding_spaces(&board);
        assert_eq!(spaces[0], oob);
        assert_eq!(spaces[1], oob);
        assert_eq!(spaces[2], oob);
        assert_eq!(spaces[3], oob);
        assert_eq!(spaces[4], empty);
        assert_eq!(spaces[5], oob);
        assert_eq!(spaces[6], empty);
        assert_eq!(spaces[7], empty);
    }

    #[test]
    fn test_is_surrounded() {
        let wall = BoardSpace::Wall;
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![
            vec![empty, wall, empty],
            vec![wall, wall, empty],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let surrounded_pos = BoardPosition::new(&board, 7, 7).unwrap();
        assert!(surrounded_pos.is_surrounded(&board));
        let not_surrounded_pos = BoardPosition::new(&board, 8, 7).unwrap();
        assert!(!not_surrounded_pos.is_surrounded(&board));
    }

    #[test]
    fn test_adjacent_to_ink() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![p2_ink, empty, p1_special],
            vec![empty, empty, empty],
        ])
        .unwrap();
        let pos1 = BoardPosition::new(&board, 7, 7).unwrap();
        assert!(pos1.adjacent_to_ink(&board, PlayerNum::P1));
        let pos2 = BoardPosition::new(&board, 7, 9).unwrap();
        assert!(!pos2.adjacent_to_ink(&board, PlayerNum::P1));
        let pos3 = BoardPosition::new(&board, 9, 9).unwrap();
        assert!(pos3.adjacent_to_ink(&board, PlayerNum::P1));
    }

    #[test]
    fn test_adjacent_to_special() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let p2_special = BoardSpace::Special {
            player_num: PlayerNum::P2,
            is_activated: false,
        };
        let board = Board::new(vec![
            vec![empty, p1_ink, empty],
            vec![p2_ink, empty, p1_special],
            vec![empty, p2_special, empty],
        ])
        .unwrap();
        let pos1 = BoardPosition::new(&board, 7, 7).unwrap();
        assert!(!pos1.adjacent_to_special(&board, PlayerNum::P1));
        let pos2 = BoardPosition::new(&board, 7, 9).unwrap();
        assert!(!pos2.adjacent_to_special(&board, PlayerNum::P1));
        let pos3 = BoardPosition::new(&board, 9, 9).unwrap();
        assert!(pos3.adjacent_to_special(&board, PlayerNum::P1));
        let pos4 = BoardPosition::new(&board, 8, 7).unwrap();
        assert!(pos4.adjacent_to_special(&board, PlayerNum::P1));
    }

    #[test]
    fn test_get_surrounded_inactive_specials() {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let board = Board::new(vec![vec![empty, empty], vec![empty, empty]]).unwrap();
        let no_spaces = board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert!(no_spaces.is_empty());

        let special_surrounded =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, p1_ink]]).unwrap();
        let one_special = special_surrounded.get_surrounded_inactive_specials(PlayerNum::P1);
        assert_eq!(one_special.len(), 1);
        assert_eq!(one_special[0].0.x(), 7);
        assert_eq!(one_special[0].0.y(), 7);

        let off_by_one_board =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, empty]]).unwrap();
        let no_special = off_by_one_board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert!(no_special.is_empty());

        let enemy_ink_board =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, p2_ink]]).unwrap();
        let one_special = enemy_ink_board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert_eq!(one_special.len(), 1);
        assert_eq!(one_special[0].0.x(), 7);
        assert_eq!(one_special[0].0.y(), 7);

        let multiple_specials_board =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, p1_special]]).unwrap();
        let two_specials = multiple_specials_board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert_eq!(two_specials.len(), 2);
        assert_eq!(two_specials[0].0.x(), 7);
        assert_eq!(two_specials[0].0.y(), 7);
        assert_eq!(two_specials[1].0.x(), 8);
        assert_eq!(two_specials[1].0.y(), 8);
    }

    #[test]
    fn test_count_inked_spaces() {
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let p2_special = BoardSpace::Special {
            player_num: PlayerNum::P2,
            is_activated: false,
        };
        let empty = BoardSpace::Empty;
        let wall = BoardSpace::Wall;
        let board = Board::new(vec![
            vec![empty, p1_ink, p2_ink],
            vec![empty, wall, p1_special],
            vec![p2_special, empty, p1_ink],
        ])
        .unwrap();
        let player1_ink_total = board.count_inked_spaces(PlayerNum::P1);
        let player2_ink_total = board.count_inked_spaces(PlayerNum::P2);
        assert_eq!(player1_ink_total, 3);
        assert_eq!(player2_ink_total, 2);
    }

    #[test]
    fn test_get_absolute_position() {
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![vec![empty, empty], vec![empty, empty]]).unwrap();
        let valid_pos = board.get_absolute_position(7, 7, 0, 0);
        assert!(valid_pos.is_ok());
        assert_eq!(
            valid_pos.unwrap(),
            BoardPosition::new(&board, 7, 7).unwrap()
        );
    }
}
