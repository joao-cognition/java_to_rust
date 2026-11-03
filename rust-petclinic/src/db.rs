//! Connection pool. The schema and seed data belong to the Java service (`schema.sql` /
//! `data.sql` under `db/postgres`); this crate never migrates.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

pub async fn connect(opts: PgConnectOptions, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(max_connections).connect_with(opts).await
}

/// Opens no connection until first use (router tests without a reachable database).
pub fn connect_lazy(opts: PgConnectOptions) -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect_lazy_with(opts)
}
