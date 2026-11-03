//! Reference data: `GET /airports`, `/airports/{iata}`, `/airlines`, `/airlines/{iata}` (ReferenceDataService).

use crate::db::{normalize_code, AirlineRow, AirportRow, AIRLINE_COLUMNS, AIRPORT_COLUMNS};
use crate::dto::{AirlineDto, AirportDto};
use crate::error::{ApiError, ApiResult};
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use sqlx::PgPool;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/airports", get(list_airports))
        .route("/airports/{iata}", get(get_airport))
        .route("/airlines", get(list_airlines))
        .route("/airlines/{iata}", get(get_airline))
}

async fn list_airports(State(s): State<AppState>) -> ApiResult<Json<Vec<AirportDto>>> {
    let rows: Vec<AirportRow> = sqlx::query_as(&format!("SELECT {AIRPORT_COLUMNS} FROM airport ORDER BY iata"))
        .fetch_all(&s.pool)
        .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

async fn get_airport(State(s): State<AppState>, Path(iata): Path<String>) -> ApiResult<Json<AirportDto>> {
    find_airport(&s.pool, &iata)
        .await?
        .map(|a| Json(a.into()))
        .ok_or_else(|| ApiError::not_found(format!("Airport {iata} not found")))
}

async fn list_airlines(State(s): State<AppState>) -> ApiResult<Json<Vec<AirlineDto>>> {
    let rows: Vec<AirlineRow> = sqlx::query_as(&format!("SELECT {AIRLINE_COLUMNS} FROM airline ORDER BY iata"))
        .fetch_all(&s.pool)
        .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

async fn get_airline(State(s): State<AppState>, Path(iata): Path<String>) -> ApiResult<Json<AirlineDto>> {
    let row: Option<AirlineRow> = sqlx::query_as(&format!("SELECT {AIRLINE_COLUMNS} FROM airline WHERE iata = $1"))
        .bind(normalize_code(&iata))
        .fetch_optional(&s.pool)
        .await?;
    row.map(|a| Json(a.into()))
        .ok_or_else(|| ApiError::not_found(format!("Airline {iata} not found")))
}

/// Airport by (normalized) IATA code.
pub async fn find_airport(pool: &PgPool, iata: &str) -> sqlx::Result<Option<AirportRow>> {
    sqlx::query_as(&format!("SELECT {AIRPORT_COLUMNS} FROM airport WHERE iata = $1"))
        .bind(normalize_code(iata))
        .fetch_optional(pool)
        .await
}
