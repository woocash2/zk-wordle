use log::error;
use tokio::select;

mod http_service;

#[tokio::main]
async fn main() {
    let service_handle = http_service::run("127.0.0.1:3000");

    select! {
        _ = service_handle => {
            error!("Service handle exit early");
        }
    }
}
