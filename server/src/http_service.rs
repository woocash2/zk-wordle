use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{
    extract::State, http::Method, response::IntoResponse, routing::get, serve, Json, Router,
};
use log::error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::game_state::SharedState;
use crate::proofs::generate_clue_proof;
use crate::proofs::generate_membership_proof;
use crate::request_response::StartResponse;
use crate::request_response::{GuessRequest, GuessResponse};

pub async fn run(addr: &str, state: Arc<SharedState>) {
    println!("starting server");
    // TODO: figure out how to handle CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new()
        .route("/start", get(handle_start))
        .route("/guess", post(handle_guess))
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

async fn handle_start(State(state): State<Arc<SharedState>>) -> impl IntoResponse {
    let game_state = state.mutable_game_state.read().clone();

    let proof = generate_membership_proof(
        game_state.solution,
        game_state.commitment.clone(),
        game_state.salt,
        game_state.path,
        state.immutable_proving_state.membership_config.clone(),
        state.immutable_proving_state.membership_pk.clone(),
    );

    Json(StartResponse {
        word_id: game_state.word_id,
        commitment: game_state.commitment.to_string(),
        proof: proof.into(),
    })
    .into_response()
}

async fn handle_guess(
    State(state): State<Arc<SharedState>>,
    Json(guess): Json<GuessRequest>,
) -> impl IntoResponse {
    if guess.word_id != state.mutable_game_state.read().word_id {
        return Json((StatusCode::BAD_REQUEST, "bad word id")).into_response();
    }
    if !state
        .immutable_proving_state
        .word_bank
        .has_word(&guess.guess)
    {
        return Json((StatusCode::BAD_REQUEST, "word does not exist")).into_response();
    }

    let game_state = state.mutable_game_state.read().clone();

    let (proof, clue) = generate_clue_proof(
        guess.guess,
        game_state.solution.clone(),
        game_state.commitment.clone(),
        game_state.salt.clone(),
        state.immutable_proving_state.clue_config.clone(),
        state.immutable_proving_state.clue_pk.clone(),
    );

    Json(GuessResponse {
        colors: clue,
        proof: proof.into(),
    })
    .into_response()
}
