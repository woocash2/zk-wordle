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

    let state_service = GameStateService::new();
    let shared_state = state_service.get_state();

    let state_service_handle = state_service.run();
    let http_service_handle = http_service::run("0.0.0.0:8080", shared_state);

    select! {
        _ = state_service_handle => {
            error!("State service handle exit early");
        }
        _ = http_service_handle => {
            error!("Service handle exit early");
        }
    }
}
