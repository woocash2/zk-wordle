use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, serve, Json, Router};
use log::error;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::game_state::GameState;

#[derive(Serialize, Deserialize)]
struct GuessRequest {
    guess: String,
}

pub async fn run(addr: &str, state: Arc<RwLock<GameState>>) {
    let app = Router::new()
        .route("/start", get(handle_start))
        .route("/guess", get(handle_guess))
        .with_state(state);

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

async fn handle_start(State(_state): State<Arc<RwLock<GameState>>>) -> impl IntoResponse {
    // TODO: prepare the start response
    "start".into_response()
}

async fn handle_guess(
    State(_state): State<Arc<RwLock<GameState>>>,
    Json(_body): Json<GuessRequest>,
) -> impl IntoResponse {
    // TODO: prepare the guess response
    "guess".into_response()
}
