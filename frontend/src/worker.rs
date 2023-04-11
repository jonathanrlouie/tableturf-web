use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew_agent::{Worker, WorkerLink, Public, HandlerId};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Input(String),
}

pub struct WebSocketWorker {
    link: WorkerLink<Self>,
}

impl Worker for WebSocketWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = Request;
    type Output = String;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
        }
    }

    fn update(&mut self, _msg: Self::Message) {
    }

    // Handle messages coming from the server
    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Input(s) => {
                self.link.respond(id, s);
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }

    fn resource_path_is_relative() -> bool {
        true
    }
}
