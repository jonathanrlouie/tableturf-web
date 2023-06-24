use crate::tableturf::{Player, Board};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub board: Board,
    pub player: Player
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameEnd {
    pub board: Board,
    pub outcome: Outcome
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Outcome {
    Win,
    Lose,
    Draw,
}

