//! PostgreSQL access. Queries are runtime-checked (`sqlx::query_as`), so builds never need a live database.
//! Read-only: the schema and data belong to the Java service.

use crate::config::Config;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::{str::FromStr, time::Duration};

/// Sized for `j2r bench --peak 10`, which keeps 64 requests in flight.
pub const MAX_CONNECTIONS: u32 = 64;

pub fn connect_options(config: &Config) -> Result<PgConnectOptions, sqlx::Error> {
    Ok(PgConnectOptions::from_str(&config.database_url)?
        .username(&config.user)
        .password(&config.password))
}

pub async fn connect(config: &Config) -> Result<PgPool, sqlx::Error> {
    connect_with(connect_options(config)?, Duration::from_secs(30)).await
}

pub async fn connect_with(
    options: PgConnectOptions,
    acquire_timeout: Duration,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .min_connections(4)
        .acquire_timeout(acquire_timeout)
        .connect_with(options)
        .await
}
