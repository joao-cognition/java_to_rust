//! Runtime settings. The database comes from the same variables as the Java `postgres` profile
//! (`application-postgres.properties`), so both services point at the same compose PostgreSQL.

use std::env;
use std::str::FromStr;

use sqlx::postgres::PgConnectOptions;

pub const PORT: u16 = 9977;
/// `server.servlet.context-path` of the Java service.
pub const CONTEXT_PATH: &str = "/petclinic";

const DEFAULT_POSTGRES_URL: &str = "jdbc:postgresql://localhost/petclinic";
const DEFAULT_POSTGRES_USER: &str = "petclinic";
const DEFAULT_POSTGRES_PASS: &str = "petclinic";
/// Above the 64 requests `j2r bench --peak` keeps in flight, so a request never waits for a
/// connection; well under PostgreSQL's default `max_connections` of 100.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 72;

#[derive(Debug, Clone)]
pub struct Config {
    pub db: PgConnectOptions,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let var = |name: &str, default: &str| env::var(name).unwrap_or_else(|_| default.to_string());
        let db = connect_options(
            &var("POSTGRES_URL", DEFAULT_POSTGRES_URL),
            &var("POSTGRES_USER", DEFAULT_POSTGRES_USER),
            &var("POSTGRES_PASS", DEFAULT_POSTGRES_PASS),
        )?;
        let max_connections = match env::var("DB_MAX_CONNECTIONS") {
            Ok(v) => v.parse().map_err(|e| format!("invalid DB_MAX_CONNECTIONS '{v}': {e}"))?,
            Err(_) => DEFAULT_MAX_CONNECTIONS,
        };
        Ok(Self { db, max_connections })
    }
}

/// Accepts the JDBC form the Java profile uses (`jdbc:postgresql://host[:port]/db`), a plain
/// `postgres://` URL, or a bare `host[:port]/db`.
pub fn connect_options(url: &str, user: &str, pass: &str) -> Result<PgConnectOptions, String> {
    let url = url.trim().strip_prefix("jdbc:").unwrap_or(url.trim());
    let url = if url.contains("://") {
        url.to_string()
    } else {
        format!("postgresql://{url}")
    };
    let opts = PgConnectOptions::from_str(&url).map_err(|e| format!("invalid POSTGRES_URL '{url}': {e}"))?;
    Ok(opts.username(user).password(pass))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_java_jdbc_default() {
        let o = connect_options(DEFAULT_POSTGRES_URL, "petclinic", "petclinic").unwrap();
        assert_eq!(o.get_host(), "localhost");
        assert_eq!(o.get_port(), 5432);
        assert_eq!(o.get_database(), Some("petclinic"));
        assert_eq!(o.get_username(), "petclinic");
    }

    #[test]
    fn parses_bare_host_and_port() {
        let o = connect_options("db.internal:6543/clinic", "u", "p").unwrap();
        assert_eq!(o.get_host(), "db.internal");
        assert_eq!(o.get_port(), 6543);
        assert_eq!(o.get_database(), Some("clinic"));
        assert_eq!(o.get_username(), "u");
    }

    #[test]
    fn env_user_wins_over_url_user() {
        let o = connect_options("postgres://other@localhost/petclinic", "petclinic", "x").unwrap();
        assert_eq!(o.get_username(), "petclinic");
    }
}
