use std::{sync::Arc, time::Duration};

use ark_bn254::Bn254;
use ark_circom::{read_zkey, CircomConfig};
use ark_groth16::ProvingKey;
use merkle::hash_of_two;
use num_bigint::{BigUint, RandomBits};
use parking_lot::RwLock;
use rand::{thread_rng, Rng};

use crate::word_bank::WordBank;

const SLEEP_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Debug)]
pub struct SharedState {
    pub game_state: GameState,
    pub config: CircomConfig<Bn254>,
    pub pk: ProvingKey<Bn254>,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub word_id: usize,
    pub solution: String,
    pub salt: BigUint,
    pub commitment: BigUint,
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
        let mut key_file = std::fs::File::open("../proof/clue_0001.zkey").unwrap();
        let (pk, _matrices) = read_zkey(&mut key_file).unwrap();
        let config = CircomConfig::<Bn254>::new("../proof/clue_js/clue.wasm", "../proof/clue.r1cs")
            .expect("circom config creation should succeed");

        let game_state = create_game_state(&word_bank, 0);
        let shared_state = SharedState {
            game_state,
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
        let game_sate = create_game_state(
            &self.word_bank,
            self.shared_state.read().game_state.word_id + 1,
        );
        let mut shared_state = self.shared_state.write();
        shared_state.game_state = game_sate;
    }
}

fn create_game_state(word_bank: &WordBank, word_id: usize) -> GameState {
    let solution = word_bank.random_word();
    let salt: BigUint = thread_rng().sample(RandomBits::new(256));
    let merkle_root = word_bank.get_merkle_root();
    let commitment = hash_of_two(&merkle_root.to_string(), &salt.to_string())
        .expect("passed string numbers should fit 256 bits");

    GameState {
        word_id,
        solution,
        salt,
        commitment,
    }
}
