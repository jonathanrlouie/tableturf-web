use crate::{
    client::{Client, Clients, Status},
    ws, Games,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, Rejection, Reply};

type Result<T> = std::result::Result<T, Rejection>;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    url: String,
}

pub async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<impl Reply> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), user_id, clients).await;
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    }))
}

async fn register_client(id: String, user_id: usize, clients: Clients) {
    clients.write().await.insert(
        id,
        Client {
            user_id,
            status: Status::Idle,
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    clients.write().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    id: String,
    clients: Clients,
    games: Games,
) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => {
            Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c, games)))
        }
        None => Err(warp::reject::not_found()),
    }
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
