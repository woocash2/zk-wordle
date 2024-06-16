use std::{sync::Arc, time::Duration};

use ark_bn254::Bn254;
use ark_circom::{read_zkey, CircomConfig};
use ark_groth16::ProvingKey;
use log::info;
use parking_lot::RwLock;

const SLEEP_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Debug)]
pub struct GameState {
    pub solution: [char; 5],
    pub salt: u32,
    pub commitment: u32,
    pub config: CircomConfig<Bn254>,
    pub pk: ProvingKey<Bn254>,
}

pub struct GameStateService {
    game_state: Arc<RwLock<GameState>>,
}

impl GameStateService {
    pub fn new() -> Self {
        GameStateService {
            game_state: Arc::new(RwLock::new(dummy_state())),
        }
    }

    pub fn get_state(&self) -> Arc<RwLock<GameState>> {
        self.game_state.clone()
    }

    pub async fn run(self) {
        {
            info!(
                "Starting game state service with initial state: {:?}",
                *self.game_state.read()
            );
        }
        loop {
            // update the state every `SLEEP_DURATION`
            tokio::time::sleep(SLEEP_DURATION).await;
            {
                let mut game_state = self.game_state.write();
                *game_state = dummy_state();
            }
            info!("New game state: {:?}", *self.game_state.read())
        }
    }
}

fn dummy_state() -> GameState {
    let mut key_file = std::fs::File::open("../proof/clue_0001.zkey").unwrap();
    let (params, _matrices) = read_zkey(&mut key_file).unwrap();

    GameState {
        solution: ['H', 'E', 'L', 'L', 'O'],
        salt: 1237,
        commitment: 0,
        config: CircomConfig::<Bn254>::new("../proof/clue_js/clue.wasm", "../proof/clue.r1cs")
            .unwrap(),
        pk: params,
    }
}
