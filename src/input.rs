use crate::tableturf::*;

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
    pub card_idx: usize,
    pub action: Action,
}

pub struct ValidInput {
    card_idx: usize,
    input: Input,
}

pub enum Input {
    Pass,
    Place(Placement),
}

// Test if a single space is adjacent to a player's inked space
fn adjacent_to_ink(x: i32, y: i32, board: &Board, player_num: PlayerNum) -> bool {
    surrounding_spaces(x, y, board)
        .iter()
        .any(|s| s.is_ink(player_num))
}

// Test if an entire placement of ink is adjacent to a player's inked space
fn placement_adjacent_to_ink(
    inked_spaces: &[(i32, i32, InkSpace)],
    board: &Board,
    player_num: PlayerNum,
) -> bool {
    inked_spaces
        .iter()
        .any(|(x, y, _)| adjacent_to_ink(*x, *y, board, player_num))
}

// Test if a single space is adjacent to a player's special space
fn adjacent_to_special(x: i32, y: i32, board: &Board, player_num: PlayerNum) -> bool {
    surrounding_spaces(x, y, board)
        .iter()
        .any(|s| s.is_special(player_num))
}

// Test if an entire placement of ink is adjacent to one of the player's special spaces
fn placement_adjacent_to_special(
    inked_spaces: &[(i32, i32, InkSpace)],
    board: &Board,
    player_num: PlayerNum,
) -> bool {
    inked_spaces
        .iter()
        .any(|(x, y, _)| adjacent_to_special(*x, *y, board, player_num))
}

// Test if an entire placement of ink overlaps ink or walls
fn placement_collision(inked_spaces: &[(i32, i32, InkSpace)], board: &Board) -> bool {
    inked_spaces
        .iter()
        .any(|(x, y, _)| !matches!(board.get_space(*x, *y), BoardSpace::Empty))
}

// Test if an entire special placement of ink overlaps any special spaces or walls
fn special_collision(inked_spaces: &[(i32, i32, InkSpace)], board: &Board) -> bool {
    inked_spaces.iter().any(|(x, y, _)| {
        let board_space = board.get_space(*x, *y);
        matches!(board_space, BoardSpace::Special { .. })
            || matches!(board_space, BoardSpace::Wall)
            || matches!(board_space, BoardSpace::OutOfBounds)
    })
}

fn invalid_special_placement(
    selected_card: &Card,
    game_state: &GameState,
    ink_spaces: &[(i32, i32, InkSpace)],
    player_num: PlayerNum,
) -> bool {
    let not_enough_special = game_state.players[player_num].special < selected_card.special;
    not_enough_special || special_collision(ink_spaces, &game_state.board)
}

fn get_absolute_position(x: usize, y: usize, board_x: i32, board_y: i32) -> Option<(i32, i32)> {
    let x: i32 = x.try_into().ok()?;
    let x = i32::checked_add(board_x, x)?;
    let y: i32 = y.try_into().ok()?;
    let y = i32::checked_add(board_y, y)?;
    Some((x, y))
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
        if input.card_idx >= HAND_SIZE {
            return None;
        };

        // Validate that the user's selected card is available
        let player = &game_state.players[player_num];
        let selected_card = player.hand[input.card_idx];
        player
            .deck
            .iter()
            .find(|c| **c == CardState::Available(selected_card))?;

        match input.action {
            Action::Pass => Some(Self {
                card_idx: input.card_idx,
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
                        get_absolute_position(x, y, board_x, board_y)
                            .map(|(int_x, int_y)| (int_x, int_y, s))
                    })
                    .collect::<Option<Vec<(i32, i32, InkSpace)>>>()?;

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
                    if !placement_adjacent_to_special(&ink_spaces[..], &game_state.board, player_num) {
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
                    card_idx: input.card_idx,
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

    pub fn card_idx(&self) -> usize {
        self.card_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
