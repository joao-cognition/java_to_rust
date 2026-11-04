//! Rust (Axum) port of the read endpoints of the Travel Shopping & Reservation API. Same contract as the Java
//! service (`/travel/api` on :9966), served on :9977 against the same PostgreSQL database, read-only.

mod error;
mod model;
mod params;
mod pricing;
mod reference;
mod routes;

use axum::{middleware, Router};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9977);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    let reference = reference::Reference::load_when_ready(&pool).await;
    tracing::info!(
        "reference data: {} airports, {} airlines, {} tax rules",
        reference.airports.len(),
        reference.airlines.len(),
        reference.tax_rules.len()
    );

    let state = Arc::new(routes::AppState { pool, reference });
    let app = Router::new()
        .nest("/travel/api", routes::api())
        .fallback(error::no_resource)
        .layer(middleware::from_fn(error::render_problems))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("rust-travel listening on :{port}/travel/api");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
