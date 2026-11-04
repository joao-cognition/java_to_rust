//! Read-only queries against the Java service's PostgreSQL schema, mirroring the JPA repository semantics.

use crate::model::{Airline, Airport, Booking, Cabin, Fare, Flight, Segment, Traveler};
use crate::pricing::{TaxKind, TaxRule};
use chrono::NaiveDateTime;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;

type Result<T> = std::result::Result<T, sqlx::Error>;

const AIRPORT: &str = "iata, icao, name, city, country, timezone, latitude, longitude";
const AIRLINE: &str = "iata, icao, name, country, alliance";
const TRAVELER: &str = "t.id, t.first_name, t.last_name, t.date_of_birth, t.email, t.nationality, t.frequent_flyer_number";
/// Flight with airline and both airports (Hibernate's eager `@ManyToOne`s); columns 0..27.
const FLIGHT_JOINED: &str =
    "f.id, f.flight_number, f.departure, f.arrival, f.aircraft, f.duration_minutes, \
     al.iata, al.icao, al.name, al.country, al.alliance, \
     o.iata, o.icao, o.name, o.city, o.country, o.timezone, o.latitude, o.longitude, \
     d.iata, d.icao, d.name, d.city, d.country, d.timezone, d.latitude, d.longitude";
const FLIGHT_JOINS: &str = "JOIN airline al ON al.iata = f.airline_iata \
     JOIN airport o ON o.iata = f.origin_iata JOIN airport d ON d.iata = f.destination_iata";

fn decode_err(what: &str, value: &str) -> sqlx::Error {
    sqlx::Error::Decode(format!("unknown {what} {value:?}").into())
}

fn airport_at(row: &PgRow, i: usize) -> Result<Airport> {
    let timezone: String = row.try_get(i + 5)?;
    let tz = timezone
        .parse()
        .map_err(|_| decode_err("timezone", &timezone))?;
    Ok(Airport {
        iata: row.try_get(i)?,
        icao: row.try_get(i + 1)?,
        name: row.try_get(i + 2)?,
        city: row.try_get(i + 3)?,
        country: row.try_get(i + 4)?,
        timezone,
        latitude: row.try_get(i + 6)?,
        longitude: row.try_get(i + 7)?,
        tz,
    })
}

fn airline_at(row: &PgRow, i: usize) -> Result<Airline> {
    Ok(Airline {
        iata: row.try_get(i)?,
        icao: row.try_get(i + 1)?,
        name: row.try_get(i + 2)?,
        country: row.try_get(i + 3)?,
        alliance: row.try_get(i + 4)?,
    })
}

fn cabin_at(row: &PgRow, i: usize) -> Result<Cabin> {
    let s: &str = row.try_get(i)?;
    Cabin::parse(s).ok_or_else(|| decode_err("cabin", s))
}

/// Flight columns 0..11 (flight + airline) with the airports given by the caller.
fn flight_at(row: &PgRow, origin: Arc<Airport>, destination: Arc<Airport>) -> Result<Flight> {
    Ok(Flight {
        id: row.try_get(0)?,
        flight_number: row.try_get(1)?,
        departure_utc: row.try_get(2)?,
        arrival_utc: row.try_get(3)?,
        aircraft: row.try_get(4)?,
        duration_minutes: row.try_get(5)?,
        airline: airline_at(row, 6)?,
        origin,
        destination,
    })
}

/// A `FLIGHT_JOINED` row starting at column `i`.
fn joined_flight_at(row: &PgRow) -> Result<Flight> {
    let origin = Arc::new(airport_at(row, 11)?);
    let destination = Arc::new(airport_at(row, 19)?);
    flight_at(row, origin, destination)
}

fn traveler_at(row: &PgRow, i: usize) -> Result<Traveler> {
    Ok(Traveler {
        id: row.try_get(i)?,
        first_name: row.try_get(i + 1)?,
        last_name: row.try_get(i + 2)?,
        date_of_birth: row.try_get(i + 3)?,
        email: row.try_get(i + 4)?,
        nationality: row.try_get(i + 5)?,
        frequent_flyer_number: row.try_get(i + 6)?,
    })
}

pub async fn airports(pool: &PgPool) -> Result<Vec<Airport>> {
    let sql = format!("SELECT {AIRPORT} FROM airport ORDER BY iata");
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    rows.iter().map(|r| airport_at(r, 0)).collect()
}

pub async fn airport(pool: &PgPool, iata: &str) -> Result<Option<Airport>> {
    let sql = format!("SELECT {AIRPORT} FROM airport WHERE iata = $1");
    let row = sqlx::query(&sql).bind(iata).fetch_optional(pool).await?;
    row.map(|r| airport_at(&r, 0)).transpose()
}

/// Origin and destination airports of a route search in one round trip.
pub async fn airport_pair(
    pool: &PgPool,
    a: &str,
    b: &str,
) -> Result<(Option<Airport>, Option<Airport>)> {
    let sql = format!("SELECT {AIRPORT} FROM airport WHERE iata = $1 OR iata = $2");
    let rows = sqlx::query(&sql).bind(a).bind(b).fetch_all(pool).await?;
    let (mut first, mut second) = (None, None);
    for row in &rows {
        let airport = airport_at(row, 0)?;
        if airport.iata == a {
            if airport.iata == b {
                second = Some(airport_at(row, 0)?);
            }
            first = Some(airport);
        } else {
            second = Some(airport);
        }
    }
    Ok((first, second))
}

pub async fn airlines(pool: &PgPool) -> Result<Vec<Airline>> {
    let sql = format!("SELECT {AIRLINE} FROM airline ORDER BY iata");
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    rows.iter().map(|r| airline_at(r, 0)).collect()
}

pub async fn airline(pool: &PgPool, iata: &str) -> Result<Option<Airline>> {
    let sql = format!("SELECT {AIRLINE} FROM airline WHERE iata = $1");
    let row = sqlx::query(&sql).bind(iata).fetch_optional(pool).await?;
    row.map(|r| airline_at(&r, 0)).transpose()
}

/// `FlightRepository.findRoute`: flights on a route inside a UTC departure window.
pub async fn flights_on_route(
    pool: &PgPool,
    origin: &Arc<Airport>,
    destination: &Arc<Airport>,
    from_utc: NaiveDateTime,
    to_utc: NaiveDateTime,
) -> Result<Vec<Flight>> {
    let rows = sqlx::query(
        "SELECT f.id, f.flight_number, f.departure, f.arrival, f.aircraft, f.duration_minutes, \
                al.iata, al.icao, al.name, al.country, al.alliance \
         FROM flight f JOIN airline al ON al.iata = f.airline_iata \
         WHERE f.origin_iata = $1 AND f.destination_iata = $2 AND f.departure >= $3 AND f.departure < $4 \
         ORDER BY f.departure, f.id",
    )
    .bind(&origin.iata)
    .bind(&destination.iata)
    .bind(from_utc)
    .bind(to_utc)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|r| flight_at(r, origin.clone(), destination.clone()))
        .collect()
}

pub async fn flight(pool: &PgPool, id: i64) -> Result<Option<Flight>> {
    let sql = format!("SELECT {FLIGHT_JOINED} FROM flight f {FLIGHT_JOINS} WHERE f.id = $1");
    let row = sqlx::query(&sql).bind(id).fetch_optional(pool).await?;
    row.map(|r| joined_flight_at(&r)).transpose()
}

/// `FareRepository.findForRoute`: every fare of every flight on the route and window, ordered by departure,
/// flight id and booking class. Flights are deduplicated: `fares[i].0` indexes into the returned flights.
pub async fn fares_on_route(
    pool: &PgPool,
    origin: &Arc<Airport>,
    destination: &Arc<Airport>,
    from_utc: NaiveDateTime,
    to_utc: NaiveDateTime,
) -> Result<(Vec<Flight>, Vec<(usize, Fare)>)> {
    let rows = sqlx::query(
        "SELECT f.id, f.flight_number, f.departure, f.arrival, f.aircraft, f.duration_minutes, \
                al.iata, al.icao, al.name, al.country, al.alliance, \
                fa.booking_class, fa.cabin, fa.base_amount, fa.currency, fa.refundable, fa.changeable, \
                fa.baggage_included_kg, fa.seats_available \
         FROM fare fa JOIN flight f ON f.id = fa.flight_id JOIN airline al ON al.iata = f.airline_iata \
         WHERE f.origin_iata = $1 AND f.destination_iata = $2 AND f.departure >= $3 AND f.departure < $4 \
         ORDER BY f.departure, f.id, fa.booking_class",
    )
    .bind(&origin.iata)
    .bind(&destination.iata)
    .bind(from_utc)
    .bind(to_utc)
    .fetch_all(pool)
    .await?;
    let mut flights: Vec<Flight> = Vec::new();
    let mut fares = Vec::with_capacity(rows.len());
    for row in &rows {
        let id: i64 = row.try_get(0)?;
        if flights.last().is_none_or(|f| f.id != id) {
            flights.push(flight_at(row, origin.clone(), destination.clone())?);
        }
        let fare = Fare {
            booking_class: row.try_get(11)?,
            cabin: cabin_at(row, 12)?,
            base_amount: row.try_get(13)?,
            currency: row.try_get(14)?,
            refundable: row.try_get(15)?,
            changeable: row.try_get(16)?,
            baggage_included_kg: row.try_get(17)?,
            seats_available: row.try_get(18)?,
        };
        fares.push((flights.len() - 1, fare));
    }
    Ok((flights, fares))
}

/// `TaxRuleRepository.findAll()` (unordered, as JPA issues it).
pub async fn tax_rules(pool: &PgPool) -> Result<Vec<TaxRule>> {
    let rows = sqlx::query(
        "SELECT kind, country, airport_iata, airline_iata, min_distance_km, max_distance_km, amount FROM tax_rule",
    )
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|r| {
            let kind: &str = r.try_get(0)?;
            Ok(TaxRule {
                kind: TaxKind::parse(kind).ok_or_else(|| decode_err("tax kind", kind))?,
                country: r.try_get(1)?,
                airport_iata: r.try_get(2)?,
                airline_iata: r.try_get(3)?,
                min_distance_km: r.try_get(4)?,
                max_distance_km: r.try_get(5)?,
                amount: r.try_get(6)?,
            })
        })
        .collect()
}

pub async fn traveler(pool: &PgPool, id: i64) -> Result<Option<Traveler>> {
    let sql = format!("SELECT {TRAVELER} FROM traveler t WHERE t.id = $1");
    let row = sqlx::query(&sql).bind(id).fetch_optional(pool).await?;
    row.map(|r| traveler_at(&r, 0)).transpose()
}

/// `findByLastNameStartingWithIgnoreCaseOrderByLastNameAscFirstNameAsc`: Spring Data escapes `\`, `%` and `_`.
pub async fn travelers_by_last_name(pool: &PgPool, prefix: &str) -> Result<Vec<Traveler>> {
    let mut pattern = String::with_capacity(prefix.len() + 1);
    for c in prefix.chars() {
        if matches!(c, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    let sql = format!(
        "SELECT {TRAVELER} FROM traveler t WHERE upper(t.last_name) LIKE upper($1) ESCAPE '\\' \
         ORDER BY t.last_name, t.first_name"
    );
    let rows = sqlx::query(&sql).bind(pattern).fetch_all(pool).await?;
    rows.iter().map(|r| traveler_at(r, 0)).collect()
}

const BOOKING: &str = "b.id, b.record_locator, b.status, b.created_at, b.total_amount, b.currency";

fn booking_at(row: &PgRow) -> Result<Booking> {
    Ok(Booking {
        id: row.try_get(0)?,
        record_locator: row.try_get(1)?,
        status: row.try_get(2)?,
        created_at: row.try_get(3)?,
        total_amount: row.try_get(4)?,
        currency: row.try_get(5)?,
        travelers: Vec::new(),
        segments: Vec::new(),
    })
}

pub async fn booking(pool: &PgPool, record_locator: &str) -> Result<Option<Booking>> {
    let sql = format!("SELECT {BOOKING} FROM booking b WHERE b.record_locator = $1");
    let Some(row) = sqlx::query(&sql)
        .bind(record_locator)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(None);
    };
    let mut bookings = vec![booking_at(&row)?];
    load_children(pool, &mut bookings).await?;
    Ok(bookings.pop())
}

/// `BookingRepository.findByTravelerLastNamePrefix`: the JPQL `like lower(concat(:prefix, '%'))` does not
/// escape the prefix, so `%`/`_` stay wildcards here.
pub async fn bookings_by_last_name(pool: &PgPool, prefix: &str) -> Result<Vec<Booking>> {
    let sql = format!(
        "SELECT DISTINCT {BOOKING} FROM booking b \
         JOIN booking_traveler bt ON bt.booking_id = b.id JOIN traveler t ON t.id = bt.traveler_id \
         WHERE lower(t.last_name) LIKE lower($1 || '%') \
         ORDER BY b.created_at DESC, b.id DESC"
    );
    let rows = sqlx::query(&sql).bind(prefix).fetch_all(pool).await?;
    let mut bookings = rows.iter().map(booking_at).collect::<Result<Vec<_>>>()?;
    load_children(pool, &mut bookings).await?;
    Ok(bookings)
}

/// Travelers and segments of several bookings, each `@OrderBy("id")`, in two queries (Hibernate's SUBSELECT fetch).
async fn load_children(pool: &PgPool, bookings: &mut [Booking]) -> Result<()> {
    if bookings.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = bookings.iter().map(|b| b.id).collect();
    let index: HashMap<i64, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();

    let sql = format!(
        "SELECT bt.booking_id, {TRAVELER} FROM booking_traveler bt JOIN traveler t ON t.id = bt.traveler_id \
         WHERE bt.booking_id = ANY($1) ORDER BY t.id"
    );
    for row in sqlx::query(&sql).bind(&ids).fetch_all(pool).await? {
        let booking_id: i64 = row.try_get(0)?;
        bookings[index[&booking_id]]
            .travelers
            .push(traveler_at(&row, 1)?);
    }

    let sql = format!(
        "SELECT {FLIGHT_JOINED}, s.booking_id, s.booking_class, s.cabin \
         FROM booking_segment s JOIN fare fa ON fa.id = s.fare_id JOIN flight f ON f.id = fa.flight_id {FLIGHT_JOINS} \
         WHERE s.booking_id = ANY($1) ORDER BY s.id"
    );
    for row in sqlx::query(&sql).bind(&ids).fetch_all(pool).await? {
        let booking_id: i64 = row.try_get(27)?;
        let segment = Segment {
            flight: joined_flight_at(&row)?,
            booking_class: row.try_get(28)?,
            cabin: cabin_at(&row, 29)?,
        };
        bookings[index[&booking_id]].segments.push(segment);
    }
    Ok(())
}
