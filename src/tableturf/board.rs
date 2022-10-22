use crate::tableturf::player::PlayerNum;

#[derive(Copy, Clone, Debug, PartialEq)]
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

fn is_surrounded(board_pos: BoardPosition, board: &Board) -> bool {
    surrounding_spaces(board_pos, board)
        .iter()
        .all(|s| !matches!(s, BoardSpace::Empty))
}

// Get the spaces surrounding the given position
pub fn surrounding_spaces(board_pos: BoardPosition, board: &Board) -> [BoardSpace; 8] {
    let x = board_pos.x() as i32;
    let y = board_pos.y() as i32;
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoardPosition(usize, usize);

impl BoardPosition {
    // Ensure that the given position meets the following criteria:
    // - x coordinate does not exceed Board row length
    // - y coordinate does not exceed Board column height
    // - x and y coordinates can be converted into i32s safely
    pub fn new(board: &Board, x: usize, y: usize) -> Option<Self> {
        if x >= board.get()[0].len() {
            return None;
        }
        if y >= board.get().len() {
            return None;
        }
        // Ensure that x and y can be safely converted into i32s later
        let _: i32 = x.try_into().ok()?;
        let _: i32 = y.try_into().ok()?;
        Some(BoardPosition(x, y))
    }

    pub fn x(&self) -> usize {
        self.0
    }

    pub fn y(&self) -> usize {
        self.1
    }
}

pub const MAX_BOARD_WIDTH: usize = 25;
pub const MAX_BOARD_HEIGHT: usize = 25;

// This cannot be an array, because custom boards might be loaded at runtime
#[derive(Debug, PartialEq)]
pub struct Board(Vec<Vec<BoardSpace>>);

impl Board {
    // Ensure that the given board meets the following criteria:
    // - all rows are the same length
    // - row length does not exceed the max board width
    // - column height does not exceed the max board height
    // - board contains at least one row
    // - rows contain at least one space
    pub fn new(spaces: Vec<Vec<BoardSpace>>) -> Option<Self> {
        if spaces.is_empty() {
            return None;
        }
        if spaces.len() > MAX_BOARD_HEIGHT {
            return None;
        }
        let row = spaces.get(0)?;
        if row.is_empty() {
            return None;
        }
        let row_len = row.len();
        if row_len > MAX_BOARD_WIDTH {
            return None;
        }
        if spaces.iter().any(|row| row.len() != row_len) {
            return None;
        }
        Some(Board(spaces))
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
    ) -> Vec<(usize, usize, BoardSpace)> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(y, r)| {
                r.iter()
                    .enumerate()
                    .filter(move |(x, s)| {
                        let board_pos = BoardPosition::new(self, *x, y).unwrap();
                        s.is_inactive_special(player_num) && is_surrounded(board_pos, self)
                    })
                    .map(move |(x, s)| (x, y, *s))
            })
            .collect()
    }

    pub fn set_ink(&mut self, ink_spaces: Vec<(BoardPosition, BoardSpace)>) {
        for (bp, s) in ink_spaces {
            (self.0)[bp.y() as usize][bp.x() as usize] = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_board() {
        let no_rows = Board::new(vec![]);
        assert!(no_rows.is_none());

        let empty_row = Board::new(vec![vec![]]);
        assert!(empty_row.is_none());

        let wall = BoardSpace::Wall;
        let empty = BoardSpace::Empty;
        let uneven_rows = Board::new(vec![vec![wall], vec![wall, wall]]);
        assert!(uneven_rows.is_none());

        let too_wide_board = Board::new(vec![vec![
            wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            wall, wall, wall, wall, wall, wall, wall, wall, wall, wall,
            wall, wall, wall, wall, wall, wall
        ]]);
        assert!(too_wide_board.is_none());

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
        ]);
        assert!(too_tall_board.is_none());

        let min_valid_board = Board::new(vec![vec![wall]]);
        assert!(min_valid_board.is_some());

        let valid_board = Board::new(vec![vec![wall, wall], vec![wall, empty]]);
        assert!(valid_board.is_some());

        let max_valid_board = Board::new(vec![
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
            vec![wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall, wall],
        ]);
        assert!(max_valid_board.is_some());
    }

    #[test]
    fn test_construct_board_position() {
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![vec![empty, empty], vec![empty, empty]]).unwrap();
        let outside_row = BoardPosition::new(&board, 2, 0);
        assert!(outside_row.is_none());
        let outside_col = BoardPosition::new(&board, 0, 2);
        assert!(outside_col.is_none());
        let outside_row_and_col = BoardPosition::new(&board, 2, 2);
        assert!(outside_row_and_col.is_none());
        let valid_pos = BoardPosition::new(&board, 1, 1);
        assert!(valid_pos.is_some());
    }
}
