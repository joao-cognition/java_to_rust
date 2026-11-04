//! Rust port of the read hot path of the Travel Shopping & Reservation API (Axum + sqlx on PostgreSQL).
//!
//! Serves `/travel/api` on :9977 against the database seeded by the Java service (postgres profile); read-only.

mod db;
mod error;
mod model;
mod params;
mod pricing;
mod routes;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9977);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(&database_url)?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("rust-travel listening on http://{addr}/travel/api");
    axum::serve(listener, routes::router(pool))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
