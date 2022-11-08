/*
use crate::client::{Client, Clients, Status};
use crate::tableturf::{Deck, GameState, Hand, PlayerNum};
use crate::Games;
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{reply::json, Message, WebSocket};

#[derive(Serialize, Debug)]
pub struct StartGameResponse {
    hand: Hand,
    deck: Deck,
}

pub async fn client_connection(
    ws: WebSocket,
    id: String,
    clients: Clients,
    mut client: Client,
    games: Games,
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
        client_msg(&id, msg, &clients, &games).await;
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients, games: &Games) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
        return;
    }

    if message == "join" {
        client_join(id, clients, games);
        return;
    }
}

async fn client_join(id: &str, clients: &Clients, games: &Games) {
    let clients_map = clients.write().await;
    if let Some(mut client) = clients_map.get_mut(id) {
        match client.status {
            Status::Idle => {
                let mut waiting_clients = clients_map
                    .values_mut()
                    .filter(|c| matches!(c.status, Status::JoiningGame));
                if let Some(opponent) = waiting_clients.next() {
                    let game_state = GameState::default();
                    let hand1 = game_state.players[PlayerNum::P1].hand();
                    let deck1 = game_state.players[PlayerNum::P1].deck();
                    let hand2 = game_state.players[PlayerNum::P2].hand();
                    let deck2 = game_state.players[PlayerNum::P2].deck();
                    if let Some(sender) = client.sender {
                        sender.send(Ok(json(StartGameResponse {
                            hand: hand1,
                            deck: deck1,
                        })));
                    }
                    if let Some(sender) = opponent.sender {
                        sender.send(Ok(json(StartGameResponse {
                            hand: hand2,
                            deck: deck2,
                        })));
                    }

                    let game_uuid = Uuid::new_v4().as_simple().to_string();
                    games.write().await.insert(game_uuid, game_state);
                    client.status = Status::InGame { uuid: game_uuid };
                    opponent.status = Status::InGame { uuid: game_uuid };
                } else {
                    client.status = Status::JoiningGame;
                }
            }
            Status::JoiningGame | Status::InGame { .. } => (),
        }
    }
}
*/
