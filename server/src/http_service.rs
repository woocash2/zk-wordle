use ark_bn254::Bn254;
use ark_circom::{circom::Inputs, CircomBuilder, CircomConfig, CircomReduction};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Groth16, Proof, ProvingKey};
use ark_snark::SNARK;
use axum::{
    extract::Path, extract::State, http::Method, response::IntoResponse, routing::get, serve, Json,
    Router,
};
use log::error;
use num_bigint::BigInt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::game_state::GameState;
use crate::game_state::SharedState;

fn generate_proof(
    guess: String,
    game_state: GameState,
    config: CircomConfig<Bn254>,
    pk: ProvingKey<Bn254>,
) -> (Proof<Bn254>, [u8; 5]) {
    let GameState {
        solution,
        salt,
        commitment,
        ..
    } = game_state;
    let guess = string_to_bigints(guess);
    let solution = string_to_bigints(solution);

    let mut builder = CircomBuilder::new(config);
    // workaround for the fact that this builder doesn't support array inputs
    builder.push_input("word", Inputs::BigIntVec(solution));
    builder.push_input("guess", Inputs::BigIntVec(guess));
    builder.push_input("salt", Inputs::BigInt(salt.into()));
    builder.push_input("commit", Inputs::BigInt(commitment.into()));

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

fn string_to_bigints(s: String) -> Vec<BigInt> {
    s.as_bytes().into_iter().map(|x| (x - 97).into()).collect()
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
    
    let (proof, clue) = generate_proof(
        guess,
        state.game_state,
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
