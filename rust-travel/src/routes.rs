//! Read endpoints of the Travel API under `/travel/api`, same parameters, JSON and status codes as the Java
//! controllers. Bookings are read-only here: POST/DELETE stay on the Java service.

use crate::db;
use crate::error::Problem;
use crate::model::{Airport, Cabin, Offer};
use crate::params::{has_text, java_trim, parse_iso_date, parse_java_int, parse_java_long, Params};
use crate::pricing::{distance_km, RouteTaxes};
use axum::extract::{OriginalUri, Path, RawQuery, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{Days, NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

pub const MAX_ADULTS: i32 = 9;

type ApiResult = Result<Response, Problem>;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/travel/api/airports", get(airports))
        .route("/travel/api/airports/{iata}", get(airport))
        .route("/travel/api/airlines", get(airlines))
        .route("/travel/api/airlines/{iata}", get(airline))
        .route("/travel/api/flights", get(flights))
        .route("/travel/api/flights/{id}", get(flight))
        .route("/travel/api/offers", get(offers))
        .route("/travel/api/travelers", get(travelers))
        .route("/travel/api/travelers/{id}", get(traveler))
        .route("/travel/api/bookings", get(bookings))
        .route("/travel/api/bookings/{record_locator}", get(booking))
        .method_not_allowed_fallback(method_not_allowed)
        .fallback(not_found)
        .with_state(pool)
}

fn json<T: Serialize + ?Sized>(value: &T, instance: &str) -> ApiResult {
    let body = serde_json::to_vec(value).map_err(|e| Problem::internal(e, instance))?;
    let mut res = (StatusCode::OK, body).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok(res)
}

/// `ReferenceDataService.normalize`.
fn normalize(code: &str) -> String {
    java_trim(code).to_uppercase()
}

async fn airports(State(pool): State<PgPool>, OriginalUri(uri): OriginalUri) -> ApiResult {
    let at = uri.path();
    let list = db::airports(&pool)
        .await
        .map_err(|e| Problem::internal(e, at))?;
    json(&list, at)
}

async fn airport(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    Path(iata): Path<String>,
) -> ApiResult {
    let at = uri.path();
    match db::airport(&pool, &normalize(&iata))
        .await
        .map_err(|e| Problem::internal(e, at))?
    {
        Some(a) => json(&a, at),
        None => Err(Problem::not_found(format!("Airport {iata} not found"), at)),
    }
}

async fn airlines(State(pool): State<PgPool>, OriginalUri(uri): OriginalUri) -> ApiResult {
    let at = uri.path();
    let list = db::airlines(&pool)
        .await
        .map_err(|e| Problem::internal(e, at))?;
    json(&list, at)
}

async fn airline(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    Path(iata): Path<String>,
) -> ApiResult {
    let at = uri.path();
    match db::airline(&pool, &normalize(&iata))
        .await
        .map_err(|e| Problem::internal(e, at))?
    {
        Some(a) => json(&a, at),
        None => Err(Problem::not_found(format!("Airline {iata} not found"), at)),
    }
}

/// `@RequestParam String name` (required).
fn required_string<'q>(
    q: &'q Params,
    name: &str,
    at: &str,
) -> Result<std::borrow::Cow<'q, str>, Problem> {
    q.string(name).ok_or_else(|| Problem::missing(name, at))
}

/// `@RequestParam @DateTimeFormat(iso = DATE) LocalDate date` (required; blank converts to null = missing).
fn required_date(q: &Params, at: &str) -> Result<NaiveDate, Problem> {
    match q.first("date") {
        Some(v) if has_text(v) => {
            parse_iso_date(v).ok_or_else(|| Problem::conversion("date", v, at))
        }
        _ => Err(Problem::missing("date", at)),
    }
}

/// `FlightService.RouteDay`: both airports and the UTC departure window of one local day at the origin.
struct RouteDay {
    origin: Arc<Airport>,
    destination: Arc<Airport>,
    from_utc: NaiveDateTime,
    to_utc: NaiveDateTime,
}

impl RouteDay {
    async fn resolve(
        pool: &PgPool,
        origin: &str,
        destination: &str,
        date: NaiveDate,
        at: &str,
    ) -> Result<Self, Problem> {
        let (from, to) = db::airport_pair(pool, &normalize(origin), &normalize(destination))
            .await
            .map_err(|e| Problem::internal(e, at))?;
        let from = from
            .ok_or_else(|| Problem::bad_request(format!("Unknown origin airport: {origin}"), at))?;
        let to = to.ok_or_else(|| {
            Problem::bad_request(format!("Unknown destination airport: {destination}"), at)
        })?;
        if from.iata == to.iata {
            return Err(Problem::bad_request(
                "origin and destination must differ",
                at,
            ));
        }
        let from_utc = start_of_day_utc(date, from.tz);
        let to_utc = date
            .checked_add_days(Days::new(1))
            .map(|d| start_of_day_utc(d, from.tz))
            .unwrap_or(NaiveDateTime::MAX);
        Ok(Self {
            origin: Arc::new(from),
            destination: Arc::new(to),
            from_utc,
            to_utc,
        })
    }
}

/// `LocalDate.atStartOfDay(zone)` in UTC: midnight, or the first valid instant after a DST gap.
fn start_of_day_utc(date: NaiveDate, tz: Tz) -> NaiveDateTime {
    let midnight = date.and_hms_opt(0, 0, 0).expect("midnight");
    let mut local = midnight;
    for _ in 0..=48 {
        if let Some(t) = tz.from_local_datetime(&local).earliest() {
            return t.naive_utc();
        }
        local += chrono::Duration::minutes(30);
    }
    midnight
}

async fn flights(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    RawQuery(raw): RawQuery,
) -> ApiResult {
    let at = uri.path();
    let q = Params::parse(raw.as_deref());
    let origin = required_string(&q, "origin", at)?;
    let destination = required_string(&q, "destination", at)?;
    let date = required_date(&q, at)?;
    let day = RouteDay::resolve(&pool, &origin, &destination, date, at).await?;
    let list = db::flights_on_route(
        &pool,
        &day.origin,
        &day.destination,
        day.from_utc,
        day.to_utc,
    )
    .await
    .map_err(|e| Problem::internal(e, at))?;
    json(&list, at)
}

async fn flight(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    Path(id): Path<String>,
) -> ApiResult {
    let at = uri.path();
    let id = parse_java_long(&id).ok_or_else(|| Problem::conversion("id", &id, at))?;
    match db::flight(&pool, id)
        .await
        .map_err(|e| Problem::internal(e, at))?
    {
        Some(f) => json(&f, at),
        None => Err(Problem::not_found(format!("Flight {id} not found"), at)),
    }
}

/// `OfferService.search`: every fare on the route and day priced for `adults`, cheapest first.
async fn offers(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    RawQuery(raw): RawQuery,
) -> ApiResult {
    let at = uri.path();
    let q = Params::parse(raw.as_deref());
    let origin = required_string(&q, "origin", at)?;
    let destination = required_string(&q, "destination", at)?;
    let date = required_date(&q, at)?;
    let adults = match q.first("adults") {
        None | Some("") => 1,
        Some(v) => parse_java_int(v).ok_or_else(|| Problem::conversion("adults", v, at))?,
    };
    let cabin = match q.first("cabin").map(|v| (v, java_trim(v))) {
        None => None,
        Some((_, "")) => None,
        Some((raw, v)) => {
            Some(Cabin::parse(v).ok_or_else(|| Problem::conversion("cabin", raw, at))?)
        }
    };
    if !(1..=MAX_ADULTS).contains(&adults) {
        return Err(Problem::validation(at));
    }

    let day = RouteDay::resolve(&pool, &origin, &destination, date, at).await?;
    let (flights, fares) = db::fares_on_route(
        &pool,
        &day.origin,
        &day.destination,
        day.from_utc,
        day.to_utc,
    )
    .await
    .map_err(|e| Problem::internal(e, at))?;
    let rules = db::tax_rules(&pool)
        .await
        .map_err(|e| Problem::internal(e, at))?;
    let route = RouteTaxes::new(
        &rules,
        &day.origin.country,
        &day.destination.country,
        &day.origin.iata,
    );

    let mut offers: Vec<Offer> = Vec::with_capacity(fares.len());
    let mut current = usize::MAX;
    let mut fuel = rust_decimal::Decimal::ZERO;
    for (flight_idx, fare) in &fares {
        if cabin.is_some_and(|c| c != fare.cabin) || fare.seats_available < adults {
            continue;
        }
        let flight = &flights[*flight_idx];
        if *flight_idx != current {
            current = *flight_idx;
            let (o, d) = (&flight.origin, &flight.destination);
            fuel = route.fuel_surcharge(
                &flight.airline.iata,
                distance_km(o.latitude, o.longitude, d.latitude, d.longitude),
            );
        }
        offers.push(Offer {
            flight,
            fare,
            pricing: route.price(fare, adults, fuel),
        });
    }
    offers.sort_by(|a, b| {
        a.pricing
            .total
            .cmp(&b.pricing.total)
            .then_with(|| a.flight.departure_utc.cmp(&b.flight.departure_utc))
            .then_with(|| a.flight.id.cmp(&b.flight.id))
            .then_with(|| a.fare.booking_class.cmp(&b.fare.booking_class))
    });
    json(&offers, at)
}

/// `@RequestParam @NotBlank String lastName`.
fn last_name<'q>(q: &'q Params, at: &str) -> Result<std::borrow::Cow<'q, str>, Problem> {
    let v = required_string(q, "lastName", at)?;
    if java_trim(&v).is_empty() {
        return Err(Problem::validation(at));
    }
    Ok(v)
}

async fn travelers(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    RawQuery(raw): RawQuery,
) -> ApiResult {
    let at = uri.path();
    let q = Params::parse(raw.as_deref());
    let prefix = last_name(&q, at)?;
    let list = db::travelers_by_last_name(&pool, java_trim(&prefix))
        .await
        .map_err(|e| Problem::internal(e, at))?;
    json(&list, at)
}

async fn traveler(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    Path(id): Path<String>,
) -> ApiResult {
    let at = uri.path();
    let id = parse_java_long(&id).ok_or_else(|| Problem::conversion("id", &id, at))?;
    match db::traveler(&pool, id)
        .await
        .map_err(|e| Problem::internal(e, at))?
    {
        Some(t) => json(&t, at),
        None => Err(Problem::not_found(format!("Traveler {id} not found"), at)),
    }
}

async fn bookings(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    RawQuery(raw): RawQuery,
) -> ApiResult {
    let at = uri.path();
    let q = Params::parse(raw.as_deref());
    let prefix = last_name(&q, at)?;
    let list = db::bookings_by_last_name(&pool, java_trim(&prefix))
        .await
        .map_err(|e| Problem::internal(e, at))?;
    json(&list, at)
}

async fn booking(
    State(pool): State<PgPool>,
    OriginalUri(uri): OriginalUri,
    Path(locator): Path<String>,
) -> ApiResult {
    let at = uri.path();
    let normalized = java_trim(&locator).to_uppercase();
    match db::booking(&pool, &normalized)
        .await
        .map_err(|e| Problem::internal(e, at))?
    {
        Some(b) => json(&b, at),
        None => Err(Problem::not_found(
            format!("Booking {locator} not found"),
            at,
        )),
    }
}

async fn method_not_allowed(method: Method, OriginalUri(uri): OriginalUri) -> Problem {
    Problem::new(
        StatusCode::METHOD_NOT_ALLOWED,
        format!("Method '{method}' is not supported."),
        uri.path(),
    )
}

/// Spring's `NoResourceFoundException` for unmapped paths under the `/travel/` context path.
async fn not_found(OriginalUri(uri): OriginalUri) -> Problem {
    let path = uri.path();
    let resource = path
        .strip_prefix("/travel/")
        .unwrap_or(path.trim_start_matches('/'));
    let resource = resource.strip_suffix('/').unwrap_or(resource);
    Problem::not_found(format!("No static resource {resource}."), path)
}
