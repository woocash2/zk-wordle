use ark_bn254::Bn254;
use ark_groth16::Proof;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ProofSerializable {
    a: String,
    b: String,
    c: String,
}

#[derive(Serialize, Deserialize)]
pub struct GuessRequest {
    pub guess: String,
}

#[derive(Serialize)]
pub struct StartResponse {
    pub word_id: usize,
    pub commitment: String,
    pub proof: ProofSerializable,
}

#[derive(Serialize)]
pub struct GuessResponse {
    pub colors: [u8; 5],
    pub proof: ProofSerializable,
}

impl From<Proof<Bn254>> for ProofSerializable {
    fn from(proof: Proof<Bn254>) -> ProofSerializable {
        ProofSerializable {
            a: proof.a.to_string(),
            b: proof.b.to_string(),
            c: proof.c.to_string(),
        }
    }
}
