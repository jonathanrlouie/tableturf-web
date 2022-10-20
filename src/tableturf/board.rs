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
    let x = board_pos.x();
    let y = board_pos.y();
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
pub struct BoardPosition(i32, i32);

impl BoardPosition {
    pub fn new(board: &Board, x: i32, y: i32) -> Option<Self> {
        if board.0.is_empty() {
            return None;
        }
        if x < 0 || x >= board.0[0].len().try_into().unwrap() {
            return None;
        }
        if y < 0 || y >= board.0.len().try_into().unwrap() {
            return None;
        }
        Some(BoardPosition(x, y))
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }
}

// This cannot be an array, because custom boards might be loaded at runtime
#[derive(Debug, PartialEq)]
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
                        let board_pos = BoardPosition::new(self, *x as i32, y as i32).unwrap();
                        s.is_inactive_special(player_num)
                            && is_surrounded(board_pos, self)
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

