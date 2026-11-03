//! `GET /offers` (OfferController / OfferService): every fare of every flight on the route and local day is priced
//! for the requested adults and returned cheapest first. This is the CPU hot path: one SQL query loads the flights
//! and fares of the day, tax rules and airports are reference data cached at first use, distance is computed once
//! per flight.

use crate::db::{normalize_code, AirportRow, FlightRow, AIRPORT_COLUMNS, FLIGHT_COLUMNS, FLIGHT_JOINS};
use crate::dto::{FareDto, FlightDto, OfferDto};
use crate::error::{ApiError, ApiResult};
use crate::geo::distance_km;
use crate::params::Params;
use crate::pricing::{Cabin, FareToPrice, TaxRule, TaxTable, TAX_RULE_COLUMNS};
use crate::AppState;
use axum::extract::{RawQuery, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, NaiveDate, NaiveDateTime, TimeZone};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// `OfferService.MAX_ADULTS`.
pub const MAX_ADULTS: i32 = 9;

pub fn router() -> Router<AppState> {
    Router::new().route("/offers", get(search))
}

async fn search(State(state): State<AppState>, RawQuery(query): RawQuery) -> ApiResult<Json<Vec<OfferDto>>> {
    let req = OfferRequest::bind(&Params::parse(query.as_deref()))?;
    Ok(Json(find_offers(&state.pool, &req).await?))
}

/// Bound and validated request parameters, in the order Spring resolves them.
#[derive(Debug, Clone, PartialEq)]
pub struct OfferRequest {
    pub origin: String,
    pub destination: String,
    pub date: NaiveDate,
    pub adults: i32,
    pub cabin: Option<Cabin>,
}

impl OfferRequest {
    /// `@RequestParam String origin, String destination, LocalDate date, @Min(1) @Max(9) int adults = 1,
    /// Cabin cabin`: binding errors stop at the first failing parameter; `@Min`/`@Max` are checked afterwards.
    pub fn bind(p: &Params) -> ApiResult<Self> {
        let origin = p.required_string("origin")?;
        let destination = p.required_string("destination")?;
        let date = p.required_date("date")?;
        let adults = p.int_or("adults", 1)?;
        let cabin = p.optional_enum("cabin", &Cabin::NAMES)?.and_then(Cabin::from_name);
        if !(1..=MAX_ADULTS).contains(&adults) {
            return Err(ApiError::validation());
        }
        Ok(Self {
            origin,
            destination,
            date,
            adults,
            cabin,
        })
    }
}

/// Airports and tax rules never change while the service runs (no endpoint writes them).
struct Reference {
    airports: HashMap<String, AirportRow>,
    taxes: TaxTable,
}

static REFERENCE: OnceLock<Arc<Reference>> = OnceLock::new();

async fn reference(pool: &PgPool) -> ApiResult<Arc<Reference>> {
    if let Some(r) = REFERENCE.get() {
        return Ok(r.clone());
    }
    let airports: Vec<AirportRow> = sqlx::query_as(&format!("SELECT {AIRPORT_COLUMNS} FROM airport"))
        .fetch_all(pool)
        .await?;
    // No ORDER BY, like `findAll()`: fuel band ties resolve on table order in both services.
    let rules: Vec<TaxRule> = sqlx::query_as(&format!("SELECT {TAX_RULE_COLUMNS} FROM tax_rule"))
        .fetch_all(pool)
        .await?;
    let loaded = Reference {
        airports: airports.into_iter().map(|a| (a.iata.clone(), a)).collect(),
        taxes: TaxTable::new(&rules),
    };
    Ok(REFERENCE.get_or_init(|| Arc::new(loaded)).clone())
}

/// `FlightService.RouteDay.lookup`: normalized id lookup, error message quotes the raw parameter.
fn lookup<'a>(reference: &'a Reference, code: &str, role: &str) -> ApiResult<&'a AirportRow> {
    reference
        .airports
        .get(&normalize_code(code))
        .ok_or_else(|| ApiError::bad_request(format!("Unknown {role} airport: {code}")))
}

/// `LocalDate.atStartOfDay(zone)` as a UTC wall-clock time: midnight, or the end of the gap when midnight does not
/// exist locally; the earlier offset when it is ambiguous.
pub fn start_of_day_utc(date: NaiveDate, tz: chrono_tz::Tz) -> NaiveDateTime {
    let midnight = date.and_hms_opt(0, 0, 0).expect("midnight");
    match tz.from_local_datetime(&midnight).earliest() {
        Some(t) => t.naive_utc(),
        None => {
            // In a gap: the instant of the transition is midnight read with the offset in force before it.
            let before = tz.from_local_datetime(&(midnight - Duration::hours(6))).earliest();
            match before {
                Some(b) => {
                    let offset = b.naive_local() - b.naive_utc();
                    let mut probe = midnight - offset;
                    while tz.from_utc_datetime(&probe).naive_local() < midnight {
                        probe += Duration::minutes(1);
                    }
                    probe
                }
                None => midnight,
            }
        }
    }
}

/// One fare joined with its flight, airline and airports.
#[derive(Debug, FromRow)]
struct OfferRow {
    #[sqlx(flatten)]
    flight: FlightRow,
    fa_booking_class: String,
    fa_cabin: String,
    fa_base_amount: Decimal,
    fa_currency: String,
    fa_refundable: bool,
    fa_changeable: bool,
    fa_baggage_included_kg: i32,
    fa_seats_available: i32,
}

fn offers_sql() -> &'static str {
    static SQL: OnceLock<String> = OnceLock::new();
    SQL.get_or_init(|| {
        format!(
            "SELECT {FLIGHT_COLUMNS}, fa.booking_class AS fa_booking_class, fa.cabin AS fa_cabin, \
             fa.base_amount AS fa_base_amount, fa.currency AS fa_currency, fa.refundable AS fa_refundable, \
             fa.changeable AS fa_changeable, fa.baggage_included_kg AS fa_baggage_included_kg, \
             fa.seats_available AS fa_seats_available \
             FROM {FLIGHT_JOINS} JOIN fare fa ON fa.flight_id = f.id \
             WHERE f.origin_iata = $1::bpchar AND f.destination_iata = $2::bpchar \
             AND f.departure >= $3 AND f.departure < $4 \
             AND fa.seats_available >= $5 AND ($6::text IS NULL OR fa.cabin = $6) \
             ORDER BY f.departure, f.id, fa.booking_class"
        )
    })
}

struct Ranked {
    offer: OfferDto,
    departure: NaiveDateTime,
}

/// `OfferService.search`.
pub async fn find_offers(pool: &PgPool, req: &OfferRequest) -> ApiResult<Vec<OfferDto>> {
    let reference = reference(pool).await?;
    let origin = lookup(&reference, &req.origin, "origin")?;
    let destination = lookup(&reference, &req.destination, "destination")?;
    if origin.iata == destination.iata {
        return Err(ApiError::bad_request("origin and destination must differ"));
    }
    let tz: chrono_tz::Tz = origin.timezone.parse().map_err(|_| ApiError::internal())?;
    let from_utc = start_of_day_utc(req.date, tz);
    let to_utc = start_of_day_utc(req.date.succ_opt().ok_or_else(ApiError::internal)?, tz);

    let rows: Vec<OfferRow> = sqlx::query_as(offers_sql())
        .bind(&origin.iata)
        .bind(&destination.iata)
        .bind(from_utc)
        .bind(to_utc)
        .bind(req.adults)
        .bind(req.cabin.map(Cabin::name))
        .fetch_all(pool)
        .await?;

    let mut ranked: Vec<Ranked> = Vec::with_capacity(rows.len());
    let mut current: Option<(i64, FlightDto, f64)> = None;
    for row in rows {
        let f = &row.flight;
        if current.as_ref().is_none_or(|(id, _, _)| *id != f.f_id) {
            let km = distance_km(f.o_latitude, f.o_longitude, f.d_latitude, f.d_longitude);
            current = Some((f.f_id, f.to_dto(), km));
        }
        let (_, flight_dto, km) = current.as_ref().expect("current flight");
        let cabin = Cabin::from_name(&row.fa_cabin).ok_or_else(ApiError::internal)?;
        let pricing = reference.taxes.price(
            &FareToPrice {
                base_amount: row.fa_base_amount,
                cabin,
                currency: &row.fa_currency,
                airline: &f.al_iata,
                origin_iata: &f.o_iata,
                origin_country: &f.o_country,
                destination_country: &f.d_country,
            },
            req.adults,
            *km,
        );
        let departure = f.f_departure;
        let offer_id = format!("{}-{}", f.f_id, row.fa_booking_class);
        ranked.push(Ranked {
            offer: OfferDto {
                offer_id,
                flight: flight_dto.clone(),
                fare: FareDto {
                    booking_class: row.fa_booking_class,
                    cabin: cabin.name().to_owned(),
                    base_amount: row.fa_base_amount,
                    currency: row.fa_currency,
                    refundable: row.fa_refundable,
                    changeable: row.fa_changeable,
                    baggage_included_kg: row.fa_baggage_included_kg,
                    seats_available: row.fa_seats_available,
                },
                pricing,
            },
            departure,
        });
    }
    sort_cheapest_first(&mut ranked);
    Ok(ranked.into_iter().map(|r| r.offer).collect())
}

/// `CHEAPEST_FIRST`: total, then departure UTC, flight id, booking class.
fn sort_cheapest_first(offers: &mut [Ranked]) {
    offers.sort_by(|a, b| {
        a.offer
            .pricing
            .total
            .cmp(&b.offer.pricing.total)
            .then_with(|| a.departure.cmp(&b.departure))
            .then_with(|| a.offer.flight.id.cmp(&b.offer.flight.id))
            .then_with(|| a.offer.fare.booking_class.cmp(&b.offer.fare.booking_class))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bind(q: &str) -> ApiResult<OfferRequest> {
        OfferRequest::bind(&Params::parse(Some(q)))
    }

    const ROUTE: &str = "origin=NCE&destination=LHR&date=2026-11-05";

    #[test]
    fn defaults_and_enum_binding() {
        let r = bind(ROUTE).unwrap();
        assert_eq!((r.adults, r.cabin), (1, None));
        assert_eq!(bind(&format!("{ROUTE}&adults=")).unwrap().adults, 1);
        assert_eq!(bind(&format!("{ROUTE}&adults=0x9")).unwrap().adults, 9);
        assert_eq!(bind(&format!("{ROUTE}&adults=%209")).unwrap().adults, 9);
        assert_eq!(
            bind(&format!("{ROUTE}&cabin=PREMIUM_ECONOMY")).unwrap().cabin,
            Some(Cabin::PremiumEconomy)
        );
        assert_eq!(bind(&format!("{ROUTE}&cabin=")).unwrap().cabin, None);
    }

    #[test]
    fn binding_errors() {
        assert_eq!(bind("destination=LHR&date=2026-11-05"), Err(ApiError::missing("origin")));
        assert_eq!(bind("origin=NCE&date=x"), Err(ApiError::missing("destination")));
        assert_eq!(bind("origin=NCE&destination=LHR"), Err(ApiError::missing("date")));
        assert_eq!(
            bind("origin=NCE&destination=LHR&date=2026-13-01"),
            Err(ApiError::convert("date", "2026-13-01"))
        );
        assert_eq!(bind(&format!("{ROUTE}&adults=x")), Err(ApiError::convert("adults", "x")));
        assert_eq!(bind(&format!("{ROUTE}&cabin=COACH")), Err(ApiError::convert("cabin", "COACH")));
        assert_eq!(bind(&format!("{ROUTE}&adults=0")), Err(ApiError::validation()));
        assert_eq!(bind(&format!("{ROUTE}&adults=10")), Err(ApiError::validation()));
        assert_eq!(bind(&format!("{ROUTE}&adults=-1")), Err(ApiError::validation()));
        // conversion of a later parameter fails before @Min/@Max is checked
        assert_eq!(
            bind(&format!("{ROUTE}&adults=0&cabin=COACH")),
            Err(ApiError::convert("cabin", "COACH"))
        );
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn local_day_window_in_utc() {
        let paris: chrono_tz::Tz = "Europe/Paris".parse().unwrap();
        assert_eq!(start_of_day_utc(d(2026, 11, 5), paris).to_string(), "2026-11-04 23:00:00");
        assert_eq!(start_of_day_utc(d(2026, 7, 5), paris).to_string(), "2026-07-04 22:00:00");
        let kolkata: chrono_tz::Tz = "Asia/Kolkata".parse().unwrap();
        assert_eq!(start_of_day_utc(d(2026, 11, 5), kolkata).to_string(), "2026-11-04 18:30:00");
        // America/Santiago skips 00:00 → 01:00 on 2026-09-06: start of day is 01:00 -03 = 04:00 UTC
        let santiago: chrono_tz::Tz = "America/Santiago".parse().unwrap();
        assert_eq!(start_of_day_utc(d(2026, 9, 6), santiago).to_string(), "2026-09-06 04:00:00");
        // and repeats 23:00-24:00 on 2026-04-04, so 2026-04-05 00:00 is unambiguous (-04)
        assert_eq!(start_of_day_utc(d(2026, 4, 5), santiago).to_string(), "2026-04-05 04:00:00");
    }
}
