/*
 * Copyright (c) 2024 Thomas Preindl
 * MIT License (see LICENSE or https://mit-license.org)
 */

use tokio::signal;
use zk_epdcalc::start_prover_service;

#[tokio::main]
async fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let server_address = "0.0.0.0:3000";
    println!("Starting Server on {}", server_address);

    let (app, handle) = start_prover_service(zk_building_part::builder());

    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    handle
        .await
        .expect("Proving Service Background Task did not terminate correctly!");
}

// From: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
// This function creates a future that waits for a SIGINT (Ctrl+C) signal.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    // This future waits for a SIGTERM signal on Unix systems.
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to Install signal handler")
            .recv()
            .await;
    };

    // This future is a placeholder for non-Unix systems.
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // This line selects the first future that completes, either the Ctrl+C signal or the SIGTERM signal.
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
