//! `GET /_smoke`: proves the service reads the database Java seeded (row counts, nothing else).
//! Not part of the Java contract.

use crate::{error::ApiError, AppState};
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Smoke {
    pub airports: i64,
    pub airlines: i64,
    pub flights: i64,
    pub fares: i64,
    pub travelers: i64,
    pub bookings: i64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/_smoke", get(smoke))
}

async fn smoke(State(state): State<AppState>) -> Result<Json<Smoke>, ApiError> {
    let counts = sqlx::query_as::<_, Smoke>(
        "SELECT (SELECT count(*) FROM airport)   AS airports,
                (SELECT count(*) FROM airline)   AS airlines,
                (SELECT count(*) FROM flight)    AS flights,
                (SELECT count(*) FROM fare)      AS fares,
                (SELECT count(*) FROM traveler)  AS travelers,
                (SELECT count(*) FROM booking)   AS bookings",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(counts))
}
