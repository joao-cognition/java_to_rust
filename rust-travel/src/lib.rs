//! Rust (Axum) port of the Travel Shopping & Reservation API read path: the same `/travel/api` contract as the
//! Java service (:9966), served on :9977 against the same PostgreSQL database. Read-only: the Java service owns
//! the schema and the seed data, so start it first.

pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod pricing;
pub mod routes;

use axum::Router;
use sqlx::PgPool;

/// Shared state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// The whole service: `/travel/api/*` plus the RFC 9457 fallback for unknown paths.
pub fn app(pool: PgPool) -> Router {
    Router::new()
        .nest("/travel/api", routes::api())
        .fallback(error::no_resource)
        .with_state(AppState { pool })
}
