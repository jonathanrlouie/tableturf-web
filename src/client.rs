use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub enum Status {
    JoiningGame,
    InGame { uuid: String },
    Idle,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub status: Status,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}
