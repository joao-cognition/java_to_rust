//! travelers and bookings (reads): `TravelerController` / `TravelerService`, `BookingController.get|search` /
//! `BookingService`. Bookings are loaded in three queries (bookings, then travelers and segments for all of them).

use crate::db::{like_prefix_escaped, FlightRow, TravelerRow, FLIGHT_COLUMNS, FLIGHT_JOINS, TRAVELER_COLUMNS};
use crate::dto::{utc_offset, BookingDto, SegmentDto, TravelerDto};
use crate::error::{ApiError, ApiResult};
use crate::params::{path_long, Params};
use crate::AppState;
use axum::extract::{Path, RawQuery, State};
use axum::routing::get;
use axum::{Json, Router};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/travelers", get(search_travelers))
        .route("/travelers/{id}", get(get_traveler))
        .route("/bookings", get(search_bookings))
        .route("/bookings/{record_locator}", get(get_booking))
}

async fn get_traveler(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<TravelerDto>> {
    let id = path_long("id", &id)?;
    let sql = format!("SELECT {TRAVELER_COLUMNS} FROM traveler t WHERE t.id = $1");
    let row = sqlx::query_as::<_, TravelerRow>(&sql).bind(id).fetch_optional(&state.pool).await?;
    row.map(|t| Json(t.into()))
        .ok_or_else(|| ApiError::not_found(format!("Traveler {id} not found")))
}

/// `findByLastNameStartingWithIgnoreCaseOrderByLastNameAscFirstNameAsc(lastName.trim())`: `%`, `_` and `\` are
/// literal (Spring Data escapes them), comparison is `upper(...) like upper(...)` as Hibernate renders it.
async fn search_travelers(State(state): State<AppState>, RawQuery(query): RawQuery) -> ApiResult<Json<Vec<TravelerDto>>> {
    let last_name = Params::parse(query.as_deref()).required_not_blank("lastName")?;
    let sql = format!(
        "SELECT {TRAVELER_COLUMNS} FROM traveler t WHERE upper(t.last_name) LIKE upper($1) ESCAPE '\\' \
         ORDER BY t.last_name, t.first_name"
    );
    let rows = sqlx::query_as::<_, TravelerRow>(&sql)
        .bind(like_prefix_escaped(last_name.trim()))
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows.into_iter().map(TravelerDto::from).collect()))
}

/// `BookingService.get`: the locator is trimmed and upper-cased for the lookup, the 404 detail keeps the raw value.
async fn get_booking(State(state): State<AppState>, Path(record_locator): Path<String>) -> ApiResult<Json<BookingDto>> {
    let rows = sqlx::query_as::<_, BookingRow>(&format!("SELECT {BOOKING_COLUMNS} FROM booking b WHERE b.record_locator = $1"))
        .bind(record_locator.trim().to_uppercase())
        .fetch_all(&state.pool)
        .await?;
    let mut found = load_bookings(&state.pool, rows).await?;
    found
        .pop()
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("Booking {record_locator} not found")))
}

/// `BookingRepository.findByTravelerLastNamePrefix(lastName.trim())`: the JPQL is
/// `lower(t.lastName) like lower(concat(:prefix, '%'))` without escaping, so `%`/`_` are wildcards here (unlike
/// `/travelers`) and PostgreSQL's default `\` escape applies; distinct bookings, newest `createdAt` first, then id desc.
async fn search_bookings(State(state): State<AppState>, RawQuery(query): RawQuery) -> ApiResult<Json<Vec<BookingDto>>> {
    let last_name = Params::parse(query.as_deref()).required_not_blank("lastName")?;
    let sql = format!(
        "SELECT {BOOKING_COLUMNS} FROM booking b WHERE EXISTS (\
           SELECT 1 FROM booking_traveler bt JOIN traveler t ON t.id = bt.traveler_id \
           WHERE bt.booking_id = b.id AND lower(t.last_name) LIKE lower(($1 || '%'))) \
         ORDER BY b.created_at DESC, b.id DESC"
    );
    let rows = sqlx::query_as::<_, BookingRow>(&sql)
        .bind(last_name.trim())
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(load_bookings(&state.pool, rows).await?))
}

const BOOKING_COLUMNS: &str = "b.id, b.record_locator, b.status, b.created_at, b.total_amount, b.currency";

#[derive(Debug, Clone, FromRow)]
pub struct BookingRow {
    pub id: i64,
    pub record_locator: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub total_amount: Decimal,
    pub currency: String,
}

#[derive(Debug, FromRow)]
struct BookingTravelerRow {
    booking_id: i64,
    #[sqlx(flatten)]
    traveler: TravelerRow,
}

#[derive(Debug, FromRow)]
struct SegmentRow {
    s_booking_id: i64,
    s_booking_class: String,
    s_cabin: String,
    #[sqlx(flatten)]
    flight: FlightRow,
}

/// `ApiMapper.booking` for each row, keeping their order. Travelers and segments are both `@OrderBy("id")`.
pub async fn load_bookings(pool: &PgPool, bookings: Vec<BookingRow>) -> ApiResult<Vec<BookingDto>> {
    if bookings.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<i64> = bookings.iter().map(|b| b.id).collect();
    let travelers = sqlx::query_as::<_, BookingTravelerRow>(&format!(
        "SELECT bt.booking_id, {TRAVELER_COLUMNS} FROM booking_traveler bt JOIN traveler t ON t.id = bt.traveler_id \
         WHERE bt.booking_id = ANY($1) ORDER BY bt.booking_id, t.id"
    ))
    .bind(&ids)
    .fetch_all(pool)
    .await?;
    let segments = sqlx::query_as::<_, SegmentRow>(&format!(
        "SELECT s.booking_id AS s_booking_id, s.booking_class AS s_booking_class, s.cabin AS s_cabin, {FLIGHT_COLUMNS} \
         FROM {FLIGHT_JOINS} JOIN fare fa ON fa.flight_id = f.id JOIN booking_segment s ON s.fare_id = fa.id \
         WHERE s.booking_id = ANY($1) ORDER BY s.booking_id, s.id"
    ))
    .bind(&ids)
    .fetch_all(pool)
    .await?;
    Ok(assemble(bookings, travelers, segments))
}

fn assemble(bookings: Vec<BookingRow>, travelers: Vec<BookingTravelerRow>, segments: Vec<SegmentRow>) -> Vec<BookingDto> {
    let mut pax: HashMap<i64, Vec<TravelerDto>> = HashMap::new();
    for row in travelers {
        pax.entry(row.booking_id).or_default().push(row.traveler.into());
    }
    let mut legs: HashMap<i64, Vec<SegmentDto>> = HashMap::new();
    for row in segments {
        let segment = SegmentDto {
            flight: row.flight.to_dto(),
            booking_class: row.s_booking_class,
            cabin: row.s_cabin,
        };
        legs.entry(row.s_booking_id).or_default().push(segment);
    }
    bookings
        .into_iter()
        .map(|b| BookingDto {
            travelers: pax.remove(&b.id).unwrap_or_default(),
            segments: legs.remove(&b.id).unwrap_or_default(),
            record_locator: b.record_locator,
            status: b.status,
            created_at: utc_offset(b.created_at),
            total_amount: b.total_amount,
            currency: b.currency,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};

    fn booking(id: i64, locator: &str) -> BookingRow {
        BookingRow {
            id,
            record_locator: locator.into(),
            status: "CONFIRMED".into(),
            created_at: NaiveDateTime::parse_from_str("2026-10-22 22:30:22", "%Y-%m-%d %H:%M:%S").unwrap(),
            total_amount: Decimal::new(38400, 2),
            currency: "EUR".into(),
        }
    }

    fn traveler(booking_id: i64, id: i64) -> BookingTravelerRow {
        BookingTravelerRow {
            booking_id,
            traveler: TravelerRow {
                id,
                first_name: format!("F{id}"),
                last_name: format!("L{id}"),
                date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 1).unwrap(),
                email: format!("t{id}@example.com"),
                nationality: "FR".into(),
                frequent_flyer_number: None,
            },
        }
    }

    #[test]
    fn assemble_keeps_booking_order_and_groups_travelers() {
        let out = assemble(
            vec![booking(2, "BBBBBB"), booking(1, "AAAAAA")],
            vec![traveler(1, 5), traveler(1, 7), traveler(2, 6)],
            vec![],
        );
        assert_eq!(
            out.iter().map(|b| b.record_locator.as_str()).collect::<Vec<_>>(),
            ["BBBBBB", "AAAAAA"]
        );
        assert_eq!(out[0].travelers.iter().map(|t| t.id).collect::<Vec<_>>(), [6]);
        assert_eq!(out[1].travelers.iter().map(|t| t.id).collect::<Vec<_>>(), [5, 7]);
        assert!(out[1].segments.is_empty());
        let json = serde_json::to_value(&out[1]).unwrap();
        assert_eq!(json["createdAt"], "2026-10-22T22:30:22+00:00");
        assert_eq!(serde_json::to_string(&json["totalAmount"]).unwrap(), "384.00");
    }
}
