use crate::client::Client;
use crate::tableturf::{
    Board, DeckRng, DrawRng, GameState, Outcome as GameOutcome, Player, PlayerNum, RawInput,
    ValidInput,
};
use hashbrown::HashMap;
use serde::Serialize;
use serde_json::from_str;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::ws::Message;

pub type Games = Arc<RwLock<HashMap<String, Game<DeckRng>>>>;

#[derive(Serialize, Debug)]
pub struct RedrawResponse {
    pub player: Player,
}

#[derive(Serialize, Debug)]
struct StateResponse {
    board: Board,
    player: Player,
}

#[derive(Serialize, Debug)]
enum Outcome {
    Win,
    Lose,
    Draw,
}

#[derive(Serialize, Debug)]
struct GameEndResponse {
    outcome: Outcome,
}

#[derive(Clone, Debug)]
enum ProtocolState {
    // true means that the player wants to redraw their hand, false means they don't
    Redraw([Option<bool>; 2]),
    InGame([Option<ValidInput>; 2]),
    // true means that the player wants a rematch, false means they don't
    Rematch([Option<bool>; 2]),
    End,
}

pub struct Game<R: DrawRng + Default> {
    game_state: GameState<R>,
    // The first element is Player 1's ID and the second is Player 2's ID
    player_ids: [String; 2],
    protocol_state: ProtocolState,
}

impl<R: DrawRng + Default> Game<R> {
    pub fn new(game_state: GameState<R>, player_ids: [String; 2]) -> Self {
        Game {
            game_state,
            player_ids,
            protocol_state: ProtocolState::Redraw([None, None]),
        }
    }

    pub fn is_over(&self) -> bool {
        matches!(self.protocol_state, ProtocolState::End)
    }

    // Given a client's ID, gets the opponent's ID for the game they have joined
    pub fn opponent_id(&self, id: String) -> String {
        if id == self.player_ids[0] {
            self.player_ids[1].clone()
        } else if id == self.player_ids[1] {
            self.player_ids[0].clone()
        } else {
            panic!(
                "Client with ID {} did not match any of the game's client IDs {:?}",
                id, self.player_ids
            );
        }
    }

    pub fn handle_message(
        &mut self,
        player_num: PlayerNum,
        msg: &str,
        client: &Client,
        opponent: &Client,
    ) {
        use ProtocolState::*;
        self.protocol_state = match self.protocol_state.clone() {
            Redraw(choices) => self.process_redraw_choice(
                client,
                opponent,
                choices,
                player_num,
                from_str(msg)
                    .map_err(|_| "unable to deserialize input into redraw choice".to_string())
                    .unwrap(),
            ),
            InGame(inputs) => self.process_input(
                client,
                opponent,
                inputs,
                player_num,
                from_str(msg)
                    .map_err(|_| "unable to deserialize input into game input".to_string())
                    .unwrap(),
            ),
            Rematch(choices) => self.process_rematch_choice(
                client,
                opponent,
                choices,
                player_num,
                from_str(msg)
                    .map_err(|_| "unable to deserialize input into rematch choice".to_string())
                    .unwrap(),
            ),
            End => End,
        }
    }

    fn process_redraw_choice(
        &mut self,
        client: &Client,
        opponent: &Client,
        choices: [Option<bool>; 2],
        player_num: PlayerNum,
        choice: bool,
    ) -> ProtocolState {
        let choices = match player_num {
            PlayerNum::P1 => [Some(choice), choices[1]],
            PlayerNum::P2 => [choices[0], Some(choice)],
        };
        match choices {
            [Some(true), Some(true)] => {
                self.game_state.redraw_hand(PlayerNum::P1);
                self.game_state.redraw_hand(PlayerNum::P2);

                let client_msg = RedrawResponse {
                    player: self.game_state.player(player_num).clone(),
                };
                let opponent_msg = RedrawResponse {
                    player: self.game_state.player(other_player(player_num)).clone(),
                };
                send_messages(client, client_msg, opponent, opponent_msg);
                ProtocolState::InGame([None, None])
            }
            [Some(true), Some(false)] => {
                self.game_state.redraw_hand(PlayerNum::P1);
                let client_msg = RedrawResponse {
                    player: self.game_state.player(player_num).clone(),
                };
                let opponent_msg = RedrawResponse {
                    player: self.game_state.player(other_player(player_num)).clone(),
                };
                send_messages(client, client_msg, opponent, opponent_msg);
                ProtocolState::InGame([None, None])
            }
            [Some(false), Some(true)] => {
                self.game_state.redraw_hand(PlayerNum::P2);
                let client_msg = RedrawResponse {
                    player: self.game_state.player(player_num).clone(),
                };
                let opponent_msg = RedrawResponse {
                    player: self.game_state.player(other_player(player_num)).clone(),
                };
                send_messages(client, client_msg, opponent, opponent_msg);
                ProtocolState::InGame([None, None])
            }
            [Some(false), Some(false)] => {
                let client_msg = RedrawResponse {
                    player: self.game_state.player(player_num).clone(),
                };
                let opponent_msg = RedrawResponse {
                    player: self.game_state.player(other_player(player_num)).clone(),
                };
                send_messages(client, client_msg, opponent, opponent_msg);
                ProtocolState::InGame([None, None])
            }
            _ => ProtocolState::Redraw(choices),
        }
    }

    fn process_input(
        &mut self,
        client: &Client,
        opponent: &Client,
        inputs: [Option<ValidInput>; 2],
        player_num: PlayerNum,
        input: RawInput,
    ) -> ProtocolState {
        let validated_input = ValidInput::new(
            input,
            self.game_state.board(),
            self.game_state.player(player_num),
            player_num,
        )
        .unwrap();
        let choices = match player_num {
            PlayerNum::P1 => [Some(validated_input), inputs[1].clone()],
            PlayerNum::P2 => [inputs[0].clone(), Some(validated_input)],
        };
        match choices {
            [Some(input1), Some(input2)] => {
                self.game_state.update(input1, input2);
                if self.game_state.turns_left() == 0 {
                    let winner = self.game_state.check_winner();
                    match (winner, player_num) {
                        (GameOutcome::P1Win, PlayerNum::P1) => {
                            send_outcomes(client, Outcome::Win, opponent, Outcome::Lose);
                        }
                        (GameOutcome::P2Win, PlayerNum::P1) => {
                            send_outcomes(client, Outcome::Lose, opponent, Outcome::Win);
                        }
                        (GameOutcome::P1Win, PlayerNum::P2) => {
                            send_outcomes(client, Outcome::Lose, opponent, Outcome::Win);
                        }
                        (GameOutcome::P2Win, PlayerNum::P2) => {
                            send_outcomes(client, Outcome::Win, opponent, Outcome::Lose);
                        }
                        (GameOutcome::Draw, _) => {
                            send_outcomes(client, Outcome::Draw, opponent, Outcome::Draw);
                        }
                    }
                    ProtocolState::Rematch([None, None])
                } else {
                    let client_msg = StateResponse {
                        board: self.game_state.board().clone(),
                        player: self.game_state.player(player_num).clone(),
                    };
                    let opponent_msg = StateResponse {
                        board: self.game_state.board().clone(),
                        player: self.game_state.player(other_player(player_num)).clone(),
                    };
                    send_messages(client, client_msg, opponent, opponent_msg);
                    ProtocolState::InGame([None, None])
                }
            }
            _ => ProtocolState::InGame(choices),
        }
    }

    fn process_rematch_choice(
        &mut self,
        client: &Client,
        opponent: &Client,
        choices: [Option<bool>; 2],
        player_num: PlayerNum,
        choice: bool,
    ) -> ProtocolState {
        let choices = match player_num {
            PlayerNum::P1 => [Some(choice), choices[1]],
            PlayerNum::P2 => [choices[0], Some(choice)],
        };
        match choices {
            [Some(true), Some(true)] => {
                self.game_state = GameState::default();
                let client_msg = StateResponse {
                    board: self.game_state.board().clone(),
                    player: self.game_state.player(player_num).clone(),
                };
                let opponent_msg = StateResponse {
                    board: self.game_state.board().clone(),
                    player: self.game_state.player(other_player(player_num)).clone(),
                };
                send_messages(client, client_msg, opponent, opponent_msg);
                ProtocolState::Redraw([None, None])
            }
            // Let ws module handle removing the game
            [_, Some(false)] | [Some(false), _] => ProtocolState::End,
            _ => ProtocolState::Rematch(choices),
        }
    }
}

fn send_outcomes(
    client: &Client,
    client_outcome: Outcome,
    opponent: &Client,
    opponent_outcome: Outcome,
) {
    let client_msg = GameEndResponse {
        outcome: client_outcome,
    };
    let opponent_msg = GameEndResponse {
        outcome: opponent_outcome,
    };
    send_messages(client, client_msg, opponent, opponent_msg);
}

fn other_player(player_num: PlayerNum) -> PlayerNum {
    match player_num {
        PlayerNum::P1 => PlayerNum::P2,
        PlayerNum::P2 => PlayerNum::P1,
    }
}

fn send_message<M: Serialize>(client: &Client, message: M) {
    if let Some(sender) = &client.sender {
        sender
            .send(Ok(Message::text(serde_json::to_string(&message).unwrap())))
            .unwrap();
    }
}

fn send_messages<M1: Serialize, M2: Serialize>(
    client1: &Client,
    message1: M1,
    client2: &Client,
    message2: M2,
) {
    send_message(client1, message1);
    send_message(client2, message2);
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_handle_message() {
    pub fn handle_message(&mut self, player_num: PlayerNum, msg: Message) -> Result<Response, String> {
        match game.handle_message(player_num, msg) {
            Ok(close_connections) => if close_connections {},
        }
    }
    */
}
