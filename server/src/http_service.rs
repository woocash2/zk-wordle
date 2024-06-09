use axum::{response::IntoResponse, routing::get, serve, Router};
use log::error;
use tokio::net::TcpListener;

pub async fn run(addr: &str) {
    let app = Router::new()
        .route("/start", get(handle_start))
        .route("/guess", get(handle_guess));

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Couldn't create listener: {}", e);
            return;
        }
    };

    if let Err(e) = serve(listener, app).await {
        error!("Error while serving: {}", e);
    }
}

async fn handle_start() -> impl IntoResponse {
    // TODO: prepare the start response
    "start".into_response()
}

async fn handle_guess() -> impl IntoResponse {
    // TODO: prepare the guess response
    "guess".into_response()
}
