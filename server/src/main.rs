use game_state::GameStateService;
use log::error;
use tokio::select;

mod game_state;
mod http_service;
mod proofs;
mod request_response;
mod word_bank;

#[tokio::main]
async fn main() {
    env_logger::init();

    let state_service = match GameStateService::new() {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to initialize the game state: {:?}", e);
            return;
        }
    };

    let shared_state = state_service.get_state();
    let state_service_handle = tokio::spawn(async move { state_service.run().await });
    let http_service_handle =
        tokio::spawn(async move { http_service::run("0.0.0.0:8080", shared_state).await });

    select! {
        _ = state_service_handle => {
            error!("State service handle exit early");
        }
        _ = http_service_handle => {
            error!("Service handle exit early");
        }
    }
}
