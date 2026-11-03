use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::EnvFilter;

use rust_petclinic::config::Config;
use rust_petclinic::{app, db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(port = config.port, "connecting to database");
    let pool = db::connect(&config.database_url, config.max_connections).await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "rust-petclinic listening");

    axum::serve(listener, app(pool.clone()))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    pool.close().await;
    tracing::info!("shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
