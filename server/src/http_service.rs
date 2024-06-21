use ark_bn254::Bn254;
use ark_circom::{circom::Inputs, CircomBuilder, CircomConfig, CircomReduction};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Groth16, Proof, ProvingKey};
use ark_snark::SNARK;
use axum::{
    extract::Path, extract::State, http::Method, response::IntoResponse, routing::get, Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;

use merkle::NodeType;
use num_bigint::BigInt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use std::io::{self};
use std::net::ToSocketAddrs;
use std::path::Path as Path2;

use crate::game_state::CmState;
use crate::game_state::MerkleState;
use crate::game_state::SharedState;

fn generate_clue_proof(
    guess: String,
    solution: String,
    cm_state: CmState,
    config: CircomConfig<Bn254>,
    pk: ProvingKey<Bn254>,
) -> (Proof<Bn254>, [u8; 5]) {
    let CmState { commitment, salt } = cm_state;

    let guess = string_to_bigints(guess);
    let solution = string_to_bigints(solution);

    let mut builder = CircomBuilder::new(config);
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

    (proof, clue)
}

fn generate_membership_proof(
    solution: String,
    cm_state: CmState,
    merkle_state: MerkleState,
    config: CircomConfig<Bn254>,
    pk: ProvingKey<Bn254>,
) -> Proof<Bn254> {
    let solution = string_to_bigints(solution);
    let mut builder = CircomBuilder::new(config);

    let mut hashes = Vec::with_capacity(merkle_state.path.len());
    let mut indicators = Vec::with_capacity(merkle_state.path.len());

    for entry in merkle_state.path {
        hashes.push(vec![entry.left.into(), entry.right.into()]);
        indicators.push(match entry.on_path {
            NodeType::Left => 0.into(),
            NodeType::Right => 1.into(),
        });
    }

    builder.push_input("word", Inputs::BigIntVec(solution));
    builder.push_input("salt", Inputs::BigInt(cm_state.salt.into()));
    builder.push_input("cm", Inputs::BigInt(cm_state.commitment.into()));
    builder.push_input("hashes", Inputs::BigIntVecVec(hashes));
    builder.push_input("pathIndicators", Inputs::BigIntVec(indicators));

    let circom = builder.build().unwrap();

    // Generate the proof
    let mut rng = rand::thread_rng();
    let proof = Groth16::<Bn254, CircomReduction>::prove(&pk, circom, &mut rng).unwrap();

    proof
}

fn string_to_bigints(s: String) -> Vec<BigInt> {
    s.as_bytes().iter().map(|x| (x - 97).into()).collect()
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

    let addr = addr
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))
        .unwrap();

    let config =
        RustlsConfig::from_pem_file(Path2::new("~/server.crt"), Path2::new("~/server.key"))
            .await
            .unwrap();

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize)]
struct StartResponse {
    word_id: usize,
    commitment: String,
    proof: ProofSerializable,
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
        proof: ProofSerializable {
            a: proof.a.to_string(),
            b: proof.b.to_string(),
            c: proof.c.to_string(),
        },
    })
    .into_response()
}

#[derive(Serialize)]
struct ProofSerializable {
    a: String,
    b: String,
    c: String,
}

#[derive(Serialize)]
struct GuessResponse {
    colors: [u8; 5],
    proof: ProofSerializable,
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
        proof: ProofSerializable {
            a: proof.a.to_string(),
            b: proof.b.to_string(),
            c: proof.c.to_string(),
        },
    })
    .into_response()
}
