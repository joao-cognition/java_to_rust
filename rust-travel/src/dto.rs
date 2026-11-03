//! JSON shapes of `ApiDtos` (Java records, so field order = declaration order). Money is `rust_decimal::Decimal`
//! serialized as a JSON number with its scale kept (`384.00`), like Jackson's `BigDecimal`. Timestamps are
//! `yyyy-MM-dd'T'HH:mm:ssxxx` in the airport's local time (flights) or UTC (booking `createdAt`).

use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AirportDto {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AirlineDto {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub country: String,
    pub alliance: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightDto {
    pub id: i64,
    pub flight_number: String,
    pub airline: AirlineDto,
    pub origin: AirportDto,
    pub destination: AirportDto,
    pub departure: String,
    pub arrival: String,
    pub aircraft: String,
    pub duration_minutes: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FareDto {
    pub booking_class: String,
    pub cabin: String,
    pub base_amount: Decimal,
    pub currency: String,
    pub refundable: bool,
    pub changeable: bool,
    pub baggage_included_kg: i32,
    pub seats_available: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingDto {
    pub passengers: i32,
    pub base_per_passenger: Decimal,
    pub taxes_per_passenger: Decimal,
    pub per_passenger: Decimal,
    pub base: Decimal,
    pub taxes: Decimal,
    pub total: Decimal,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferDto {
    pub offer_id: String,
    pub flight: FlightDto,
    pub fare: FareDto,
    pub pricing: PricingDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelerDto {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: chrono::NaiveDate,
    pub email: String,
    pub nationality: String,
    pub frequent_flyer_number: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentDto {
    pub flight: FlightDto,
    pub booking_class: String,
    pub cabin: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingDto {
    pub record_locator: String,
    pub status: String,
    pub created_at: String,
    pub travelers: Vec<TravelerDto>,
    pub segments: Vec<SegmentDto>,
    pub total_amount: Decimal,
    pub currency: String,
}

const TIMESTAMP: &str = "%Y-%m-%dT%H:%M:%S%:z";

/// `ApiMapper.atAirport`: a UTC wall-clock timestamp rendered in the airport's zone with its offset.
pub fn at_airport(utc: NaiveDateTime, timezone: &str) -> String {
    match timezone.parse::<chrono_tz::Tz>() {
        Ok(tz) => Utc.from_utc_datetime(&utc).with_timezone(&tz).format(TIMESTAMP).to_string(),
        Err(_) => utc_offset(utc),
    }
}

/// `ApiMapper.utc`: `2026-10-22T22:30:22+00:00`.
pub fn utc_offset(utc: NaiveDateTime) -> String {
    Utc.from_utc_datetime(&utc).format(TIMESTAMP).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn local_times_carry_offsets() {
        assert_eq!(at_airport(ts("2026-11-04 05:50:00"), "Europe/Paris"), "2026-11-04T06:50:00+01:00");
        assert_eq!(at_airport(ts("2026-11-04 07:45:00"), "Europe/London"), "2026-11-04T07:45:00+00:00");
        assert_eq!(at_airport(ts("2026-11-04 07:45:00"), "Asia/Kolkata"), "2026-11-04T13:15:00+05:30");
        assert_eq!(utc_offset(ts("2026-10-22 22:30:22")), "2026-10-22T22:30:22+00:00");
    }

    #[test]
    fn money_keeps_its_scale() {
        let v = serde_json::to_string(&Decimal::new(38400, 2)).unwrap();
        assert_eq!(v, "384.00");
    }
}
