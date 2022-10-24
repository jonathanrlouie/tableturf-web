use crate::tableturf::board::{self, Board, BoardPosition, BoardSpace};
use crate::tableturf::card::{Card, Grid, InkSpace, ROW_LEN};
use crate::tableturf::hand::HandIndex;
use crate::tableturf::player::{Player, PlayerNum};


// Represents the number of counter-clockwise rotations applied to a Card
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
    ink_spaces: Vec<(BoardPosition, InkSpace)>,
    special_activated: bool,
}

impl Placement {
    pub fn new(
        ink_spaces: Vec<(BoardPosition, InkSpace)>,
        special_activated: bool,
        hand_idx: HandIndex,
        board: &Board,
        player: &Player,
        player_num: PlayerNum,
    ) -> Option<Placement> {
        let selected_card = player.get_card(hand_idx);

        if special_activated {
            // Check that player has enough special and that the special isn't
            // overlapping any walls or special spaces.
            if invalid_special_placement(
                &selected_card,
                board,
                player,
                &ink_spaces[..],
                player_num,
            ) {
                return None;
            }

            // Check that ink placement is adjacent to one of the player's special spaces
            if !placement_adjacent_to_special(
                &ink_spaces[..],
                board,
                player_num,
            ) {
                return None;
            }
        // Check that ink placement is over empty squares
        } else {
            if placement_collision(&ink_spaces[..], board) {
                return None;
            }

            // Check that ink placement is adjacent to player's ink
            if !placement_adjacent_to_ink(&ink_spaces[..], board, player_num) {
                return None;
            }
        }
        Some(Placement {
            ink_spaces,
            special_activated
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

fn into_board_space(ink_space: &InkSpace, player_num: PlayerNum) -> BoardSpace {
    match ink_space {
        InkSpace::Normal => BoardSpace::Ink { player_num },
        InkSpace::Special => BoardSpace::Special {
            player_num,
            is_activated: false,
        },
    }
}

pub enum Input {
    Pass,
    Place(Placement),
}

impl ValidInput {
    // validates:
    // - board position
    // - card index in hand
    // - special availability
    pub fn new(input: RawInput, board: &Board, player: &Player, player_num: PlayerNum) -> Option<Self> {
        // Ensure given index is within the range of 0..4
        let hand_idx = HandIndex::new(input.hand_idx)?;

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
                let selected_card = player.get_card(hand_idx);
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
                        board.get_absolute_position(x, y, board_x, board_y)
                            .map(|bp| (bp, s))
                    })
                    .collect::<Option<Vec<(BoardPosition, InkSpace)>>>()?;

                let placement = Placement::new(
                    ink_spaces,
                    special_activated,
                    hand_idx,
                    board,
                    player,
                    player_num,
                )?;

                Some(Self {
                    hand_idx,
                    input: Input::Place(placement)
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

fn invalid_special_placement(
    selected_card: &Card,
    board: &Board,
    player: &Player,
    ink_spaces: &[(BoardPosition, InkSpace)],
    player_num: PlayerNum,
) -> bool {
    let not_enough_special = player.special < selected_card.special();
    not_enough_special || special_collision(ink_spaces, board)
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
    use crate::tableturf::card::{InkSpace, CardSpace, Card};
    use crate::tableturf::board::Board;

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
        assert_eq!(zero_rotations, [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, i, i, s, e, e, e],
            [e, e, i, i, i, i, e, e],
            [e, e, i, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ]);

        let one_rotation = rotate_input(&splattershot, Rotation::One);
        assert_eq!(one_rotation, [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, i, e, e, e, e],
            [e, e, s, i, e, e, e, e],
            [e, e, i, i, e, e, e, e],
            [e, e, i, i, i, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ]);

        let two_rotations = rotate_input(&splattershot, Rotation::Two);
        assert_eq!(two_rotations, [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, i, e, e],
            [e, e, i, i, i, i, e, e],
            [e, e, e, s, i, i, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ]);

        let three_rotations = rotate_input(&splattershot, Rotation::Three);
        assert_eq!(three_rotations, [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, i, i, i, e, e],
            [e, e, e, e, i, i, e, e],
            [e, e, e, e, i, s, e, e],
            [e, e, e, e, i, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ]);
    }
}
