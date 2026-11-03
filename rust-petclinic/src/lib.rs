//! Rust (Axum + sqlx) port of spring-petclinic-rest. Same port layout as the Java service:
//! context path `/petclinic`, REST API under `/petclinic/api`, health at `/petclinic/actuator/health`.

pub mod config;
pub mod db;
pub mod routes;

pub use routes::app;
