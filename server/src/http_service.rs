use axum::{
    extract::Path, extract::State, http::Method, response::IntoResponse, routing::get, serve, Json,
    Router,
};
use log::error;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::game_state::SharedState;
use crate::proofs::generate_clue_proof;
use crate::proofs::generate_membership_proof;
use crate::request_response::GuessResponse;
use crate::request_response::StartResponse;

pub async fn run(addr: &str, state: Arc<RwLock<SharedState>>) {
    println!("starting server");
    // TODO: figure out how to handle CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/start", get(handle_start))
        .route("/guess/:guess", get(handle_guess))
        .layer(cors)
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

async fn handle_start(State(state): State<Arc<RwLock<SharedState>>>) -> impl IntoResponse {
    let state = state.read().clone();
    let commitment = state.cm_state.commitment.clone();

    let proof = generate_membership_proof(
        state.game_state.solution,
        state.cm_state,
        state.merkle_state,
        state.membership_config,
        state.membership_pk,
    );

    Json(StartResponse {
        word_id: state.game_state.word_id,
        commitment: commitment.to_string(),
        proof: proof.into(),
    })
    .into_response()
}

async fn handle_guess(
    State(state): State<Arc<RwLock<SharedState>>>,
    Path(guess): Path<String>,
) -> impl IntoResponse {
    let state = state.read().clone();

    let (proof, clue) = generate_clue_proof(
        guess,
        state.game_state.solution,
        state.cm_state,
        state.clue_config.clone(),
        state.clue_pk,
    );

    Json(GuessResponse {
        colors: clue,
        proof: proof.into(),
    })
    .into_response()
}
