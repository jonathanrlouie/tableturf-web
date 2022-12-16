use crate::tableturf::{Player, Board};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RedrawResponse {
    pub player: Player,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateResponse {
    pub board: Board,
    pub player: Player,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Outcome {
    Win,
    Lose,
    Draw,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameEndResponse {
    pub outcome: Outcome,
}
