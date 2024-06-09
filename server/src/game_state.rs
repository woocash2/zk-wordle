use std::{sync::Arc, time::Duration};

use log::info;
use parking_lot::RwLock;

const SLEEP_DURATION: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct GameState {
    pub solution: String,
    pub salt: String,
    pub commitment: String,
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
    GameState {
        solution: "hello".into(),
        salt: "xyzab".into(),
        commitment: "qwert".into(),
    }
}
