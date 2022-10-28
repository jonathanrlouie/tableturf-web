/*
rough draft of the main game logic
fn main_logic() {
    let mut game_state = GameState::new();
    while game_state.turns_left() > 0 {
        let (input1, input2) = join!(read_socket1, read_socket2);
        game_state.update_game_state(&mut rng, input1, inpu2);
        send(socket1, game_state);
        send(socket2, game_state);
    }
    let outcome = game_state.check_winner()
}
*/
