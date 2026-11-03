//! Rust (Axum + sqlx) port of the spring-petclinic-rest read hot path.
//!
//! See `README.md` for the list of endpoints and how to run it next to the Java service.

pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod routes;
pub mod validation;

pub use routes::app;
