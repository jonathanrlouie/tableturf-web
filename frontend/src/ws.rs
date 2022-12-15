/*
use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use reqwasm::websocket::{futures::WebSocket, Message};
use wasm_bindgen_futures::spawn_local;
use tracing;

pub fn connect() -> Sender<String> {
    // send curl request first to get url
    //let url = curl::request("curl -X POST \'http://localhost:8000/register\' -H \'Content-Type: applicationi/json\' -d ");
    let ws = WebSocket::open(url).unwrap();

    let (mut write, mut read) = ws.split();

    let (in_tx, mut in_rx) = futures::channel::mpsc::channel::<String>(1000);
    spawn_local(async move {
        while let Some(s) = in_rx.next().await {
            tracing::debug!("got event from channel! {}", s);
            write.send(Message::Text(s)).await.unwrap();
        }
    });

    spawn_local(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(data)) => {
                    tracing::debug!("from websocket: {}", data);
                }
                Ok(Message::Bytes(b)) => {
                    let decoded = std::str::from_utf8(&b);
                    if let Ok(val) = decoded {
                        tracing::debug!("from websocket: {}", val);
                    }
                }
                Err(e) => {
                    tracing::error!("ws: {:?}", e)
                }
            }
        }
        tracing::debug!("WebSocket closed");
    });

    in_tx
}
*/
