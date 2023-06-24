use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use reqwasm::{
    http::{Request as HttpRequest},
    websocket::{futures::WebSocket, Message}
};
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;
use yew_agent::Dispatched;
use crate::worker::{WebSocketWorker, Request};
use gloo::console::log;
//use tracing;

#[derive(Deserialize)]
struct RegistrationResponse {
    url: String
}

#[tracing::instrument]
pub fn connect(user_id: String) -> Sender<String> {
    let (in_tx, mut in_rx) = futures::channel::mpsc::channel::<String>(1000);
    let mut ws_worker = WebSocketWorker::dispatcher();
    spawn_local(async move {
        // send curl request first to get url
        //tracing::debug!("Sending curl request for ws URL");
        let response = HttpRequest::post("http://localhost:8000/register")
            .header("Content-Type", "application/json")
            .body(format!("{{ \"user_id\": {}}}", user_id))
            .send()
            .await
            .unwrap();

        //tracing::debug!("Parsing JSON response with ws URL");
        let url_response: RegistrationResponse = response.json().await.unwrap();
        //tracing::debug!("Opening ws connection");
        let ws = WebSocket::open(&url_response.url).unwrap();

        let (mut write, mut read) = ws.split();
    
        spawn_local(async move {
            while let Some(s) = in_rx.next().await {
                log!("Sent message: ", &s);
                write.send(Message::Text(s)).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        //tracing::debug!("from websocket: {}", data);
                        log!("Received message: ", &data);
                        ws_worker.send(Request::Input(data));
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        if let Ok(val) = decoded {
                            //tracing::debug!("from websocket: {}", val);
                            log!("Received message: ", val);
                            ws_worker.send(Request::Input(val.into()));
                        }
                    }
                    Err(e) => {
                        //tracing::error!("ws: {:?}", e)
                    }
                }
            }
            //tracing::debug!("WebSocket closed");
        });
    });

    in_tx
}

