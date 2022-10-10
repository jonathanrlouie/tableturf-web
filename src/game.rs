use crate::input::*;
use crate::tableturf::*;

// input1: player 1's input
// input2: player 2's input
fn update_game_state(game_state: &mut GameState, input1: ValidInput, input2: ValidInput) {
    let card_idx1 = input1.card_idx();
    let card_idx2 = input2.card_idx();
    match (input1.get(), input2.get()) {
        (Input::Pass, Input::Pass) => {
            game_state.players[0].special += 1;
            game_state.players[1].special += 1;
        }
        (Input::Place(placement), Input::Pass) => {
            game_state.players[1].special += 1;
            game_state.place(card_idx1, placement, 0);
        }
        (Input::Pass, Input::Place(placement)) => {
            game_state.players[0].special += 1;
            game_state.place(card_idx2, placement, 1);
        }
        (Input::Place(placement1), Input::Place(placement2)) => {
            game_state.place_both(card_idx1, card_idx2, placement1, placement2);
        }
    };
    let player1 = &mut game_state.players[0];
    player1.replace_card(card_idx1);
    player1.update_special_gauge(&mut game_state.board);
    let player2 = &mut game_state.players[1];
    player2.replace_card(card_idx2);
    player2.update_special_gauge(&mut game_state.board);

    if game_state.turns_left > 0 {
        game_state.turns_left -= 1;
    }
}

/*
rough draft of the main game logic
fn main_logic() {
    // pass in hands to make it easier to test GameState
    let mut game_state = GameState::new(rng, hand1, hand2);
    while game_state.turns_left > 0 {
        update_game_state(game_state);
    }
    let outcome = game_state.check_winner()
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
