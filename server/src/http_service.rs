use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{extract::State, http::Method, response::IntoResponse, routing::get, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use log::{error, info};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use std::io::{self};
use std::net::ToSocketAddrs;
use std::path::Path as Path2;

use crate::game_state::SharedState;
use crate::proofs::generate_clue_proof;
use crate::request_response::StartResponse;
use crate::request_response::{GuessRequest, GuessResponse};

/// Runs the HTTP service. Routes two paths:
/// START, GET
/// GUESS, POST { word_id, guess }
pub async fn run(addr: &str, state: Arc<SharedState>) {
    info!("Starting server...");
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new()
        .route("/start", get(handle_start))
        .route("/guess", post(handle_guess))
        .layer(cors)
        .with_state(state);

    let addr = addr
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))
        .unwrap();

    let config = RustlsConfig::from_pem_file(
        Path2::new("/home/bb/fullchain.pem"),
        Path2::new("/home/bb/privkey.pem"),
    )
    .await
    .unwrap();

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Responds with the current round of the game (word_id) and the membership proof.
async fn handle_start(State(state): State<Arc<SharedState>>) -> impl IntoResponse {
    let game_state = state.mutable_game_state.read().clone();

    Json(StartResponse {
        word_id: game_state.word_id,
        commitment: game_state.commitment.to_string(),
        proof: game_state.membership_proof.into(),
    })
    .into_response()
}

/// Checks if the guess is for the current round (verify word_id), check that guess word exists
/// in the WordBank, and respond with a clue, and a clue correctness proof.
async fn handle_guess(
    State(state): State<Arc<SharedState>>,
    Json(guess): Json<GuessRequest>,
) -> impl IntoResponse {
    if guess.word_id != state.mutable_game_state.read().word_id {
        return Json((StatusCode::BAD_REQUEST, "bad word id")).into_response();
    }
    if !state.immutable_state.word_bank.has_word(&guess.guess) {
        return Json((StatusCode::BAD_REQUEST, "word does not exist")).into_response();
    }

    let game_state = state.mutable_game_state.read().clone();

    let (proof, clue) = match generate_clue_proof(
        guess.guess,
        game_state.solution.clone(),
        game_state.commitment.clone(),
        game_state.salt.clone(),
        state.immutable_state.clue_config.clone(),
        state.immutable_state.clue_pk.clone(),
    ) {
        Ok(value) => value,
        Err(e) => {
            error!("Clue proof generation failed: {:?}", e);
            return Json((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to generate a clue proof",
            ))
            .into_response();
        }
    };

    Json(GuessResponse {
        colors: clue,
        proof: proof.into(),
    })
    .into_response()
}
