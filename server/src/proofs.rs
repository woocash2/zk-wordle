use ark_bn254::Bn254;
use ark_circom::{circom::Inputs, CircomBuilder, CircomConfig, CircomReduction};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Groth16, Proof, ProvingKey};
use ark_snark::SNARK;
use merkle::{MerklePathEntry, NodeType};
use num_bigint::{BigInt, BigUint};

pub fn generate_clue_proof(
    guess: String,
    solution: String,
    commitment: BigUint,
    salt: BigUint,
    config: CircomConfig<Bn254>,
    pk: ProvingKey<Bn254>,
) -> (Proof<Bn254>, [u8; 5]) {
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

pub fn generate_membership_proof(
    solution: String,
    commitment: BigUint,
    salt: BigUint,
    path: Vec<MerklePathEntry>,
    config: CircomConfig<Bn254>,
    pk: ProvingKey<Bn254>,
) -> Proof<Bn254> {
    let solution = string_to_bigints(solution);
    let mut builder = CircomBuilder::new(config);

    let mut hashes = Vec::with_capacity(path.len());
    let mut indicators = Vec::with_capacity(path.len());

    for entry in path {
        hashes.push(vec![entry.left.into(), entry.right.into()]);
        indicators.push(match entry.on_path {
            NodeType::Left => 0.into(),
            NodeType::Right => 1.into(),
        });
    }

    builder.push_input("word", Inputs::BigIntVec(solution));
    builder.push_input("salt", Inputs::BigInt(salt.into()));
    builder.push_input("cm", Inputs::BigInt(commitment.into()));
    builder.push_input("hashes", Inputs::BigIntVecVec(hashes));
    builder.push_input("pathIndicators", Inputs::BigIntVec(indicators));

    let circom = builder.build().unwrap();

    // Generate the proof
    let mut rng = rand::thread_rng();
    Groth16::<Bn254, CircomReduction>::prove(&pk, circom, &mut rng).unwrap()
}

fn string_to_bigints(s: String) -> Vec<BigInt> {
    s.as_bytes().iter().map(|x| (x - 97).into()).collect()
}
