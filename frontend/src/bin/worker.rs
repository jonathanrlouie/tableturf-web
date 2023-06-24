use yew_agent::PublicWorker;
use frontend::worker::WebSocketWorker;

fn main() {
    WebSocketWorker::register();
}
