//! The read endpoints of the Java controllers in `web/`, under `/travel/api`.

use crate::error::{ApiError, ApiResult};
use crate::model::{
    at_airport, utc_timestamp, Airport, BookingDto, Fare, FlightDto, FlightRow, OfferDto,
    SegmentDto, Traveler,
};
use crate::params::{path_long, Params};
use crate::pricing::{self, Route};
use crate::reference::{java_trim, normalize, Reference};
use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

/// `OfferService.MAX_ADULTS`.
pub const MAX_ADULTS: i32 = 9;
const CABINS: &[&str] = &["ECONOMY", "PREMIUM_ECONOMY", "BUSINESS", "FIRST"];

pub struct AppState {
    pub pool: PgPool,
    pub reference: Reference,
}

type AppStateRef = State<Arc<AppState>>;

pub fn api() -> Router<Arc<AppState>> {
    Router::new()
        .route("/airports", get(airports))
        .route("/airports/{iata}", get(airport))
        .route("/airlines", get(airlines))
        .route("/airlines/{iata}", get(airline))
        .route("/flights", get(flights))
        .route("/flights/{id}", get(flight))
        .route("/offers", get(offers))
        .route("/travelers", get(travelers))
        .route("/travelers/{id}", get(traveler))
        .route("/bookings", get(bookings))
        .route("/bookings/{record_locator}", get(booking))
}

fn json<T: serde::Serialize>(value: &T) -> Response {
    Json(value).into_response()
}

// ---- reference data (AirportController, AirlineController, ReferenceDataService) ----

async fn airports(State(s): AppStateRef) -> Response {
    json(&s.reference.airports)
}

async fn airport(State(s): AppStateRef, Path(iata): Path<String>) -> ApiResult<Response> {
    let a = s
        .reference
        .airport(&normalize(&iata))
        .ok_or_else(|| ApiError::not_found(format!("Airport {iata} not found")))?;
    Ok(json(a))
}

async fn airlines(State(s): AppStateRef) -> Response {
    json(&s.reference.airlines)
}

async fn airline(State(s): AppStateRef, Path(iata): Path<String>) -> ApiResult<Response> {
    let a = s
        .reference
        .airline(&normalize(&iata))
        .ok_or_else(|| ApiError::not_found(format!("Airline {iata} not found")))?;
    Ok(json(a))
}

// ---- schedules (FlightController, FlightService) ----

const FLIGHT_COLUMNS: &str =
    "f.id, f.flight_number, f.airline_iata, f.origin_iata, f.destination_iata, \
                              f.departure, f.arrival, f.aircraft, f.duration_minutes";

fn flight_dto<'a>(r: &'a Reference, f: &'a FlightRow) -> ApiResult<FlightDto<'a>> {
    let missing = || ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error");
    let origin = r.airport(&f.origin_iata).ok_or_else(missing)?;
    let destination = r.airport(&f.destination_iata).ok_or_else(missing)?;
    Ok(FlightDto {
        id: f.id,
        flight_number: &f.flight_number,
        airline: r.airline(&f.airline_iata).ok_or_else(missing)?,
        origin,
        destination,
        departure: at_airport(f.departure, origin),
        arrival: at_airport(f.arrival, destination),
        aircraft: &f.aircraft,
        duration_minutes: f.duration_minutes,
    })
}

/// `FlightService.RouteDay`: both airports and the UTC departure window of one local day at the origin.
struct RouteDay<'a> {
    origin: &'a Airport,
    destination: &'a Airport,
    from_utc: NaiveDateTime,
    to_utc: NaiveDateTime,
}

impl<'a> RouteDay<'a> {
    fn of(r: &'a Reference, origin: &str, destination: &str, date: NaiveDate) -> ApiResult<Self> {
        let lookup = |iata: &str, role: &str| {
            r.airport(&normalize(iata))
                .ok_or_else(|| ApiError::bad_request(format!("Unknown {role} airport: {iata}")))
        };
        let from = lookup(origin, "origin")?;
        let to = lookup(destination, "destination")?;
        if from.iata == to.iata {
            return Err(ApiError::bad_request("origin and destination must differ"));
        }
        let next = date
            .checked_add_days(Days::new(1))
            .ok_or_else(|| ApiError::bad_request("date out of range"))?;
        Ok(Self {
            origin: from,
            destination: to,
            from_utc: start_of_day_utc(date, from),
            to_utc: start_of_day_utc(next, from),
        })
    }
}

/// `LocalDate.atStartOfDay(zone)` in UTC: the earliest offset on an overlap, the first instant after a gap.
fn start_of_day_utc(date: NaiveDate, airport: &Airport) -> NaiveDateTime {
    let midnight = date.and_time(NaiveTime::MIN);
    let Some(tz) = airport.tz else {
        return midnight;
    };
    let mut local = midnight;
    for _ in 0..(48 * 60) {
        if let Some(t) = tz.from_local_datetime(&local).earliest() {
            return t.naive_utc();
        }
        local += chrono::Duration::minutes(1);
    }
    midnight
}

async fn flights(State(s): AppStateRef, RawQuery(q): RawQuery) -> ApiResult<Response> {
    let p = Params::parse(q.as_deref());
    let (origin, destination, date) = (
        p.string("origin")?,
        p.string("destination")?,
        p.date("date")?,
    );
    let day = RouteDay::of(&s.reference, &origin, &destination, date)?;
    let rows: Vec<FlightRow> = sqlx::query_as(&format!(
        "SELECT {FLIGHT_COLUMNS} FROM flight f \
         WHERE f.origin_iata = $1::bpchar AND f.destination_iata = $2::bpchar \
           AND f.departure >= $3 AND f.departure < $4 ORDER BY f.departure, f.id"
    ))
    .bind(&day.origin.iata)
    .bind(&day.destination.iata)
    .bind(day.from_utc)
    .bind(day.to_utc)
    .fetch_all(&s.pool)
    .await?;
    let dtos = rows
        .iter()
        .map(|f| flight_dto(&s.reference, f))
        .collect::<ApiResult<Vec<_>>>()?;
    Ok(json(&dtos))
}

async fn flight(State(s): AppStateRef, Path(raw): Path<String>) -> ApiResult<Response> {
    let id = path_long("id", &raw)?;
    let row: Option<FlightRow> = sqlx::query_as(&format!(
        "SELECT {FLIGHT_COLUMNS} FROM flight f WHERE f.id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    let row = row.ok_or_else(|| ApiError::not_found(format!("Flight {id} not found")))?;
    Ok(json(&flight_dto(&s.reference, &row)?))
}

// ---- shopping (OfferController, OfferService, PricingService) — the CPU hot path ----

#[derive(sqlx::FromRow)]
struct FareRow {
    #[sqlx(flatten)]
    flight: FlightRow,
    #[sqlx(flatten)]
    fare: Fare,
}

async fn offers(State(s): AppStateRef, RawQuery(q): RawQuery) -> ApiResult<Response> {
    let p = Params::parse(q.as_deref());
    let origin = p.string("origin")?;
    let destination = p.string("destination")?;
    let date = p.date("date")?;
    let adults = p.int_or("adults", 1)?;
    let cabin = p.optional_enum("cabin", CABINS)?;
    if !(1..=MAX_ADULTS).contains(&adults) {
        return Err(ApiError::validation());
    }

    let r = &s.reference;
    let day = RouteDay::of(r, &origin, &destination, date)?;
    let candidates: Vec<FareRow> = sqlx::query_as(&format!(
        "SELECT {FLIGHT_COLUMNS}, fa.booking_class, fa.cabin, fa.base_amount, fa.currency, fa.refundable, \
                fa.changeable, fa.baggage_included_kg, fa.seats_available \
         FROM fare fa JOIN flight f ON f.id = fa.flight_id \
         WHERE f.origin_iata = $1::bpchar AND f.destination_iata = $2::bpchar \
           AND f.departure >= $3 AND f.departure < $4 \
         ORDER BY f.departure, f.id, fa.booking_class"
    ))
    .bind(&day.origin.iata)
    .bind(&day.destination.iata)
    .bind(day.from_utc)
    .bind(day.to_utc)
    .fetch_all(&s.pool)
    .await?;

    struct Priced<'a> {
        total: Decimal,
        row: &'a FareRow,
        pricing: crate::model::Pricing,
    }
    let mut priced = Vec::with_capacity(candidates.len());
    let mut current: Option<(i64, f64)> = None;
    for row in &candidates {
        if cabin.is_some_and(|c| c != row.fare.cabin) || row.fare.seats_available < adults {
            continue;
        }
        let f = &row.flight;
        let (origin, destination) = (
            r.airport(&f.origin_iata).unwrap_or(day.origin),
            r.airport(&f.destination_iata).unwrap_or(day.destination),
        );
        let distance_km = match current {
            Some((id, d)) if id == f.id => d,
            _ => {
                let d = pricing::distance_km(origin, destination);
                current = Some((f.id, d));
                d
            }
        };
        let route = Route {
            origin,
            destination,
            airline_iata: &f.airline_iata,
            distance_km,
        };
        let pricing = pricing::price(&row.fare, adults, &r.tax_rules, &route);
        priced.push(Priced {
            total: pricing.total,
            row,
            pricing,
        });
    }
    priced.sort_by(|a, b| {
        a.total
            .cmp(&b.total)
            .then_with(|| a.row.flight.departure.cmp(&b.row.flight.departure))
            .then_with(|| a.row.flight.id.cmp(&b.row.flight.id))
            .then_with(|| a.row.fare.booking_class.cmp(&b.row.fare.booking_class))
    });

    let mut dtos = Vec::with_capacity(priced.len());
    for o in priced {
        dtos.push(OfferDto {
            offer_id: format!("{}-{}", o.row.flight.id, o.row.fare.booking_class),
            flight: flight_dto(r, &o.row.flight)?,
            fare: &o.row.fare,
            pricing: o.pricing,
        });
    }
    Ok(json(&dtos))
}

// ---- travelers (TravelerController, TravelerService) ----

const TRAVELER_COLUMNS: &str =
    "t.id, t.first_name, t.last_name, t.date_of_birth, t.email, t.nationality, t.frequent_flyer_number";

async fn traveler(State(s): AppStateRef, Path(raw): Path<String>) -> ApiResult<Response> {
    let id = path_long("id", &raw)?;
    let t: Option<Traveler> = sqlx::query_as(&format!(
        "SELECT {TRAVELER_COLUMNS} FROM traveler t WHERE t.id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    Ok(json(&t.ok_or_else(|| {
        ApiError::not_found(format!("Traveler {id} not found"))
    })?))
}

/// Spring Data `StartingWithIgnoreCase`: the prefix is escaped, then matched case-insensitively.
async fn travelers(State(s): AppStateRef, RawQuery(q): RawQuery) -> ApiResult<Response> {
    let last_name = Params::parse(q.as_deref()).not_blank("lastName")?;
    let mut pattern = String::new();
    for c in java_trim(&last_name).chars() {
        if matches!(c, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    let found: Vec<Traveler> = sqlx::query_as(&format!(
        "SELECT {TRAVELER_COLUMNS} FROM traveler t WHERE upper(t.last_name) LIKE upper($1) ESCAPE '\\' \
         ORDER BY t.last_name, t.first_name"
    ))
    .bind(pattern)
    .fetch_all(&s.pool)
    .await?;
    Ok(json(&found))
}

// ---- reservations (BookingController, BookingService read methods) ----

#[derive(sqlx::FromRow)]
struct BookingRow {
    id: i64,
    record_locator: String,
    status: String,
    created_at: NaiveDateTime,
    total_amount: Decimal,
    currency: String,
}

#[derive(sqlx::FromRow)]
struct BookingTravelerRow {
    booking_id: i64,
    #[sqlx(flatten)]
    traveler: Traveler,
}

#[derive(sqlx::FromRow)]
struct SegmentRow {
    booking_id: i64,
    segment_booking_class: String,
    segment_cabin: String,
    #[sqlx(flatten)]
    flight: FlightRow,
}

const BOOKING_COLUMNS: &str =
    "b.id, b.record_locator, b.status, b.created_at, b.total_amount, b.currency";

/// A page of bookings with their travelers (ordered by id) and segments (ordered by id), like Hibernate's
/// `@OrderBy("id")` subselect fetches.
struct LoadedBookings {
    rows: Vec<BookingRow>,
    travelers: Vec<BookingTravelerRow>,
    segments: Vec<SegmentRow>,
}

impl LoadedBookings {
    async fn load(pool: &PgPool, rows: Vec<BookingRow>) -> ApiResult<Self> {
        let ids: Vec<i64> = rows.iter().map(|b| b.id).collect();
        let travelers = sqlx::query_as(&format!(
            "SELECT bt.booking_id, {TRAVELER_COLUMNS} FROM booking_traveler bt \
             JOIN traveler t ON t.id = bt.traveler_id WHERE bt.booking_id = ANY($1) ORDER BY t.id"
        ))
        .bind(&ids)
        .fetch_all(pool)
        .await?;
        let segments = sqlx::query_as(&format!(
            "SELECT s.booking_id, s.booking_class AS segment_booking_class, s.cabin AS segment_cabin, \
                    {FLIGHT_COLUMNS} \
             FROM booking_segment s JOIN fare fa ON fa.id = s.fare_id JOIN flight f ON f.id = fa.flight_id \
             WHERE s.booking_id = ANY($1) ORDER BY s.id"
        ))
        .bind(&ids)
        .fetch_all(pool)
        .await?;
        Ok(Self {
            rows,
            travelers,
            segments,
        })
    }

    fn dtos<'a>(&'a self, r: &'a Reference) -> ApiResult<Vec<BookingDto<'a>>> {
        let mut travelers_by: HashMap<i64, Vec<&Traveler>> = HashMap::new();
        for t in &self.travelers {
            travelers_by
                .entry(t.booking_id)
                .or_default()
                .push(&t.traveler);
        }
        let mut segments_by: HashMap<i64, Vec<SegmentDto<'a>>> = HashMap::new();
        for seg in &self.segments {
            segments_by
                .entry(seg.booking_id)
                .or_default()
                .push(SegmentDto {
                    flight: flight_dto(r, &seg.flight)?,
                    booking_class: &seg.segment_booking_class,
                    cabin: &seg.segment_cabin,
                });
        }
        Ok(self
            .rows
            .iter()
            .map(|b| BookingDto {
                record_locator: &b.record_locator,
                status: &b.status,
                created_at: utc_timestamp(b.created_at),
                travelers: travelers_by.remove(&b.id).unwrap_or_default(),
                segments: segments_by.remove(&b.id).unwrap_or_default(),
                total_amount: b.total_amount,
                currency: &b.currency,
            })
            .collect())
    }
}

async fn booking(State(s): AppStateRef, Path(record_locator): Path<String>) -> ApiResult<Response> {
    let row: Option<BookingRow> = sqlx::query_as(&format!(
        "SELECT {BOOKING_COLUMNS} FROM booking b WHERE b.record_locator = $1::bpchar"
    ))
    .bind(java_trim(&record_locator).to_uppercase())
    .fetch_optional(&s.pool)
    .await?;
    let row =
        row.ok_or_else(|| ApiError::not_found(format!("Booking {record_locator} not found")))?;
    let loaded = LoadedBookings::load(&s.pool, vec![row]).await?;
    let dtos = loaded.dtos(&s.reference)?;
    Ok(json(&dtos[0]))
}

/// Same JPQL as `BookingRepository.findByTravelerLastNamePrefix`: the prefix is NOT escaped there.
async fn bookings(State(s): AppStateRef, RawQuery(q): RawQuery) -> ApiResult<Response> {
    let last_name = Params::parse(q.as_deref()).not_blank("lastName")?;
    let rows: Vec<BookingRow> = sqlx::query_as(&format!(
        "SELECT DISTINCT {BOOKING_COLUMNS} FROM booking b \
         JOIN booking_traveler bt ON bt.booking_id = b.id JOIN traveler t ON t.id = bt.traveler_id \
         WHERE lower(t.last_name) LIKE lower($1 || '%') ORDER BY b.created_at DESC, b.id DESC"
    ))
    .bind(java_trim(&last_name))
    .fetch_all(&s.pool)
    .await?;
    let loaded = LoadedBookings::load(&s.pool, rows).await?;
    Ok(json(&loaded.dtos(&s.reference)?))
}
