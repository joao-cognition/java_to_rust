//! Runtime configuration, read from environment variables.

use std::env;

pub const DEFAULT_DATABASE_URL: &str = "postgres://petclinic:petclinic@localhost:5432/petclinic";
pub const DEFAULT_PORT: u16 = 9977;
pub const DEFAULT_MAX_CONNECTIONS: u32 = 16;

/// Context path shared with the Java service so both are drop-in interchangeable.
pub const CONTEXT_PATH: &str = "/petclinic";

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let port = match env::var("PORT") {
            Ok(v) => v
                .parse::<u16>()
                .map_err(|e| format!("invalid PORT '{v}': {e}"))?,
            Err(_) => DEFAULT_PORT,
        };
        let max_connections = match env::var("DB_MAX_CONNECTIONS") {
            Ok(v) => v
                .parse::<u32>()
                .map_err(|e| format!("invalid DB_MAX_CONNECTIONS '{v}': {e}"))?,
            Err(_) => DEFAULT_MAX_CONNECTIONS,
        };
        Ok(Self {
            database_url,
            port,
            max_connections,
        })
    }
}
