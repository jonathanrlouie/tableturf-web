use common::PlayerNum;
use hashbrown::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

#[derive(Error, Debug)]
#[error("Error sending message")]
pub struct SendError;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub enum Status {
    JoiningGame,
    InGame { uuid: String, player_num: PlayerNum },
    Idle,
}

#[derive(Debug, Clone)]
pub struct Sender(pub mpsc::UnboundedSender<Result<Message, warp::Error>>);

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub status: Status,
    pub sender: Option<Sender>,
}

pub trait SendMsg {
    fn send(&self, msg: &str) -> Result<(), SendError>;
}

impl SendMsg for Sender {
    fn send(&self, msg: &str) -> Result<(), SendError> {
        self.0.send(Ok(Message::text(msg))).map_err(|_| SendError)
    }
}
