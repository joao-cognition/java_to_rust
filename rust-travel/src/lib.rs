//! Rust port of the read endpoints of the Travel Shopping & Reservation API (`/travel/api`), serving the same JSON
//! contract as the Spring Boot service from the same PostgreSQL database.

pub mod db;
pub mod dto;
pub mod error;
pub mod geo;
pub mod params;
pub mod pricing;
pub mod routes;

use axum::Router;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// The whole service: every route lives under `/travel/api`; anything else is a Spring-style 404 problem.
pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/travel/api", routes::router())
        .fallback(error::no_static_resource)
        .method_not_allowed_fallback(error::method_not_allowed)
        .layer(axum::middleware::from_fn(error::problem_instance))
        .with_state(state)
}
