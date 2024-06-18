use ark_bn254::Bn254;
use ark_circom::CircomBuilder;
use ark_circom::CircomConfig;
use ark_circom::CircomReduction;
use ark_ff::BigInteger;
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_groth16::Proof;
use ark_groth16::ProvingKey;
use ark_snark::SNARK;
use axum::extract::Path;
use axum::http::Method;
use axum::{extract::State, response::IntoResponse, routing::get, serve, Json, Router};
use log::error;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::game_state::SharedState;

fn generate_proof(
    guess: &[u8],
    word: &[char],
    config: CircomConfig<Bn254>,
    pk: ProvingKey<Bn254>,
) -> (Proof<Bn254>, [u8; 5]) {
    let mut builder = CircomBuilder::new(config);
    // workaround for the fact that this builder doesn't support array inputs
    builder.push_input("word0", (word[0] as u8) - 97);
    builder.push_input("word1", (word[1] as u8) - 97);
    builder.push_input("word2", (word[2] as u8) - 97);
    builder.push_input("word3", (word[3] as u8) - 97);
    builder.push_input("word4", (word[4] as u8) - 97);
    builder.push_input("salt", 1237);
    builder.push_input("guess0", guess[0] - 97);
    builder.push_input("guess1", guess[1] - 97);
    builder.push_input("guess2", guess[2] - 97);
    builder.push_input("guess3", guess[3] - 97);
    builder.push_input("guess4", guess[4] - 97);
    builder.push_input("commit", 0);

    let circom = builder.build().unwrap();

    // the first five public inputs are actually public outputs
    let mut inputs = circom.get_public_inputs().unwrap();
    let _ = inputs.split_off(5);

    let mut clue: [u8; 5] = [0, 0, 0, 0, 0];

    println!("{:?}", inputs);
    println!("{:?}", PrimeField::into_bigint(inputs[0]).to_bytes_le());

    for i in 0..inputs.len() {
        if i < 5 {
            // since the values are either 0, 1 or 2, we can just take the first little endian byte
            clue[i] = PrimeField::into_bigint(inputs[i]).to_bytes_le()[0];
        }
    }

    let mut rng = rand::thread_rng();

    // Generate the proof
    let proof = Groth16::<Bn254, CircomReduction>::prove(&pk, circom, &mut rng).unwrap();

    println!("{:?}", proof.a);
    println!("{:?}", proof.b);
    println!("{:?}", proof.c);

    (proof, clue)
}

#[derive(Serialize, Deserialize)]
struct GuessRequest {
    guess: String,
}

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

#[derive(Serialize)]
struct StartResponse {
    word_id: usize,
    commitment: String,
}

async fn handle_start(State(state): State<Arc<RwLock<SharedState>>>) -> impl IntoResponse {
    let state = state.read().clone();

    Json(StartResponse {
        word_id: state.game_state.word_id,
        commitment: state.game_state.commitment.to_string(),
    })
    .into_response()
}

#[derive(Serialize)]
struct BBProof {
    a: String,
    b: String,
    c: String,
}

#[derive(Serialize)]
struct GuessResponse {
    colors: [u8; 5],
    proof: BBProof,
}

async fn handle_guess(
    State(state): State<Arc<RwLock<SharedState>>>,
    Path(guess): Path<String>,
) -> impl IntoResponse {
    let state = state.read().clone();
    let mut array = [0, 0, 0, 0, 0];
    array[..guess.len()].copy_from_slice(guess.as_bytes());
    let solution: Vec<_> = state.game_state.solution.chars().collect();
    let (proof, clue) = generate_proof(
        &array,
        solution.as_slice(),
        state.config.clone(),
        state.pk.clone(),
    );

    Json(GuessResponse {
        colors: clue,
        proof: BBProof {
            a: proof.a.to_string(),
            b: proof.b.to_string(),
            c: proof.c.to_string(),
        },
    })
    .into_response()
}
