use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew_agent::{Worker, WorkerLink, Public, HandlerId};
use gloo::console::log;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Input(String),
}

pub struct WebSocketWorker {
    link: WorkerLink<Self>,
    // subscribed Yew components that we will forward messages received from the backend to
    subscribers: HashSet<HandlerId>,
}

impl Worker for WebSocketWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = Request;
    type Output = String;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new()
        }
    }

    fn update(&mut self, _msg: Self::Message) {
    }

    // Handle messages coming from the server
    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Input(s) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, s.clone());
                }
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }

    fn resource_path_is_relative() -> bool {
        true
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
