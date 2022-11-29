use crate::client::{Client, Clients, Status};
use crate::game::{Game, Games, StateResponse};
use crate::tableturf::{Board, GameState, Player, PlayerNum};
use futures::{FutureExt, StreamExt};
use hashbrown::HashMap;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{info, warn};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub async fn client_connection(
    ws: WebSocket,
    id: String,
    clients: Clients,
    mut client: Client,
    mut games: Games,
) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.write().await.insert(id.clone(), client);

    println!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients, &mut games).await;
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

#[tracing::instrument]
async fn client_msg(id: &str, msg: Message, clients: &Clients, games: &mut Games) {
    info!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v.trim(),
        Err(_) => return,
    };

    if message == "ping" {
        return;
    }

    let mut clients_map = clients.write().await;
    let client = match clients_map.get_mut(id) {
        Some(v) => v,
        None => {
            println!(
                "Message from client {} did not match any connected clients",
                id
            );
            return;
        }
    };
    match &client.status {
        Status::InGame { uuid, player_num } => {
            let uuid = uuid.clone();
            let player_num = *player_num;
            let mut games_map = games.write().await;
            let game = match games_map.get_mut(&uuid) {
                Some(v) => v,
                None => {
                    warn!("Game with ID {} did not match any existing games", uuid);
                    return;
                }
            };
            let [client, opponent] = clients_map
                .get_many_mut([id, &game.opponent_id(id.to_string())])
                .unwrap();
            game.handle_message(player_num, message, client, opponent);
            if game.is_over() {
                client.status = Status::Idle;
                client
                    .sender
                    .as_ref()
                    .unwrap()
                    .send(Ok(Message::text("leave")))
                    .unwrap();
                opponent.status = Status::Idle;
                opponent
                    .sender
                    .as_ref()
                    .unwrap()
                    .send(Ok(Message::text("leave")))
                    .unwrap();
                games_map.remove(&uuid);
            }
        }
        Status::Idle => {
            if message == "join" {
                info!("client {} joining a game", id);
                client_join(id, &mut clients_map, games).await;
            }
        }
        Status::JoiningGame => {}
    }
}

async fn client_join(id: &str, clients: &mut HashMap<String, Client>, games: &mut Games) {
    let mut waiting_clients = clients
        .iter_mut()
        .filter(|(_, c)| matches!(c.status, Status::JoiningGame))
        .map(|(id, _)| id);
    if let Some(opponent_id) = waiting_clients.next() {
        let opponent_id = opponent_id.clone();
        let [client, opponent] = clients.get_many_mut([id, &opponent_id]).unwrap();

        let game_state = GameState::default();
        let player1 = game_state.player(PlayerNum::P1).clone();
        let player2 = game_state.player(PlayerNum::P2).clone();
        if let Some(sender) = &client.sender {
            sender
                .send(Ok(Message::text(
                    serde_json::to_string(&StateResponse {
                        board: game_state.board().clone(),
                        player: player1,
                    })
                    .unwrap(),
                )))
                .unwrap();
        }
        if let Some(sender) = &opponent.sender {
            sender
                .send(Ok(Message::text(
                    serde_json::to_string(&StateResponse {
                        board: game_state.board().clone(),
                        player: player2,
                    })
                    .unwrap(),
                )))
                .unwrap();
        }

        let game_uuid = Uuid::new_v4().as_simple().to_string();
        games.write().await.insert(
            game_uuid.clone(),
            Game::new(game_state, [id.to_string(), opponent_id.to_string()]),
        );
        client.status = Status::InGame {
            uuid: game_uuid.clone(),
            player_num: PlayerNum::P1,
        };
        opponent.status = Status::InGame {
            uuid: game_uuid,
            player_num: PlayerNum::P2,
        };
    } else {
        let client = clients.get_mut(id).unwrap();
        client.status = Status::JoiningGame;
    }
}
