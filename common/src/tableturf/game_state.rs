use crate::tableturf::board::{Board, BoardPosition, BoardSpace};
use crate::tableturf::card::{Card, CardSpace, InkSpace};
use crate::tableturf::deck::{Deck, DeckIndex, DrawRng, Hand, HandIndex};
use crate::tableturf::input::{Input, Placement, ValidInput};
use crate::tableturf::player::{Player, PlayerNum, Players};
use rand::prelude::IteratorRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::cmp::Ordering;
use std::fmt::Debug;

#[derive(Debug)]
pub struct DeckRng {
    rng: StdRng,
}

impl Default for DeckRng {
    fn default() -> Self {
        let rng = StdRng::from_rng(rand::thread_rng()).unwrap();
        DeckRng { rng }
    }
}

impl DrawRng for DeckRng {
    fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, iter: I) -> Option<T> {
        iter.choose(&mut self.rng)
    }

    fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Hand {
        let cards = iter.choose_multiple(&mut self.rng, 4);
        Hand::new([cards[0], cards[1], cards[2], cards[3]]).unwrap()
    }
}

#[derive(Debug)]
pub enum Outcome {
    P1Win,
    P2Win,
    Draw,
}

#[derive(Debug)]
pub struct GameState<R: Debug> {
    board: Board,
    players: Players,
    turns_left: u32,
    rng: R,
}

fn default_deck() -> [Card; 15] {
    let e: CardSpace = None;
    let i: CardSpace = Some(InkSpace::Normal);
    let s: CardSpace = Some(InkSpace::Special);
    [
        Card::new(
            "Splattershot".to_string(),
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
        Card::new(
            "Slosher".to_string(),
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
        Card::new(
            "Zapfish".to_string(),
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
        Card::new(
            "Blaster".to_string(),
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
        Card::new(
            "Splat Dualies".to_string(),
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
        Card::new(
            "Flooder".to_string(),
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
        Card::new(
            "Splat Roller".to_string(),
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
        Card::new(
            "Tri-Stringer".to_string(),
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
        Card::new(
            "Chum".to_string(),
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
        Card::new(
            "Splat Charger".to_string(),
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
        Card::new(
            "Splatana Wiper".to_string(),
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
        Card::new(
            "SquidForce".to_string(),
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
        Card::new(
            "Heavy Splatling".to_string(),
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
        Card::new(
            "Splat Bomb".to_string(),
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
        Card::new(
            "Marigold".to_string(),
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

impl<R: DrawRng + Default + Debug> Default for GameState<R> {
    fn default() -> Self {
        //let rng = StdRng::from_rng(rand::thread_rng()).unwrap();
        let mut rng = R::default();
        let ee = BoardSpace::Empty;
        let s1 = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let s2 = BoardSpace::Special {
            player_num: PlayerNum::P2,
            is_activated: false,
        };
        let board = Board::new(vec![
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, s2, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, s1, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
            vec![ee, ee, ee, ee, ee, ee, ee, ee, ee],
        ])
        .unwrap();
        let (deck1, hand1) = Deck::draw_hand(default_deck(), &mut rng);
        let (deck2, hand2) = Deck::draw_hand(default_deck(), &mut rng);

        let players = [
            Player::new(hand1, deck1, PlayerNum::P1, 0),
            Player::new(hand2, deck2, PlayerNum::P2, 0),
        ];
        GameState::new(board, players, 12, rng)
    }
}

impl<R: DrawRng + Debug> GameState<R> {
    pub fn new(board: Board, players: [Player; 2], turns_left: u32, rng: R) -> Self {
        GameState {
            board,
            players: Players::new(players),
            turns_left,
            rng,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn player(&self, player_num: PlayerNum) -> &Player {
        &self.players[player_num]
    }

    pub fn turns_left(&self) -> u32 {
        self.turns_left
    }

    pub fn check_winner(&self) -> Outcome {
        let p1_inked_spaces = self.board.count_inked_spaces(PlayerNum::P1);
        let p2_inked_spaces = self.board.count_inked_spaces(PlayerNum::P2);

        match p1_inked_spaces.cmp(&p2_inked_spaces) {
            Ordering::Greater => Outcome::P1Win,
            Ordering::Less => Outcome::P2Win,
            Ordering::Equal => Outcome::Draw,
        }
    }

    pub fn redraw_hand(&mut self, player_num: PlayerNum) {
        self.players[player_num].redraw_hand(&mut self.rng);
    }

    // input1: player 1's input
    // input2: player 2's input
    pub fn update(&mut self, input1: ValidInput, input2: ValidInput) {
        let hand_idx1 = input1.hand_idx();
        let hand_idx2 = input2.hand_idx();
        match (input1.get(), input2.get()) {
            (Input::Pass, Input::Pass) => {
                self.players[PlayerNum::P1].special += 1;
                self.players[PlayerNum::P2].special += 1;
            }
            (Input::Place(placement), Input::Pass) => {
                self.players[PlayerNum::P2].special += 1;
                self.place(hand_idx1, placement, PlayerNum::P1);
            }
            (Input::Pass, Input::Place(placement)) => {
                self.players[PlayerNum::P1].special += 1;
                self.place(hand_idx2, placement, PlayerNum::P2);
            }
            (Input::Place(placement1), Input::Place(placement2)) => {
                self.place_both(hand_idx1, hand_idx2, placement1, placement2);
            }
        };
        let player1 = &mut self.players[PlayerNum::P1];
        player1.replace_card(hand_idx1, &mut self.rng);
        update_special_gauge(player1, PlayerNum::P1, &mut self.board);
        let player2 = &mut self.players[PlayerNum::P2];
        player2.replace_card(hand_idx2, &mut self.rng);
        update_special_gauge(player2, PlayerNum::P2, &mut self.board);

        if self.turns_left > 0 {
            self.turns_left -= 1;
        }
    }

    fn place(&mut self, hand_idx: HandIndex, placement: Placement, player_num: PlayerNum) {
        let player = &mut self.players[player_num];
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
        let player1 = &mut self.players[PlayerNum::P1];
        let priority1 = player1.deck().index(player1.hand()[hand_idx1]).0.priority();
        player1.spend_special(&placement1, hand_idx1);
        let player2 = &mut self.players[PlayerNum::P2];
        let priority2 = player2.deck().index(player2.hand()[hand_idx2]).0.priority();
        player2.spend_special(&placement2, hand_idx2);

        let overlap: Vec<(BoardPosition, InkSpace, InkSpace)> = placement1
            .ink_spaces()
            .0
            .iter()
            .filter_map(|(bp1, s1)| {
                placement2
                    .ink_spaces()
                    .0
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
        board.set_space(
            bp,
            BoardSpace::Special {
                player_num,
                is_activated: true,
            },
        );
    }
    player.special += special_spaces.len() as u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableturf::board::{Board, BoardPosition};
    use crate::tableturf::deck::{Deck, DeckIndex, Hand, HandIndex};
    use crate::tableturf::input::{Action, Placement, RawInput, RawPlacement, Rotation};

    #[derive(Debug)]
    struct MockRng1;
    #[derive(Debug)]
    struct MockRng2;
    #[derive(Debug)]
    struct MockRng3;

    impl DrawRng for MockRng1 {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Hand {
            let v: Vec<DeckIndex> = iter.collect();
            Hand::new([v[0], v[1], v[2], v[3]]).unwrap()
        }
    }

    impl DrawRng for MockRng2 {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Hand {
            let v: Vec<DeckIndex> = iter.collect();
            Hand::new([v[11], v[12], v[13], v[14]]).unwrap()
        }
    }

    impl DrawRng for MockRng3 {
        fn draw<T, I: Iterator<Item = T> + Sized>(&mut self, mut iter: I) -> Option<T> {
            iter.next()
        }

        fn draw_hand<I: Iterator<Item = DeckIndex> + Sized>(&mut self, iter: I) -> Hand {
            let v: Vec<DeckIndex> = iter.collect();
            Hand::new([v[13], v[1], v[2], v[3]]).unwrap()
        }
    }

    fn board_pos(board: &Board, x: usize, y: usize) -> BoardPosition {
        BoardPosition::new(board, x, y).unwrap()
    }

    fn draw_hand1() -> (Deck, Hand) {
        Deck::draw_hand(default_deck(), &mut MockRng1)
    }

    fn draw_hand2() -> (Deck, Hand) {
        Deck::draw_hand(default_deck(), &mut MockRng2)
    }

    fn game_state1() -> GameState<MockRng1> {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, p1_ink, p2_ink, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let (deck1, hand1) = draw_hand1();
        let player1 = Player::new(hand1, deck1, PlayerNum::P1, 0);

        let (deck1, hand1) = draw_hand1();
        let player2 = Player::new(hand1, deck1, PlayerNum::P2, 0);

        GameState::new(board, [player1, player2], 12, MockRng1)
    }

    fn game_state2() -> GameState<MockRng2> {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, p1_ink, p2_ink, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let (deck1, hand1) = draw_hand1();
        let player1 = Player::new(hand1, deck1, PlayerNum::P1, 0);

        let (deck2, hand2) = Deck::draw_hand(default_deck(), &mut MockRng3);
        let player2 = Player::new(hand2, deck2, PlayerNum::P2, 0);

        GameState::new(board, [player1, player2], 12, MockRng2)
    }

    fn game_state_offset() -> GameState<MockRng1> {
        let empty = BoardSpace::Empty;
        let p1_ink = BoardSpace::Ink {
            player_num: PlayerNum::P1,
        };
        let p2_ink = BoardSpace::Ink {
            player_num: PlayerNum::P2,
        };
        let board = Board::new(vec![
            vec![empty, empty, empty, empty, empty],
            vec![empty, empty, empty, empty, empty],
            vec![empty, empty, p2_ink, empty, empty],
            vec![p1_ink, empty, empty, empty, empty],
        ])
        .unwrap();

        let (deck1, hand1) = draw_hand1();
        let player1 = Player::new(hand1, deck1, 0);

        let (deck1, hand1) = draw_hand1();
        let player2 = Player::new(hand1, deck1, 0);

        GameState::new(board, [player1, player2], 12, MockRng1)
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
        let spaces = board_pos(&board, 8, 8).surrounding_spaces(&board);
        assert_eq!(spaces[0], empty);
        assert_eq!(spaces[1], p1_ink);
        assert_eq!(spaces[2], p2_ink);
        assert_eq!(spaces[3], empty);
        assert_eq!(spaces[4], p1_special);
        assert_eq!(spaces[5], p2_special);
        assert_eq!(spaces[6], empty);
        assert_eq!(spaces[7], p1_ink);

        let spaces = board_pos(&board, 7, 7).surrounding_spaces(&board);
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
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let empty = BoardSpace::Empty;
        let board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, p1_ink, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let (deck1, hand1) = draw_hand1();
        let player1 = Player::new(hand1, deck1, 0);

        let (deck2, hand2) = draw_hand2();
        let player2 = Player::new(hand2, deck2, 0);

        let mut game_state = GameState::new(board, [player1, player2], 12, MockRng1);

        let raw_placement = RawPlacement {
            x: 5,
            y: 5,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let hand_idx = HandIndex::H1;
        game_state.place(
            hand_idx,
            Placement::new(
                raw_placement,
                hand_idx,
                &game_state.board,
                &game_state.players[PlayerNum::P1],
                PlayerNum::P1,
            )
            .unwrap(),
            PlayerNum::P1,
        );

        let expected_board = Board::new(vec![
            vec![p1_ink, p1_ink, p1_special, empty],
            vec![p1_ink, p1_ink, p1_ink, p1_ink],
            vec![p1_ink, p1_ink, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state.board, expected_board);
    }

    #[test]
    fn test_place_both() {
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

        let mut game_state1 = game_state1();

        let raw_placement1 = RawPlacement {
            x: 5,
            y: 5,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let raw_placement2 = RawPlacement {
            x: 5,
            y: 5,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let hand_idx = HandIndex::H1;
        game_state1.place_both(
            hand_idx,
            hand_idx,
            Placement::new(
                raw_placement1,
                hand_idx,
                &game_state1.board,
                &game_state1.players[PlayerNum::P1],
                PlayerNum::P1,
            )
            .unwrap(),
            Placement::new(
                raw_placement2,
                hand_idx,
                &game_state1.board,
                &game_state1.players[PlayerNum::P2],
                PlayerNum::P2,
            )
            .unwrap(),
        );

        let expected_board1 = Board::new(vec![
            vec![wall, wall, wall, empty],
            vec![wall, wall, wall, wall],
            vec![wall, p1_ink, p2_ink, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state1.board, expected_board1);

        let mut game_state_offset = game_state_offset();
        let board_offset = &game_state_offset.board;

        let raw_placement1 = RawPlacement {
            x: 5,
            y: 5,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let raw_placement2 = RawPlacement {
            x: 6,
            y: 5,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let hand_idx = HandIndex::H1;
        game_state_offset.place_both(
            hand_idx,
            hand_idx,
            Placement::new(
                raw_placement1,
                hand_idx,
                board_offset,
                &game_state_offset.players[PlayerNum::P1],
                PlayerNum::P1,
            )
            .unwrap(),
            Placement::new(
                raw_placement2,
                hand_idx,
                board_offset,
                &game_state_offset.players[PlayerNum::P2],
                PlayerNum::P2,
            )
            .unwrap(),
        );

        let expected_board_offset = Board::new(vec![
            vec![p1_ink, wall, p1_special, p2_special, empty],
            vec![p1_ink, wall, wall, wall, p2_ink],
            vec![p1_ink, p2_ink, p2_ink, empty, empty],
            vec![p1_ink, empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state_offset.board, expected_board_offset);

        let mut game_state2 = game_state2();

        let raw_placement1 = RawPlacement {
            x: 5,
            y: 5,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let raw_placement2 = RawPlacement {
            x: 5,
            y: 4,
            special_activated: false,
            rotation: Rotation::Zero,
        };
        let hand_idx = HandIndex::H1;
        game_state2.place_both(
            hand_idx,
            hand_idx,
            Placement::new(
                raw_placement1,
                hand_idx,
                &game_state2.board,
                &game_state2.players[PlayerNum::P1],
                PlayerNum::P1,
            )
            .unwrap(),
            Placement::new(
                raw_placement2,
                hand_idx,
                &game_state2.board,
                &game_state2.players[PlayerNum::P2],
                PlayerNum::P2,
            )
            .unwrap(),
        );

        let expected_board2 = Board::new(vec![
            vec![p1_ink, p1_ink, p2_special, empty],
            vec![p1_ink, p2_ink, p2_ink, p1_ink],
            vec![p1_ink, p1_ink, p2_ink, empty],
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
            (board_pos(&board, 7, 7), InkSpace::Normal, InkSpace::Normal),
            (
                board_pos(&board, 8, 7),
                InkSpace::Special,
                InkSpace::Special,
            ),
            (board_pos(&board, 9, 7), InkSpace::Special, InkSpace::Normal),
            (board_pos(&board, 7, 8), InkSpace::Normal, InkSpace::Special),
        ];
        let result = resolve_overlap(overlap, BoardSpace::Wall, BoardSpace::Wall);
        let expected = vec![
            (board_pos(&board, 7, 7), BoardSpace::Wall),
            (board_pos(&board, 8, 7), BoardSpace::Wall),
            (
                board_pos(&board, 9, 7),
                BoardSpace::Special {
                    player_num: PlayerNum::P1,
                    is_activated: false,
                },
            ),
            (
                board_pos(&board, 7, 8),
                BoardSpace::Special {
                    player_num: PlayerNum::P2,
                    is_activated: false,
                },
            ),
        ];
        assert!(result == expected);
    }

    #[test]
    fn test_update_special_gauge() {
        let (deck, hand) = draw_hand1();
        let mut player = Player::new(hand, deck, 0);
        let p1_special = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: false,
        };
        let p1_special_active = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: true,
        };
        let wall = BoardSpace::Wall;
        let empty = BoardSpace::Empty;
        let mut board = Board::new(vec![
            vec![p1_special, wall, wall, p1_special_active],
            vec![wall, wall, wall, wall],
            vec![p1_special, empty, empty, empty],
            vec![empty, empty, empty, p1_special_active],
        ])
        .unwrap();
        update_special_gauge(&mut player, PlayerNum::P1, &mut board);
        assert_eq!(player.special, 1);
    }

    #[test]
    fn test_check_winner() {
        let game_state1 = game_state1();
        let outcome = game_state1.check_winner();
        assert!(matches!(outcome, Outcome::Draw));

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
            vec![empty, empty, p1_ink, empty],
            vec![empty, empty, empty, p2_ink],
            vec![empty, p1_special, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let (deck, hand) = draw_hand1();
        let player1 = Player::new(hand, deck, 0);

        let (deck, hand) = draw_hand1();
        let player2 = Player::new(hand, deck, 0);

        let game_state_p1_win = GameState::new(board, [player1, player2], 12, MockRng1);
        let outcome = game_state_p1_win.check_winner();
        assert!(matches!(outcome, Outcome::P1Win));

        let board = Board::new(vec![
            vec![empty, empty, p1_ink, empty],
            vec![empty, empty, empty, p2_ink],
            vec![empty, p2_ink, empty, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();

        let (deck, hand) = draw_hand1();
        let player1 = Player::new(hand, deck, 0);

        let (deck, hand) = draw_hand1();
        let player2 = Player::new(hand, deck, 0);

        let game_state_p2_win = GameState::new(board, [player1, player2], 12, MockRng1);
        let outcome = game_state_p2_win.check_winner();
        assert!(matches!(outcome, Outcome::P2Win));
    }

    #[test]
    fn test_update() {
        // Both players pass
        let mut game_state = game_state1();
        let empty = BoardSpace::Empty;
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
        let p1_special_active = BoardSpace::Special {
            player_num: PlayerNum::P1,
            is_activated: true,
        };
        let p2_special_active = BoardSpace::Special {
            player_num: PlayerNum::P2,
            is_activated: true,
        };

        let input1 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Pass,
            },
            &game_state.board,
            &game_state.players[PlayerNum::P1],
            PlayerNum::P1,
        )
        .unwrap();

        let input2 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Pass,
            },
            &game_state.board,
            &game_state.players[PlayerNum::P2],
            PlayerNum::P2,
        )
        .unwrap();

        game_state.update(input1, input2);
        let expected_board = Board::new(vec![
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, p1_ink, p2_ink, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state.turns_left(), 11);
        assert_eq!(game_state.board, expected_board);
        assert_eq!(game_state.players[PlayerNum::P1].special, 1);
        assert_eq!(game_state.players[PlayerNum::P2].special, 1);
        assert_eq!(
            game_state.players[PlayerNum::P1].hand()[HandIndex::H1],
            DeckIndex::D5
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H1],
            DeckIndex::D5
        );

        // One player passes
        let mut game_state = game_state1();

        let input1 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Place(RawPlacement {
                    x: 5,
                    y: 5,
                    special_activated: false,
                    rotation: Rotation::Zero,
                }),
            },
            &game_state.board,
            &game_state.players[PlayerNum::P1],
            PlayerNum::P1,
        )
        .unwrap();

        let input2 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Pass,
            },
            &game_state.board,
            &game_state.players[PlayerNum::P2],
            PlayerNum::P2,
        )
        .unwrap();

        game_state.update(input1, input2);
        let expected_board = Board::new(vec![
            vec![p1_ink, p1_ink, p1_special, empty],
            vec![p1_ink, p1_ink, p1_ink, p1_ink],
            vec![p1_ink, p1_ink, p2_ink, empty],
            vec![empty, empty, empty, empty],
        ])
        .unwrap();
        assert_eq!(game_state.turns_left(), 11);
        assert_eq!(game_state.board, expected_board);
        assert_eq!(game_state.players[PlayerNum::P1].special, 0);
        assert_eq!(game_state.players[PlayerNum::P2].special, 1);
        assert_eq!(
            game_state.players[PlayerNum::P1].hand()[HandIndex::H1],
            DeckIndex::D5
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H1],
            DeckIndex::D5
        );

        // Both players place ink
        let board = Board::new(vec![
            vec![empty, empty, empty, p1_ink],
            vec![empty, empty, empty, empty],
            vec![empty, empty, empty, empty],
            vec![empty, p1_ink, p2_ink, empty],
        ])
        .unwrap();

        let (deck, hand) = draw_hand1();
        let player1 = Player::new(hand, deck, 0);

        let (deck, hand) = draw_hand1();
        let player2 = Player::new(hand, deck, 0);

        let mut game_state = GameState::new(board, [player1, player2], 1, MockRng1);

        let input1 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Place(RawPlacement {
                    x: 5,
                    y: 5,
                    special_activated: false,
                    rotation: Rotation::Zero,
                }),
            },
            &game_state.board,
            &game_state.players[PlayerNum::P1],
            PlayerNum::P1,
        )
        .unwrap();

        let input2 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H2,
                action: Action::Place(RawPlacement {
                    x: 5,
                    y: 5,
                    special_activated: false,
                    rotation: Rotation::Zero,
                }),
            },
            &game_state.board,
            &game_state.players[PlayerNum::P2],
            PlayerNum::P2,
        )
        .unwrap();

        game_state.update(input1, input2);
        let expected_board = Board::new(vec![
            vec![p2_ink, p1_ink, p1_special_active, p1_ink],
            vec![p1_ink, p2_special_active, p2_ink, p1_ink],
            vec![p2_ink, p2_ink, p2_ink, empty],
            vec![empty, p1_ink, p2_ink, empty],
        ])
        .unwrap();
        assert_eq!(game_state.turns_left(), 0);
        assert_eq!(game_state.board, expected_board);
        assert_eq!(game_state.players[PlayerNum::P1].special, 1);
        assert_eq!(game_state.players[PlayerNum::P2].special, 1);
        assert_eq!(
            game_state.players[PlayerNum::P1].hand()[HandIndex::H1],
            DeckIndex::D5
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H1],
            DeckIndex::D1
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H2],
            DeckIndex::D5
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H3],
            DeckIndex::D3
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H4],
            DeckIndex::D4
        );

        // Both players place specials
        let board = Board::new(vec![
            vec![empty, p2_ink, empty, p1_special_active],
            vec![empty, empty, empty, empty],
            vec![empty, p1_ink, empty, empty],
            vec![empty, empty, p2_special, empty],
        ])
        .unwrap();

        let (deck, hand) = draw_hand1();
        let player1 = Player::new(hand, deck, 7);

        let (deck, hand) = draw_hand1();
        let player2 = Player::new(hand, deck, 8);

        let mut game_state = GameState::new(board, [player1, player2], 5, MockRng1);

        let input1 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H1,
                action: Action::Place(RawPlacement {
                    x: 5,
                    y: 5,
                    special_activated: true,
                    rotation: Rotation::Zero,
                }),
            },
            &game_state.board,
            &game_state.players[PlayerNum::P1],
            PlayerNum::P1,
        )
        .unwrap();

        let input2 = ValidInput::new(
            RawInput {
                hand_idx: HandIndex::H2,
                action: Action::Place(RawPlacement {
                    x: 5,
                    y: 5,
                    special_activated: true,
                    rotation: Rotation::Zero,
                }),
            },
            &game_state.board,
            &game_state.players[PlayerNum::P2],
            PlayerNum::P2,
        )
        .unwrap();

        game_state.update(input1, input2);
        let expected_board = Board::new(vec![
            vec![p2_ink, p1_ink, p1_special_active, p1_special_active],
            vec![p1_ink, p2_special_active, p2_ink, p1_ink],
            vec![p2_ink, p2_ink, p2_ink, empty],
            vec![empty, empty, p2_special, empty],
        ])
        .unwrap();
        assert_eq!(game_state.turns_left(), 4);
        assert_eq!(game_state.board, expected_board);
        assert_eq!(game_state.players[PlayerNum::P1].special, 5);
        assert_eq!(game_state.players[PlayerNum::P2].special, 6);
        assert_eq!(
            game_state.players[PlayerNum::P1].hand()[HandIndex::H1],
            DeckIndex::D5
        );
        assert_eq!(
            game_state.players[PlayerNum::P2].hand()[HandIndex::H2],
            DeckIndex::D5
        );
    }
}
