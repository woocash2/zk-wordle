use std::sync::Arc;

use ark_circom::read_zkey;
use ark_circom::CircomReduction;
use axum::{extract::State, response::IntoResponse, routing::get, serve, Json, Router};
use log::error;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::game_state::GameState;

use ark_bn254::Bn254;
use ark_circom::CircomBuilder;
use ark_circom::CircomConfig;
use ark_groth16::Groth16;
use ark_snark::SNARK;

fn generate_proof() {
    let mut key_file = std::fs::File::open("../proof/clue_0001.zkey").unwrap();
    let (params, _matrices) = read_zkey(&mut key_file).unwrap();

    let cfg =
        CircomConfig::<Bn254>::new("../proof/clue_js/clue.wasm", "../proof/clue.r1cs").unwrap();

    let mut builder = CircomBuilder::new(cfg);
    builder.push_input("word0", 1);
    builder.push_input("word1", 1);
    builder.push_input("word2", 1);
    builder.push_input("word3", 1);
    builder.push_input("word4", 1);
    builder.push_input("salt", 1237);
    builder.push_input("guess0", 1);
    builder.push_input("guess1", 1);
    builder.push_input("guess2", 1);
    builder.push_input("guess3", 1);
    builder.push_input("guess4", 1);
    builder.push_input("commit", 0);

    let circom = builder.build().unwrap();

    let inputs = circom.get_public_inputs().unwrap();

    let mut rng = rand::thread_rng();

    // Generate the proof
    let proof = Groth16::<Bn254, CircomReduction>::prove(&params, circom, &mut rng).unwrap();

    println!("{:?}", proof.a);
    println!("{:?}", proof.b);
    println!("{:?}", proof.c);

    let pvk = Groth16::<Bn254>::process_vk(&params.vk).unwrap();

    let verified = Groth16::<Bn254>::verify_with_processed_vk(&pvk, &inputs, &proof).unwrap();

    assert!(verified);
    println!("hej");
}

#[derive(Serialize, Deserialize)]
struct GuessRequest {
    guess: String,
}

pub async fn run(addr: &str, state: Arc<RwLock<GameState>>) {
    generate_proof();

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
