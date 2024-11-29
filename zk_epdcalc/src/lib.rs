use crate::handlers::{create_epd, get_epd_creation, get_epd_result, post_verify_epd, start_epd_creation};
use crate::proving::ProvingService;
use crate::requests::ConfigFactoryMap;
use axum::extract::FromRef;
use axum::routing::{get, post};
use axum::Router;
use tokio::task::JoinHandle;

pub use requests::{new_config_factory, ConfigFactory};

mod error;
mod handlers;
mod proving;
mod requests;
mod verify;

pub fn start_prover_service(
    proof_systems: Vec<(&'static str, Box<dyn ConfigFactory + Sync + Send>)>,
) -> (Router, JoinHandle<()>) {
    let (proving_service, handle) = ProvingService::new();

    let config_factory_map = ConfigFactoryMap::new(proof_systems);
    let app_state = AppState {
        config_factory_map,
        proving_service,
    };

    let router = Router::new()
        .route("/create", post(create_epd))
        .route("/creation", post(start_epd_creation))
        .route("/creation/:id", get(get_epd_creation))
        .route("/creation/:id/result", get(get_epd_result))
        .route("/verify", post(post_verify_epd))
        .with_state(app_state);
    (router, handle)
}

#[derive(Clone, FromRef)]
struct AppState {
    proving_service: ProvingService,
    config_factory_map: ConfigFactoryMap,
}
