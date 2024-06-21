use std::{sync::Arc, time::Duration};

use ark_bn254::Bn254;
use ark_circom::{read_zkey, CircomConfig};
use ark_groth16::ProvingKey;
use merkle::{hash_word_with_salt, MerklePathEntry};
use num_bigint::{BigUint, RandomBits};
use parking_lot::RwLock;
use rand::{thread_rng, Rng};

use crate::word_bank::{PickWordResult, WordBank};

const SLEEP_DURATION: Duration = Duration::from_secs(60 * 60);

pub struct SharedState {
    // this state changes per each round
    pub mutable_game_state: RwLock<MutableState>,
    // this state is fixed for the whole lifetime of the server
    pub immutable_proving_state: ImmutableState,
}

#[derive(Clone, Debug)]
pub struct MutableState {
    pub word_id: u32,
    pub solution: String,
    pub salt: BigUint,
    pub commitment: BigUint,
    pub path: Vec<MerklePathEntry>,
}

pub struct ImmutableState {
    pub membership_config: CircomConfig<Bn254>,
    pub clue_config: CircomConfig<Bn254>,
    pub clue_pk: ProvingKey<Bn254>,
    pub membership_pk: ProvingKey<Bn254>,
    pub word_bank: WordBank, // not clonable
}

pub struct GameStateService {
    shared_state: Arc<SharedState>,
}

impl GameStateService {
    pub fn new() -> Self {
        println!("Creating word bank...");
        let word_bank = WordBank::new().expect("word bank creation should succeed");

        println!("Setting circom configs and proving keys (this may take a while)...");
        let mut key_file = std::fs::File::open("../keys/clue_final.zkey").unwrap();
        let (clue_pk, _matrices) = read_zkey(&mut key_file).unwrap();
        let mut key_file = std::fs::File::open("../keys/membership_final.zkey").unwrap();
        let (membership_pk, _matrices) = read_zkey(&mut key_file).unwrap();
        let membership_config = CircomConfig::<Bn254>::new(
            "../proof-membership/membership_js/membership.wasm",
            "../proof-membership/membership.r1cs",
        )
        .expect("membership config creation should succeed");
        let clue_config = CircomConfig::<Bn254>::new(
            "../proof-clue/clue_js/clue.wasm",
            "../proof-clue/clue.r1cs",
        )
        .expect("clue config creation should succeed");

        let game_state = create_game(&word_bank, 0);
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

        GameStateService {
            shared_state: Arc::new(shared_state),
        }
    }

    pub fn get_state(&self) -> Arc<SharedState> {
        self.shared_state.clone()
    }

    pub async fn run(mut self) {
        println!(
            "Starting game state service with initial state: {:?}",
            self.shared_state.mutable_game_state.read().clone(),
        );

        loop {
            // update the state every `SLEEP_DURATION`
            tokio::time::sleep(SLEEP_DURATION).await;
            self.update_game_state();
            println!(
                "New game state: {:?}",
                self.shared_state.mutable_game_state.read().clone()
            )
        }
    }

    fn update_game_state(&mut self) {
        let game_state = create_game(
            &self.shared_state.immutable_proving_state.word_bank,
            self.shared_state.mutable_game_state.read().word_id + 1,
        );
        *self.shared_state.mutable_game_state.write() = game_state;
    }
}

fn create_game(word_bank: &WordBank, word_id: u32) -> MutableState {
    let PickWordResult {
        word: solution,
        path,
    } = word_bank.pick_word();

    let salt: BigUint = thread_rng().sample(RandomBits::new(256));
    let commitment =
        hash_word_with_salt(&solution, &salt).expect("passed string numbers should fit 256 bits");

    MutableState {
        word_id,
        solution,
        commitment,
        salt,
        path,
    }
}
