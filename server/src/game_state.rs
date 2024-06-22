use ark_bn254::Bn254;
use ark_circom::{read_zkey, CircomConfig};
use ark_groth16::ProvingKey;
use log::{error, info};
use merkle::{hash_word_with_salt, MerklePathEntry};
use num_bigint::{BigUint, RandomBits};
use parking_lot::RwLock;
use rand::{thread_rng, Rng};
use std::{io, sync::Arc, time::Duration};

use crate::word_bank::{self, PickWordResult, WordBank};

const SLEEP_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Debug)]
pub enum Error {
    WordBankCreate(word_bank::Error),
    FileRead(io::Error),
    ZKeyRead,
    MembershipConfigCreate,
    ClueConfigCreate,
    CreateState(merkle::Error),
}

/// All data shared by the game service and HTTP service request handlers.
pub struct SharedState {
    // this state changes per each round
    pub mutable_game_state: RwLock<MutableState>,
    // this state is fixed for the whole lifetime of the server
    pub immutable_proving_state: ImmutableState,
}

/// Contains the gameplay-related data that changes every fixed period of time, including
/// the secret word, and circuit inputs which depend on it. HTTP service uses these as
/// inputs for generating proofs.
#[derive(Clone, Debug)]
pub struct MutableState {
    // Describes the current round of the game. Goes like 0, 1, 2, ..., per each update
    pub word_id: u32,
    // The secret solution word
    pub solution: String,
    // Random salt, sampled every round
    pub salt: BigUint,
    // cm(solution, salt)
    pub commitment: BigUint,
    // Merkle path from H(solution) to root
    pub path: Vec<MerklePathEntry>,
}

/// Contains the shared state that doesn't change. This includes proving keys, circom configs,
/// and WordBank. HTTP service needs these for generating proofs & checking word existence
pub struct ImmutableState {
    pub membership_config: CircomConfig<Bn254>,
    pub clue_config: CircomConfig<Bn254>,
    pub clue_pk: ProvingKey<Bn254>,
    pub membership_pk: ProvingKey<Bn254>,
    pub word_bank: WordBank, // not clonable
}

/// Service which holds the shared state, and updates it every fixed period of time.
pub struct GameStateService {
    shared_state: Arc<SharedState>,
}

impl GameStateService {
    /// Create the service. Creates the word bank, loads the proving keys, and creates the initial shared state.
    pub fn new() -> Result<Self, Error> {
        info!("Creating word bank...");
        let word_bank = WordBank::new().map_err(Error::WordBankCreate)?;

        info!("Setting circom configs and proving keys (this may take a while)...");
        let mut key_file =
            std::fs::File::open("../keys/clue_final.zkey").map_err(Error::FileRead)?;
        let (clue_pk, _matrices) = read_zkey(&mut key_file).map_err(|_| Error::ZKeyRead)?;
        let mut key_file =
            std::fs::File::open("../keys/membership_final.zkey").map_err(Error::FileRead)?;
        let (membership_pk, _matrices) = read_zkey(&mut key_file).map_err(|_| Error::ZKeyRead)?;
        let membership_config = CircomConfig::<Bn254>::new(
            "../proof-membership/membership_js/membership.wasm",
            "../proof-membership/membership.r1cs",
        )
        .map_err(|_| Error::MembershipConfigCreate)?;
        let clue_config = CircomConfig::<Bn254>::new(
            "../proof-clue/clue_js/clue.wasm",
            "../proof-clue/clue.r1cs",
        )
        .map_err(|_| Error::ClueConfigCreate)?;

        info!("Creating initial game state...");
        let game_state = create_game(&word_bank, 0).map_err(Error::CreateState)?;
        let shared_state = SharedState {
            mutable_game_state: RwLock::new(game_state),
            immutable_proving_state: ImmutableState {
                membership_config,
                clue_config,
                clue_pk,
                membership_pk,
                word_bank,
            },
        };

        Ok(GameStateService {
            shared_state: Arc::new(shared_state),
        })
    }

    /// Returns the cloned Arc of the Shared state that other services may want to observe
    pub fn get_state(&self) -> Arc<SharedState> {
        self.shared_state.clone()
    }

    /// Runs the service. Every SLEEP_DURATION game state is updated.
    pub async fn run(mut self) {
        info!(
            "Starting game state service with initial state: {:?}",
            self.shared_state.mutable_game_state.read().clone(),
        );

        loop {
            // update the state every `SLEEP_DURATION`
            tokio::time::sleep(SLEEP_DURATION).await;
            self.update_game_state();
        }
    }

    fn update_game_state(&mut self) {
        match create_game(
            &self.shared_state.immutable_proving_state.word_bank,
            self.shared_state.mutable_game_state.read().word_id + 1,
        ) {
            Ok(game_state) => {
                *self.shared_state.mutable_game_state.write() = game_state;
                info!(
                    "New game state: {:?}",
                    self.shared_state.mutable_game_state.read().clone()
                );
            }
            Err(e) => {
                error!(
                    "Failed to update state, continuing with the same state: {:?}",
                    e
                );
            }
        }
    }
}

/// Creates the game's mutable state, by picking a random word from the word bank, and obtaining
/// the remaining parts (cm, salt, merkle path) accordingly.
fn create_game(word_bank: &WordBank, word_id: u32) -> Result<MutableState, merkle::Error> {
    let PickWordResult {
        word: solution,
        path,
    } = word_bank.pick_word();

    let salt: BigUint = thread_rng().sample(RandomBits::new(256));
    let commitment = hash_word_with_salt(&solution, &salt)?;

    Ok(MutableState {
        word_id,
        solution,
        commitment,
        salt,
        path,
    })
}
