//! Configuration from the same environment variables and defaults as the Java `postgres` profile
//! (`application-postgres.properties`), plus `PORT`.

use std::env;

pub const DEFAULT_POSTGRES_URL: &str = "jdbc:postgresql://localhost:5432/travel";
pub const DEFAULT_PORT: u16 = 9977;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `postgresql://…` URL (the JDBC `jdbc:` prefix is stripped), without credentials.
    pub database_url: String,
    pub user: String,
    pub password: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| env::var(key).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let url = get("POSTGRES_URL").unwrap_or_else(|| DEFAULT_POSTGRES_URL.into());
        let port = match get("PORT") {
            Some(p) => p
                .parse()
                .map_err(|_| format!("PORT is not a port number: {p}"))?,
            None => DEFAULT_PORT,
        };
        Ok(Self {
            database_url: strip_jdbc(&url).to_string(),
            user: get("POSTGRES_USER").unwrap_or_else(|| "travel".into()),
            password: get("POSTGRES_PASS").unwrap_or_else(|| "travel".into()),
            port,
        })
    }
}

fn strip_jdbc(url: &str) -> &str {
    url.strip_prefix("jdbc:").unwrap_or(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_java_postgres_profile() {
        let c = Config::from_lookup(|_| None).unwrap();
        assert_eq!(c.database_url, "postgresql://localhost:5432/travel");
        assert_eq!((c.user.as_str(), c.password.as_str()), ("travel", "travel"));
        assert_eq!(c.port, 9977);
    }

    #[test]
    fn jdbc_prefix_is_optional() {
        let c = Config::from_lookup(|k| match k {
            "POSTGRES_URL" => Some("postgres://db:5433/x".into()),
            "PORT" => Some("8080".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!(c.database_url, "postgres://db:5433/x");
        assert_eq!(c.port, 8080);
    }

    #[test]
    fn bad_port_is_an_error() {
        assert!(Config::from_lookup(|k| (k == "PORT").then(|| "x".into())).is_err());
    }
}
