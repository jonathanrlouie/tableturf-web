use crate::tableturf::{Player, Board};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Redraw {
        player: Player
    },
    GameState {
        board: Board,
        player: Player
    },
    GameEnd {
        outcome: Outcome
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Outcome {
    Win,
    Lose,
    Draw,
}

