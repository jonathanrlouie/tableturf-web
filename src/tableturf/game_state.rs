use crate::tableturf::board::Board;
use crate::tableturf::player::Player;

pub struct GameState {
    pub board: Board,
    pub players: [Player; 2],
    pub turns_left: u32,
}

