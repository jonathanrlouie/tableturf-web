use crate::tableturf::board::{Board, BoardPosition, BoardSpace};
use crate::tableturf::card::InkSpace;
use crate::tableturf::hand::HandIndex;
use crate::tableturf::deck::DrawRng;
use crate::tableturf::input::{Input, Placement, ValidInput};
use crate::tableturf::player::{Player, PlayerNum};
use std::cmp::Ordering;
use rand::rngs::ThreadRng;
use rand::prelude::IteratorRandom;

struct DeckRng {
    rng: ThreadRng
}

impl DrawRng for DeckRng {
    fn draw<T, I: Iterator<Item=T> + Sized>(&mut self, iter: I) -> Option<T> {
        iter.choose(&mut self.rng)
    }
}

enum Outcome {
    P1Win,
    P2Win,
    Draw,
}

pub struct GameState {
    board: Board,
    players: [Player; 2],
    turns_left: u32,
}

impl GameState {
    pub fn new(board: Board, players: [Player; 2], turns_left: u32) -> Self {
        GameState {
            board,
            players,
            turns_left
        }
    }

    pub fn check_winner(&self) -> Outcome {
        let p1_inked_spaces = count_inked_spaces(&self.board, PlayerNum::P1);
        let p2_inked_spaces = count_inked_spaces(&self.board, PlayerNum::P2);

        match p1_inked_spaces.cmp(&p2_inked_spaces) {
            Ordering::Greater => Outcome::P1Win,
            Ordering::Less => Outcome::P2Win,
            Ordering::Equal => Outcome::Draw,
        }
    }

    // input1: player 1's input
    // input2: player 2's input
    pub fn update_game_state<R: DrawRng>(&mut self, rng: &mut R, input1: ValidInput, input2: ValidInput) {
        let hand_idx1 = input1.hand_idx();
        let hand_idx2 = input2.hand_idx();
        match (input1.get(), input2.get()) {
            (Input::Pass, Input::Pass) => {
                self.players[0].special += 1;
                self.players[1].special += 1;
            }
            (Input::Place(placement), Input::Pass) => {
                self.players[1].special += 1;
                self.place(hand_idx1, placement, PlayerNum::P1);
            }
            (Input::Pass, Input::Place(placement)) => {
                self.players[0].special += 1;
                self.place(hand_idx2, placement, PlayerNum::P2);
            }
            (Input::Place(placement1), Input::Place(placement2)) => {
                self.place_both(hand_idx1, hand_idx2, placement1, placement2);
            }
        };
        let player1 = &mut self.players[0];
        player1.replace_card(hand_idx1, rng);
        update_special_gauge(player1, PlayerNum::P1, &mut self.board);
        let player2 = &mut self.players[1];
        player2.replace_card(hand_idx2, rng);
        update_special_gauge(player2, PlayerNum::P2, &mut self.board);

        if self.turns_left > 0 {
            self.turns_left -= 1;
        }
    }

    fn place(&mut self, hand_idx: HandIndex, placement: Placement, player_num: PlayerNum) {
        let player = &mut self.players[player_num.idx()];
        player.spend_special(&placement, hand_idx);
        self.board.set_ink(placement.into_board_spaces(player_num));
    }

    fn place_both(
        &mut self,
        hand_idx1: HandIndex,
        hand_idx2: HandIndex,
        placement1: Placement,
        placement2: Placement,
    ) {
        // Spend special, if activated
        let player1 = &mut self.players[0];
        let priority1 = player1.deck().get(player1.hand().get(hand_idx1)).priority();
        player1.spend_special(&placement1, hand_idx1);
        let player2 = &mut self.players[1];
        let priority2 = player2.deck().get(player2.hand().get(hand_idx2)).priority();
        player2.spend_special(&placement2, hand_idx2);

        let overlap: Vec<(BoardPosition, InkSpace, InkSpace)> = placement1
            .ink_spaces()
            .iter()
            .filter_map(|(bp1, s1)| {
                placement2
                    .ink_spaces()
                    .iter()
                    .find(|&&(bp2, _)| bp1.x() == bp2.x() && bp1.y() == bp2.y())
                    .map(|(_, s2)| (*bp1, *s1, *s2))
            })
            .collect();

        if !overlap.is_empty() {
            let overlap_resolved = match priority1.cmp(&priority2) {
                Ordering::Greater => resolve_overlap(
                    overlap,
                    BoardSpace::Ink {
                        player_num: PlayerNum::P2,
                    },
                    BoardSpace::Special {
                        player_num: PlayerNum::P2,
                        is_activated: false,
                    },
                ),
                Ordering::Less => resolve_overlap(
                    overlap,
                    BoardSpace::Ink {
                        player_num: PlayerNum::P1,
                    },
                    BoardSpace::Special {
                        player_num: PlayerNum::P1,
                        is_activated: false,
                    },
                ),
                Ordering::Equal => resolve_overlap(overlap, BoardSpace::Wall, BoardSpace::Wall),
            };
            // No need to try to find parts that don't overlap as long as
            // we set the overlapping ink last
            self.board
                .set_ink(placement1.into_board_spaces(PlayerNum::P1));
            self.board
                .set_ink(placement2.into_board_spaces(PlayerNum::P2));
            self.board.set_ink(overlap_resolved);
        } else {
            self.board
                .set_ink(placement1.into_board_spaces(PlayerNum::P1));
            self.board
                .set_ink(placement2.into_board_spaces(PlayerNum::P2));
        }
    }
}

fn count_inked_spaces(board: &Board, player_num: PlayerNum) -> u32 {
    board.get().iter().fold(0, |acc, row| {
        acc + row
            .iter()
            .filter(|s| s.is_ink(player_num))
            .fold(0, |acc, _| acc + 1)
    })
}

// normal_collision_space is the space used when an ink space conflicts with an ink space
// special_collision_space is the space used when a special space conflicts with a special space
fn resolve_overlap(
    overlap: Vec<(BoardPosition, InkSpace, InkSpace)>,
    normal_collision_space: BoardSpace,
    special_collision_space: BoardSpace,
) -> Vec<(BoardPosition, BoardSpace)> {
    overlap
        .iter()
        .map(|(bp, s1, s2)| {
            (
                *bp,
                match (s1, s2) {
                    (InkSpace::Normal, InkSpace::Normal) => normal_collision_space,
                    (InkSpace::Special, InkSpace::Normal) => BoardSpace::Special {
                        player_num: PlayerNum::P1,
                        is_activated: false,
                    },
                    (InkSpace::Normal, InkSpace::Special) => BoardSpace::Special {
                        player_num: PlayerNum::P2,
                        is_activated: false,
                    },
                    (InkSpace::Special, InkSpace::Special) => special_collision_space,
                },
            )
        })
        .collect::<Vec<(BoardPosition, BoardSpace)>>()
}


fn update_special_gauge(player: &mut Player, player_num: PlayerNum, board: &mut Board) {
    let special_spaces = board.get_surrounded_inactive_specials(player_num);
    // activate surrounded special spaces
    for (bp, _) in &special_spaces {
        board.get_mut()[bp.y()][bp.x()] = BoardSpace::Special {
            player_num,
            is_activated: true,
        }
    }
    player.special += special_spaces.len() as u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::board::{self, Board, BoardPosition};
    use crate::tableturf::card::{Card, CardSpace, CardState};
    use crate::tableturf::deck::{Deck, DeckIndex};
    use crate::tableturf::hand::{Hand, HandIndex};

    fn board_pos(board: &Board, x: usize, y: usize) -> BoardPosition {
        BoardPosition::new(board, x, y).unwrap()
    }

    fn card_states() -> [CardState; 15] {
        let e: CardSpace = None;
        let i: CardSpace = Some(InkSpace::Normal);
        let s: CardSpace = Some(InkSpace::Special);
        [
            // Splattershot
            CardState::new(
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
                true,
            ),
            // Slosher
            CardState::new(
                Card::new(
                    6,
                    [
                        [e, e, e, e, e, e, e, e],
                        [e, e, e, e, e, e, e, e],
                        [e, e, i, e, e, e, e, e],
                        [e, e, e, s, i, e, e, e],
                        [e, e, i, i, i, e, e, e],
                        [e, e, e, e, e, e, e, e],
                        [e, e, e, e, e, e, e, e],
                        [e, e, e, e, e, e, e, e],
                    ],
                    3,
                ),
                true,
            ),
            // Zapfish
            CardState::new(
                Card::new(
                    9,
                    [
                        [e, e, e, e, e, e, e, e],
                        [e, e, e, e, e, e, e, e],
                        [e, e, e, e, e, i, e, e],
                        [e, e, e, i, i, e, e, e],
                        [e, e, e, i, s, i, e, e],
                        [e, e, i, e, i, i, e, e],
                        [e, e, e, e, e, e, e, e],
                        [e, e, e, e, e, e, e, e],
                    ],
                    4,
                ),
                true,
            ),
            // Blaster
            CardState::new(
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
                true,
            ),
            // Splat Dualies
            CardState::new(
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
                true,
            ),
            // Flooder
            CardState::new(
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
                true,
            ),
            // Splat Roller
            CardState::new(
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
                true,
            ),
            // Tri-Stringer
            CardState::new(
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
                true,
            ),
            // Chum
            CardState::new(
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
                true,
            ),
            // Splat Charger
            CardState::new(
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
                true,
            ),
            // Splatana Wiper
            CardState::new(
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
                true,
            ),
            // SquidForce
            CardState::new(
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
                true,
            ),
            // Heavy Splatling
            CardState::new(
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
                true,
            ),
            // Splat Bomb
            CardState::new(
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
                true,
            ),
            // Marigold
            CardState::new(
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
                true,
            ),
        ]
    }

    fn default_deck1() -> Deck {
        let mut card_states = card_states();
        card_states[0].is_available = false;
        card_states[1].is_available = false;
        card_states[2].is_available = false;
        card_states[3].is_available = false;
        Deck::new(card_states)
    }

    fn default_deck2() -> Deck {
        let mut card_states = card_states();
        card_states[11].is_available = false;
        card_states[12].is_available = false;
        card_states[13].is_available = false;
        card_states[14].is_available = false;
        Deck::new(card_states)
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
        let player1_ink_total = count_inked_spaces(&board, PlayerNum::P1);
        let player2_ink_total = count_inked_spaces(&board, PlayerNum::P2);
        assert_eq!(player1_ink_total, 3);
        assert_eq!(player2_ink_total, 2);
    }

    #[test]
    fn test_surrounding_spaces() {
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
        let oob = BoardSpace::OutOfBounds;
        let board = Board::new(vec![
            vec![empty, p1_ink, p2_ink],
            vec![empty, wall, p1_special],
            vec![p2_special, empty, p1_ink],
        ])
        .unwrap();
        let spaces = board::surrounding_spaces(board_pos(&board, 1, 1), &board);
        assert_eq!(spaces[0], empty);
        assert_eq!(spaces[1], p1_ink);
        assert_eq!(spaces[2], p2_ink);
        assert_eq!(spaces[3], empty);
        assert_eq!(spaces[4], p1_special);
        assert_eq!(spaces[5], p2_special);
        assert_eq!(spaces[6], empty);
        assert_eq!(spaces[7], p1_ink);

        let spaces = board::surrounding_spaces(board_pos(&board, 0, 0), &board);
        assert_eq!(spaces[0], oob);
        assert_eq!(spaces[1], oob);
        assert_eq!(spaces[2], oob);
        assert_eq!(spaces[3], oob);
        assert_eq!(spaces[4], p1_ink);
        assert_eq!(spaces[5], oob);
        assert_eq!(spaces[6], empty);
        assert_eq!(spaces[7], wall);
    }

    #[test]
    fn test_place() {
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
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let player1 = Player::new(
            Hand([
                DeckIndex::new(0).unwrap(),
                DeckIndex::new(1).unwrap(),
                DeckIndex::new(2).unwrap(),
                DeckIndex::new(3).unwrap(),
            ]),
            default_deck1(),
            0,
        ).unwrap();

        let player2 = Player::new(
            Hand([
                DeckIndex::new(11).unwrap(),
                DeckIndex::new(12).unwrap(),
                DeckIndex::new(13).unwrap(),
                DeckIndex::new(14).unwrap(),
            ]),
            default_deck2(),
            0,
        ).unwrap();

        let mut game_state = GameState::new(
            board,
            [player1, player2],
            12,
        );

        game_state.place(
            HandIndex::new(0).unwrap(),
            Placement::new(
                vec![
                    (board_pos(&game_state.board, 0, 0), InkSpace::Normal),
                    (board_pos(&game_state.board, 1, 0), InkSpace::Normal),
                    (board_pos(&game_state.board, 2, 0), InkSpace::Special),
                    (board_pos(&game_state.board, 0, 1), InkSpace::Normal),
                    (board_pos(&game_state.board, 1, 1), InkSpace::Normal),
                    (board_pos(&game_state.board, 2, 1), InkSpace::Normal),
                    (board_pos(&game_state.board, 3, 1), InkSpace::Normal),
                    (board_pos(&game_state.board, 0, 2), InkSpace::Normal),
                ],
                false,
            ),
            PlayerNum::P1,
        );

        let expected_board = Board::new(vec![
            vec![p1_ink, p1_ink, p1_special, empty],
            vec![p1_ink, p1_ink, p1_ink, p1_ink],
            vec![p1_ink, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state.board, expected_board);
    }

    fn game_state1() -> GameState {
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let player1 = Player::new(
            Hand([
                DeckIndex::new(0).unwrap(),
                DeckIndex::new(1).unwrap(),
                DeckIndex::new(2).unwrap(),
                DeckIndex::new(3).unwrap(),
            ]),
            default_deck1(),
            0,
        ).unwrap();

        let player2 = Player::new(
            Hand([
                DeckIndex::new(0).unwrap(),
                DeckIndex::new(1).unwrap(),
                DeckIndex::new(2).unwrap(),
                DeckIndex::new(3).unwrap(),
            ]),
            default_deck1(),
            0,
        ).unwrap();

        GameState::new(
            board,
            [player1, player2],
            12,
        )
    }

    fn game_state2() -> GameState {
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let player1 = Player::new(
            Hand([
                DeckIndex::new(0).unwrap(),
                DeckIndex::new(1).unwrap(),
                DeckIndex::new(2).unwrap(),
                DeckIndex::new(3).unwrap(),
            ]),
            default_deck1(),
            0,
        ).unwrap();

        let mut card_states = card_states();
        card_states[13].is_available = false;
        card_states[1].is_available = false;
        card_states[2].is_available = false;
        card_states[3].is_available = false;
        let player2 = Player::new(
            Hand([
                DeckIndex::new(13).unwrap(),
                DeckIndex::new(1).unwrap(),
                DeckIndex::new(2).unwrap(),
                DeckIndex::new(3).unwrap(),
            ]),
            Deck::new(card_states),
            0,
        ).unwrap();

        GameState {
            board,
            players: [player1, player2],
            turns_left: 12,
        }
    }

    #[test]
    fn test_place_both() {
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let p2_special = BoardSpace::Special {
            player_num: PlayerNum::P2,
            is_activated: false,
        };
        let empty = BoardSpace::Empty;
        let wall = BoardSpace::Wall;

        let mut game_state1 = game_state1();
        let board1 = &game_state1.board;

        game_state1.place_both(
            HandIndex::new(0).unwrap(),
            HandIndex::new(0).unwrap(),
            Placement::new(
                vec![
                    (board_pos(board1, 0, 0), InkSpace::Normal),
                    (board_pos(board1, 1, 0), InkSpace::Normal),
                    (board_pos(board1, 2, 0), InkSpace::Special),
                    (board_pos(board1, 0, 1), InkSpace::Normal),
                    (board_pos(board1, 1, 1), InkSpace::Normal),
                    (board_pos(board1, 2, 1), InkSpace::Normal),
                    (board_pos(board1, 3, 1), InkSpace::Normal),
                    (board_pos(board1, 0, 2), InkSpace::Normal),
                ],
                false,
            ),
            Placement::new(
                vec![
                    (board_pos(board1, 0, 0), InkSpace::Normal),
                    (board_pos(board1, 1, 0), InkSpace::Normal),
                    (board_pos(board1, 2, 0), InkSpace::Special),
                    (board_pos(board1, 0, 1), InkSpace::Normal),
                    (board_pos(board1, 1, 1), InkSpace::Normal),
                    (board_pos(board1, 2, 1), InkSpace::Normal),
                    (board_pos(board1, 3, 1), InkSpace::Normal),
                    (board_pos(board1, 0, 2), InkSpace::Normal),
                ],
                false,
            ),
        );

        let expected_board1 = Board::new(vec![
            vec![wall, wall, wall, empty],
            vec![wall, wall, wall, wall],
            vec![wall, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state1.board, expected_board1);

        let mut game_state2 = game_state2();
        game_state2.place_both(
            HandIndex::new(0).unwrap(),
            HandIndex::new(0).unwrap(),
            Placement::new(
                vec![
                    (board_pos(&game_state2.board, 0, 0), InkSpace::Normal),
                    (board_pos(&game_state2.board, 1, 0), InkSpace::Normal),
                    (board_pos(&game_state2.board, 2, 0), InkSpace::Special),
                    (board_pos(&game_state2.board, 0, 1), InkSpace::Normal),
                    (board_pos(&game_state2.board, 1, 1), InkSpace::Normal),
                    (board_pos(&game_state2.board, 2, 1), InkSpace::Normal),
                    (board_pos(&game_state2.board, 3, 1), InkSpace::Normal),
                    (board_pos(&game_state2.board, 0, 2), InkSpace::Normal),
                ],
                false,
            ),
            Placement::new(
                vec![
                    (board_pos(&game_state2.board, 1, 1), InkSpace::Normal),
                    (board_pos(&game_state2.board, 2, 1), InkSpace::Normal),
                    (board_pos(&game_state2.board, 2, 0), InkSpace::Special),
                ],
                false,
            ),
        );

        let expected_board2 = Board::new(vec![
            vec![p1_ink, p1_ink, p2_special, empty],
            vec![p1_ink, p2_ink, p2_ink, p1_ink],
            vec![p1_ink, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state2.board, expected_board2);
    }

    #[test]
    fn test_resolve_overlap() {
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        let overlap = vec![
            (board_pos(&board, 0, 0), InkSpace::Normal, InkSpace::Normal),
            (
                board_pos(&board, 1, 0),
                InkSpace::Special,
                InkSpace::Special,
            ),
            (board_pos(&board, 2, 0), InkSpace::Special, InkSpace::Normal),
            (board_pos(&board, 0, 1), InkSpace::Normal, InkSpace::Special),
        ];
        let result = resolve_overlap(overlap, BoardSpace::Wall, BoardSpace::Wall);
        let expected = vec![
            (board_pos(&board, 0, 0), BoardSpace::Wall),
            (board_pos(&board, 1, 0), BoardSpace::Wall),
            (
                board_pos(&board, 2, 0),
                BoardSpace::Special {
                    player_num: PlayerNum::P1,
                    is_activated: false,
                },
            ),
            (
                board_pos(&board, 0, 1),
                BoardSpace::Special {
                    player_num: PlayerNum::P2,
                    is_activated: false,
                },
            ),
        ];
        assert!(result == expected);
    }
}
