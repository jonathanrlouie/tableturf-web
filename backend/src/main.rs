use crate::client::Clients;
use crate::game::Games;
use hashbrown::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use warp::{
    http::{header, Method},
    Filter
};

mod client;
mod game;
mod handler;
mod util;
mod ws;

#[tracing::instrument]
#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::daily("./logs", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(non_blocking)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let games: Games = Arc::new(RwLock::new(HashMap::new()));
    info!("created clients and games maps");

    let health_route = warp::path!("health").and_then(handler::health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and(with_games(games.clone()))
        .and_then(handler::ws_handler);

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .with(
            warp::cors()
                .allow_credentials(true)
                .allow_methods(&[
                    Method::OPTIONS,
                    Method::GET,
                    Method::POST,
                    Method::DELETE,
                    Method::PUT,
                ])
                .allow_headers(vec![
                    header::CONTENT_TYPE,
                    header::ACCEPT,
                    header::ACCESS_CONTROL_ALLOW_ORIGIN])
                .expose_headers(vec![header::LINK])
                .max_age(300)
                .allow_any_origin());


    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}
