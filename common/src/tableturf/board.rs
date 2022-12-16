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
    #[error("{0} coordinate {1} could not be converted from usize to i32")]
    CoordinateToInt(Coordinate, usize),
    #[error("{0} offset {1} could not be converted from usize to i32")]
    OffsetToInt(Coordinate, usize),
    #[error("Final {coordinate} coordinate with base {base} and offset {offset} overflowed")]
    Overflow {
        coordinate: Coordinate,
        base: i32,
        offset: i32,
    },
    #[error("{0} coordinate {1} could not be converted from i32 to usize")]
    CoordinateToUsize(Coordinate, i32),
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
    // - x and y coordinates can be converted into i32s safely
    pub fn new(board: &Board, x: usize, y: usize) -> Result<Self, BoardPositionError> {
        let width = board.get()[0].len();
        let height = board.get().len();
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
        // Ensure that x and y can be safely converted into i32s later
        let _: i32 = x
            .try_into()
            .map_err(|_| BoardPositionError::CoordinateToInt(Coordinate::X, x))?;
        let _: i32 = y
            .try_into()
            .map_err(|_| BoardPositionError::CoordinateToInt(Coordinate::Y, y))?;
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
        let x = self.x() as i32;
        let y = self.y() as i32;
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

// This cannot be an array, because custom boards might be loaded at runtime
// TODO: Optimise this code by flattening the Vectors
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Board(Vec<Vec<BoardSpace>>);

impl Board {
    // Ensure that the given board meets the following criteria:
    // - all rows are the same length
    // - row length does not exceed the max board width
    // - column height does not exceed the max board height
    // - board contains at least one row
    // - rows contain at least one space
    pub fn new(spaces: Vec<Vec<BoardSpace>>) -> Result<Self, BoardError> {
        if spaces.is_empty() {
            return Err(BoardError::NoRows);
        }
        if spaces.len() > MAX_BOARD_HEIGHT {
            return Err(BoardError::TooLarge {
                dimension: Dimension::Height(spaces.len()),
                max: Dimension::Height(MAX_BOARD_HEIGHT),
            });
        }
        let row = &spaces[0];
        if row.is_empty() {
            return Err(BoardError::EmptyRows);
        }
        let row_len = row.len();
        if row_len > MAX_BOARD_WIDTH {
            return Err(BoardError::TooLarge {
                dimension: Dimension::Width(row_len),
                max: Dimension::Width(MAX_BOARD_WIDTH),
            });
        }
        if spaces.iter().any(|row| row.len() != row_len) {
            return Err(BoardError::MismatchedRowLengths);
        }
        Ok(Board(spaces))
    }

    pub fn height(&self) -> usize {
        self.0.len()
    }

    pub fn width(&self) -> usize {
        self.0[0].len()
    }

    pub fn get(&self) -> &Vec<Vec<BoardSpace>> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Vec<Vec<BoardSpace>> {
        &mut self.0
    }

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

    pub fn get_surrounded_inactive_specials(
        &self,
        player_num: PlayerNum,
    ) -> Vec<(BoardPosition, BoardSpace)> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(y, r)| {
                r.iter()
                    .enumerate()
                    .filter(move |(x, s)| {
                        let board_pos = BoardPosition::new(self, *x, y).unwrap();
                        s.is_inactive_special(player_num) && board_pos.is_surrounded(self)
                    })
                    .map(move |(x, s)| (BoardPosition::new(self, x, y).unwrap(), *s))
            })
            .collect()
    }

    pub fn set_ink(&mut self, ink_spaces: Vec<(BoardPosition, BoardSpace)>) {
        for (bp, s) in ink_spaces {
            (self.0)[bp.y() as usize][bp.x() as usize] = s;
        }
    }

    pub fn count_inked_spaces(&self, player_num: PlayerNum) -> u32 {
        self.get().iter().fold(0, |acc, row| {
            acc + row
                .iter()
                .filter(|s| s.is_ink(player_num))
                .fold(0, |acc, _| acc + 1)
        })
    }

    // Calculate the absolute board position for a given base position and offsets
    pub fn get_absolute_position(
        &self,
        // The number of spaces to the right of the board_x base position
        x_offset: usize,
        // The number of spaces down from the board_y base position
        y_offset: usize,
        board_x: i32,
        board_y: i32,
    ) -> Result<BoardPosition, BoardPositionError> {
        // The conversion errors don't provide any useful info, so we can discard them
        let x_offset: i32 = x_offset
            .try_into()
            .map_err(|_| BoardPositionError::OffsetToInt(Coordinate::X, x_offset))?;
        let y_offset: i32 = y_offset
            .try_into()
            .map_err(|_| BoardPositionError::OffsetToInt(Coordinate::Y, y_offset))?;
        let x = i32::checked_add(board_x, x_offset).ok_or(BoardPositionError::Overflow {
            coordinate: Coordinate::X,
            base: board_x,
            offset: x_offset,
        })?;
        let y = i32::checked_add(board_y, y_offset).ok_or(BoardPositionError::Overflow {
            coordinate: Coordinate::Y,
            base: board_y,
            offset: y_offset,
        })?;
        let x: usize = x
            .try_into()
            .map_err(|_| BoardPositionError::CoordinateToUsize(Coordinate::X, x))?;
        let y: usize = y
            .try_into()
            .map_err(|_| BoardPositionError::CoordinateToUsize(Coordinate::Y, y))?;
        BoardPosition::new(self, x, y)
    }
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
        let outside_row = BoardPosition::new(&board, 2, 0);
        assert!(outside_row.is_err());
        let outside_col = BoardPosition::new(&board, 0, 2);
        assert!(outside_col.is_err());
        let outside_row_and_col = BoardPosition::new(&board, 2, 2);
        assert!(outside_row_and_col.is_err());
        let valid_pos = BoardPosition::new(&board, 1, 1);
        assert!(valid_pos.is_ok());
    }

    #[test]
    fn test_surrounding_spaces() {
        let oob = BoardSpace::OutOfBounds;
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![vec![empty, empty], vec![empty, empty]]).unwrap();
        let spaces = BoardPosition::new(&board, 0, 0)
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
        let surrounded_pos = BoardPosition::new(&board, 0, 0).unwrap();
        assert!(surrounded_pos.is_surrounded(&board));
        let not_surrounded_pos = BoardPosition::new(&board, 1, 0).unwrap();
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
        let pos1 = BoardPosition::new(&board, 0, 0).unwrap();
        assert!(pos1.adjacent_to_ink(&board, PlayerNum::P1));
        let pos2 = BoardPosition::new(&board, 0, 2).unwrap();
        assert!(!pos2.adjacent_to_ink(&board, PlayerNum::P1));
        let pos3 = BoardPosition::new(&board, 2, 2).unwrap();
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
        let pos1 = BoardPosition::new(&board, 0, 0).unwrap();
        assert!(!pos1.adjacent_to_special(&board, PlayerNum::P1));
        let pos2 = BoardPosition::new(&board, 0, 2).unwrap();
        assert!(!pos2.adjacent_to_special(&board, PlayerNum::P1));
        let pos3 = BoardPosition::new(&board, 2, 2).unwrap();
        assert!(pos3.adjacent_to_special(&board, PlayerNum::P1));
        let pos4 = BoardPosition::new(&board, 1, 0).unwrap();
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
        assert_eq!(one_special[0].0.x(), 0);
        assert_eq!(one_special[0].0.y(), 0);

        let off_by_one_board =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, empty]]).unwrap();
        let no_special = off_by_one_board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert!(no_special.is_empty());

        let enemy_ink_board =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, p2_ink]]).unwrap();
        let one_special = enemy_ink_board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert_eq!(one_special.len(), 1);
        assert_eq!(one_special[0].0.x(), 0);
        assert_eq!(one_special[0].0.y(), 0);

        let multiple_specials_board =
            Board::new(vec![vec![p1_special, p1_ink], vec![p1_ink, p1_special]]).unwrap();
        let two_specials = multiple_specials_board.get_surrounded_inactive_specials(PlayerNum::P1);
        assert_eq!(two_specials.len(), 2);
        assert_eq!(two_specials[0].0.x(), 0);
        assert_eq!(two_specials[0].0.y(), 0);
        assert_eq!(two_specials[1].0.x(), 1);
        assert_eq!(two_specials[1].0.y(), 1);
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
        let valid_pos = board.get_absolute_position(7, 7, -7, -7);
        assert!(valid_pos.is_ok());
        assert_eq!(
            valid_pos.unwrap(),
            BoardPosition::new(&board, 0, 0).unwrap()
        );

        let invalid_pos = board.get_absolute_position(7, 7, -8, -8);
        assert!(invalid_pos.is_err());
    }
}
