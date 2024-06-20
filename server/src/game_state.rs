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

#[derive(Clone, Debug)]
pub struct SharedState {
    pub game_state: GameState,
    pub cm_state: CmState,
    pub merkle_state: MerkleState,
    pub config: CircomConfig<Bn254>,
    pub pk: ProvingKey<Bn254>,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub word_id: usize,
    pub solution: String,
}

#[derive(Clone, Debug)]
pub struct CmState {
    pub salt: BigUint,
    pub commitment: BigUint,
}

#[derive(Clone, Debug)]
pub struct MerkleState {
    pub path: Vec<MerklePathEntry>,
    pub root_hash: BigUint,
}

pub struct GameStateService {
    shared_state: Arc<RwLock<SharedState>>,
    word_bank: WordBank,
}

impl GameStateService {
    pub fn new() -> Self {
        println!("Creating word bank...");
        let word_bank = WordBank::new().expect("word bank creation should succeed");

        println!("Setting circom config and proving key...");
        let mut key_file = std::fs::File::open("../proof/circuit_final.zkey").unwrap();
        let (pk, _matrices) = read_zkey(&mut key_file).unwrap();
        let config = CircomConfig::<Bn254>::new("../proof/clue_js/clue.wasm", "../proof/clue.r1cs")
            .expect("circom config creation should succeed");

        let (game_state, cm_state, merkle_state) = create_game(&word_bank, 0);
        let shared_state = SharedState {
            game_state,
            cm_state,
            merkle_state,
            config,
            pk,
        };

        GameStateService {
            shared_state: Arc::new(RwLock::new(shared_state)),
            word_bank,
        }
    }

    pub fn get_state(&self) -> Arc<RwLock<SharedState>> {
        self.shared_state.clone()
    }

    pub async fn run(mut self) {
        println!(
            "Starting game state service with initial state: {:?}",
            self.shared_state.read().game_state.clone(),
        );

        loop {
            // update the state every `SLEEP_DURATION`
            tokio::time::sleep(SLEEP_DURATION).await;
            self.update_game_state();
            println!(
                "New game state: {:?}",
                self.shared_state.read().game_state.clone()
            )
        }
    }

    fn update_game_state(&mut self) {
        let (game_sate, cm_state, merkle_state) = create_game(
            &self.word_bank,
            self.shared_state.read().game_state.word_id + 1,
        );
        let mut shared_state = self.shared_state.write();
        shared_state.game_state = game_sate;
        shared_state.cm_state = cm_state;
        shared_state.merkle_state = merkle_state;
    }
}

fn create_game(word_bank: &WordBank, word_id: usize) -> (GameState, CmState, MerkleState) {
    let PickWordResult {
        word: solution,
        path,
        root_hash,
    } = word_bank.pick_word();

    let salt: BigUint = thread_rng().sample(RandomBits::new(256));
    let commitment =
        hash_word_with_salt(&solution, &salt).expect("passed string numbers should fit 256 bits");

    (
        GameState { word_id, solution },
        CmState { commitment, salt },
        MerkleState { path, root_hash },
    )
}
