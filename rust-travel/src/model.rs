//! Rows of the tables the JPA entities map to, and their JSON shapes (`web/dto/ApiDtos.java`, `ApiMapper.java`).

use chrono::{NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Airport {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip)]
    #[sqlx(skip)]
    pub tz: Option<Tz>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Airline {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub country: String,
    pub alliance: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FlightRow {
    pub id: i64,
    pub flight_number: String,
    pub airline_iata: String,
    pub origin_iata: String,
    pub destination_iata: String,
    pub departure: NaiveDateTime,
    pub arrival: NaiveDateTime,
    pub aircraft: String,
    pub duration_minutes: i32,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Fare {
    pub booking_class: String,
    pub cabin: String,
    pub base_amount: Decimal,
    pub currency: String,
    pub refundable: bool,
    pub changeable: bool,
    pub baggage_included_kg: i32,
    pub seats_available: i32,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Traveler {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub email: String,
    pub nationality: String,
    pub frequent_flyer_number: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaxRule {
    pub kind: String,
    pub country: Option<String>,
    pub airport_iata: Option<String>,
    pub airline_iata: Option<String>,
    pub min_distance_km: Option<i32>,
    pub max_distance_km: Option<i32>,
    pub amount: Decimal,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightDto<'a> {
    pub id: i64,
    pub flight_number: &'a str,
    pub airline: &'a Airline,
    pub origin: &'a Airport,
    pub destination: &'a Airport,
    pub departure: String,
    pub arrival: String,
    pub aircraft: &'a str,
    pub duration_minutes: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pricing {
    pub passengers: i32,
    pub base_per_passenger: Decimal,
    pub taxes_per_passenger: Decimal,
    pub per_passenger: Decimal,
    pub base: Decimal,
    pub taxes: Decimal,
    pub total: Decimal,
    pub currency: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferDto<'a> {
    pub offer_id: String,
    pub flight: FlightDto<'a>,
    pub fare: &'a Fare,
    pub pricing: Pricing,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentDto<'a> {
    pub flight: FlightDto<'a>,
    pub booking_class: &'a str,
    pub cabin: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingDto<'a> {
    pub record_locator: &'a str,
    pub status: &'a str,
    pub created_at: String,
    pub travelers: Vec<&'a Traveler>,
    pub segments: Vec<SegmentDto<'a>>,
    pub total_amount: Decimal,
    pub currency: &'a str,
}

/// `ApiDtos.TIMESTAMP_PATTERN` (`yyyy-MM-dd'T'HH:mm:ssxxx`).
const TIMESTAMP: &str = "%Y-%m-%dT%H:%M:%S%:z";

/// `ApiMapper.atAirport`: a UTC wall-clock timestamp rendered in the airport's local time with its offset.
pub fn at_airport(utc: NaiveDateTime, airport: &Airport) -> String {
    match airport.tz {
        Some(tz) => tz.from_utc_datetime(&utc).format(TIMESTAMP).to_string(),
        None => utc_timestamp(utc),
    }
}

/// `ApiMapper.utc`.
pub fn utc_timestamp(utc: NaiveDateTime) -> String {
    chrono::Utc
        .from_utc_datetime(&utc)
        .format(TIMESTAMP)
        .to_string()
}
