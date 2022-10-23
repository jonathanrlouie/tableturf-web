use crate::tableturf::board::{self, Board, BoardPosition, BoardSpace};
use crate::tableturf::card::{Card, Grid, InkSpace, ROW_LEN};
use crate::tableturf::game_state::GameState;
use crate::tableturf::hand::HandIndex;
use crate::tableturf::player::PlayerNum;

// A card can be rotated in one of 4 different ways
#[derive(Copy, Clone)]
pub enum Rotation {
    Zero,
    One,
    Two,
    Three,
}

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

pub struct RawInput {
    pub hand_idx: usize,
    pub action: Action,
}

pub struct ValidInput {
    hand_idx: HandIndex,
    input: Input,
}

pub struct Placement {
    // Inked spaces with absolute board positions
    pub ink_spaces: Vec<(BoardPosition, InkSpace)>,
    pub special_activated: bool,
}

impl Placement {
    pub fn get_ink_spaces(self) -> Vec<(BoardPosition, InkSpace)> {
        self.ink_spaces
    }

    pub fn is_special_activated(&self) -> bool {
        self.special_activated
    }

    pub fn into_board_spaces(self, player_num: PlayerNum) -> Vec<(BoardPosition, BoardSpace)> {
        self.ink_spaces
            .iter()
            .map(|(bp, s)| (*bp, s.into_board_space(player_num)))
            .collect()
    }
}

pub enum Input {
    Pass,
    Place(Placement),
}

// Test if a single space is adjacent to a player's inked space
fn adjacent_to_ink(board_pos: BoardPosition, board: &Board, player_num: PlayerNum) -> bool {
    board::surrounding_spaces(board_pos, board)
        .iter()
        .any(|s| s.is_ink(player_num))
}

// Test if an entire placement of ink is adjacent to a player's inked space
fn placement_adjacent_to_ink(
    inked_spaces: &[(BoardPosition, InkSpace)],
    board: &Board,
    player_num: PlayerNum,
) -> bool {
    inked_spaces
        .iter()
        .any(|(bp, _)| adjacent_to_ink(*bp, board, player_num))
}

// Test if a single space is adjacent to a player's special space
fn adjacent_to_special(board_pos: BoardPosition, board: &Board, player_num: PlayerNum) -> bool {
    board::surrounding_spaces(board_pos, board)
        .iter()
        .any(|s| s.is_special(player_num))
}

// Test if an entire placement of ink is adjacent to one of the player's special spaces
fn placement_adjacent_to_special(
    inked_spaces: &[(BoardPosition, InkSpace)],
    board: &Board,
    player_num: PlayerNum,
) -> bool {
    inked_spaces
        .iter()
        .any(|(bp, _)| adjacent_to_special(*bp, board, player_num))
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

fn invalid_special_placement(
    selected_card: &Card,
    game_state: &GameState,
    ink_spaces: &[(BoardPosition, InkSpace)],
    player_num: PlayerNum,
) -> bool {
    let not_enough_special = game_state.players[player_num.idx()].special < selected_card.special;
    not_enough_special || special_collision(ink_spaces, &game_state.board)
}

fn get_absolute_position(
    board: &Board,
    x: usize,
    y: usize,
    board_x: i32,
    board_y: i32,
) -> Option<BoardPosition> {
    let x: i32 = x.try_into().ok()?;
    let y: i32 = y.try_into().ok()?;
    let x = i32::checked_add(board_x, x)?;
    let y = i32::checked_add(board_y, y)?;
    let x: usize = x.try_into().ok()?;
    let y: usize = y.try_into().ok()?;
    BoardPosition::new(board, x, y)
}

fn rotate_grid_ccw(grid: &mut Grid) {
    for i in 0..(ROW_LEN / 2) {
        for j in i..(ROW_LEN - i - 1) {
            let temp = grid[i][j];
            grid[i][j] = grid[j][ROW_LEN - 1 - i];
            grid[j][ROW_LEN - 1 - i] = grid[ROW_LEN - 1 - i][ROW_LEN - 1 - i];
            grid[ROW_LEN - 1 - i][ROW_LEN - 1 - j] = grid[ROW_LEN - 1 - j][i];
            grid[ROW_LEN - 1 - j][i] = temp;
        }
    }
}

fn rotate_input(card: &Card, rotation: Rotation) -> Grid {
    let mut grid = card.spaces;
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

impl ValidInput {
    // validates:
    // - board position
    // - card availability
    // - card index in hand
    // - special availability
    pub fn new(input: RawInput, game_state: &GameState, player_num: PlayerNum) -> Option<Self> {
        // Ensure given index is within the range of 0..4
        let hand_idx = HandIndex::new(input.hand_idx)?;

        // Validate that the user's selected card is available
        let player = &game_state.players[player_num.idx()];
        let selected_card_state = player.deck().get(player.hand().get(hand_idx));
        if !selected_card_state.is_available {
            return None;
        }
        let selected_card = selected_card_state.card;

        match input.action {
            Action::Pass => Some(Self {
                hand_idx,
                input: Input::Pass,
            }),
            Action::Place {
                x: board_x,
                y: board_y,
                special_activated,
                rotation,
            } => {
                let grid = rotate_input(&selected_card, rotation);
                let ink_spaces = grid
                    .iter()
                    .enumerate()
                    .flat_map(|(y, row)| {
                        row.iter()
                            .enumerate()
                            .filter_map(move |(x, cell)| cell.map(|s| (x, y, s)))
                    })
                    .map(|(x, y, s)| {
                        get_absolute_position(&game_state.board, x, y, board_x, board_y)
                            .map(|bp| (bp, s))
                    })
                    .collect::<Option<Vec<(BoardPosition, InkSpace)>>>()?;

                if special_activated {
                    // Check that player has enough special and that the special isn't
                    // overlapping any walls or special spaces.
                    if invalid_special_placement(
                        &selected_card,
                        game_state,
                        &ink_spaces[..],
                        player_num,
                    ) {
                        return None;
                    }

                    // Check that ink placement is adjacent to one of the player's special spaces
                    if !placement_adjacent_to_special(
                        &ink_spaces[..],
                        &game_state.board,
                        player_num,
                    ) {
                        return None;
                    }
                // Check that ink placement is over empty squares
                } else {
                    if placement_collision(&ink_spaces[..], &game_state.board) {
                        return None;
                    }

                    // Check that ink placement is adjacent to player's ink
                    if !placement_adjacent_to_ink(&ink_spaces[..], &game_state.board, player_num) {
                        return None;
                    }
                }

                Some(Self {
                    hand_idx,
                    input: Input::Place(Placement {
                        ink_spaces,
                        special_activated,
                    }),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {}
}
