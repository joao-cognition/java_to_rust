//! Reference data (airports, airlines, tax rules) comes from the static `data.sql`, which the Java service applies
//! at startup and never changes at runtime, so the Rust service loads it once and serves it from memory.

use crate::model::{Airline, Airport, TaxRule};
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;

pub struct Reference {
    pub airports: Vec<Airport>,
    pub airlines: Vec<Airline>,
    pub tax_rules: Vec<TaxRule>,
    airport_index: HashMap<String, usize>,
    airline_index: HashMap<String, usize>,
}

impl Reference {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let mut airports: Vec<Airport> = sqlx::query_as(
            "SELECT iata, icao, name, city, country, timezone, latitude, longitude FROM airport ORDER BY iata",
        )
        .fetch_all(pool)
        .await?;
        for a in &mut airports {
            a.tz = a.timezone.parse().ok();
            if a.tz.is_none() {
                tracing::warn!("unknown timezone {} for airport {}", a.timezone, a.iata);
            }
        }
        let airlines: Vec<Airline> =
            sqlx::query_as("SELECT iata, icao, name, country, alliance FROM airline ORDER BY iata")
                .fetch_all(pool)
                .await?;
        let tax_rules: Vec<TaxRule> = sqlx::query_as(
            "SELECT kind, country, airport_iata, airline_iata, min_distance_km, max_distance_km, amount \
             FROM tax_rule ORDER BY code",
        )
        .fetch_all(pool)
        .await?;
        let airport_index = airports
            .iter()
            .enumerate()
            .map(|(i, a)| (a.iata.clone(), i))
            .collect();
        let airline_index = airlines
            .iter()
            .enumerate()
            .map(|(i, a)| (a.iata.clone(), i))
            .collect();
        Ok(Self {
            airports,
            airlines,
            tax_rules,
            airport_index,
            airline_index,
        })
    }

    /// Waits until the Java service has finished applying `data.sql`. Its statements commit one by one, so the
    /// reference tables are only known to be complete once the demo generator, which runs afterwards, has written
    /// flights.
    pub async fn load_when_ready(pool: &PgPool) -> Self {
        loop {
            let seeded = sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM flight)")
                .fetch_one(pool)
                .await;
            match seeded {
                Ok(true) => match Self::load(pool).await {
                    Ok(r) => return r,
                    Err(e) => tracing::warn!("reference data not readable yet ({e}); retrying"),
                },
                Ok(false) => tracing::warn!(
                    "flight table is empty; waiting for the Java service to finish seeding"
                ),
                Err(e) => tracing::warn!("reference data not readable yet ({e}); retrying"),
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn airport(&self, iata: &str) -> Option<&Airport> {
        self.airport_index.get(iata).map(|&i| &self.airports[i])
    }

    pub fn airline(&self, iata: &str) -> Option<&Airline> {
        self.airline_index.get(iata).map(|&i| &self.airlines[i])
    }
}

/// `ReferenceDataService.normalize` (`trim().toUpperCase(Locale.ROOT)`).
pub fn normalize(code: &str) -> String {
    java_trim(code).to_uppercase()
}

/// `String.trim()`: strips leading/trailing chars `<= ' '` (not Unicode whitespace).
pub fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= ' ')
}
