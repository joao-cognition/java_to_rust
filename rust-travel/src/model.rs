//! Rows of the shared PostgreSQL schema and their JSON shapes (Java `ApiDtos` field names).

use chrono::{NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use rust_decimal::Decimal;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::fmt;
use std::sync::Arc;

/// `ApiDtos.TIMESTAMP_PATTERN` = `yyyy-MM-dd'T'HH:mm:ssxxx` (no fraction, numeric offset even for UTC).
const TIMESTAMP: &str = "%Y-%m-%dT%H:%M:%S%:z";

#[derive(Serialize)]
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
    pub tz: Tz,
}

#[derive(Serialize)]
pub struct Airline {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub country: String,
    pub alliance: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Cabin {
    Economy,
    PremiumEconomy,
    Business,
    First,
}

impl Cabin {
    /// `Enum.valueOf`: exact, case-sensitive constant name.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ECONOMY" => Some(Self::Economy),
            "PREMIUM_ECONOMY" => Some(Self::PremiumEconomy),
            "BUSINESS" => Some(Self::Business),
            "FIRST" => Some(Self::First),
            _ => None,
        }
    }
}

pub struct Flight {
    pub id: i64,
    pub flight_number: String,
    pub airline: Airline,
    pub origin: Arc<Airport>,
    pub destination: Arc<Airport>,
    pub departure_utc: NaiveDateTime,
    pub arrival_utc: NaiveDateTime,
    pub aircraft: String,
    pub duration_minutes: i32,
}

/// UTC wall-clock time rendered in the local time of an airport (`ApiMapper.atAirport`).
struct AtAirport<'a>(NaiveDateTime, &'a Airport);

impl fmt::Display for AtAirport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1
            .tz
            .from_utc_datetime(&self.0)
            .format(TIMESTAMP)
            .fmt(f)
    }
}

struct Display<T: fmt::Display>(T);

impl<T: fmt::Display> Serialize for Display<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.0)
    }
}

impl Serialize for Flight {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("FlightDto", 9)?;
        st.serialize_field("id", &self.id)?;
        st.serialize_field("flightNumber", &self.flight_number)?;
        st.serialize_field("airline", &self.airline)?;
        st.serialize_field("origin", &*self.origin)?;
        st.serialize_field("destination", &*self.destination)?;
        st.serialize_field(
            "departure",
            &Display(AtAirport(self.departure_utc, &self.origin)),
        )?;
        st.serialize_field(
            "arrival",
            &Display(AtAirport(self.arrival_utc, &self.destination)),
        )?;
        st.serialize_field("aircraft", &self.aircraft)?;
        st.serialize_field("durationMinutes", &self.duration_minutes)?;
        st.end()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fare {
    pub booking_class: String,
    pub cabin: Cabin,
    pub base_amount: Decimal,
    pub currency: String,
    pub refundable: bool,
    pub changeable: bool,
    pub baggage_included_kg: i32,
    pub seats_available: i32,
}

/// Priced amounts for one fare and a number of passengers (`Pricing.java`); all amounts have two decimals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pricing<'a> {
    pub passengers: i32,
    pub base_per_passenger: Decimal,
    pub taxes_per_passenger: Decimal,
    pub per_passenger: Decimal,
    pub base: Decimal,
    pub taxes: Decimal,
    pub total: Decimal,
    pub currency: &'a str,
}

/// A priced fare; `offerId` is `<flightId>-<bookingClass>` (`Offer.offerId`).
pub struct Offer<'a> {
    pub flight: &'a Flight,
    pub fare: &'a Fare,
    pub pricing: Pricing<'a>,
}

impl Serialize for Offer<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("OfferDto", 4)?;
        st.serialize_field(
            "offerId",
            &Display(format_args!(
                "{}-{}",
                self.flight.id, self.fare.booking_class
            )),
        )?;
        st.serialize_field("flight", self.flight)?;
        st.serialize_field("fare", self.fare)?;
        st.serialize_field("pricing", &self.pricing)?;
        st.end()
    }
}

pub struct Traveler {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub email: String,
    pub nationality: String,
    pub frequent_flyer_number: Option<String>,
}

impl Serialize for Traveler {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("TravelerDto", 7)?;
        st.serialize_field("id", &self.id)?;
        st.serialize_field("firstName", &self.first_name)?;
        st.serialize_field("lastName", &self.last_name)?;
        st.serialize_field(
            "dateOfBirth",
            &Display(self.date_of_birth.format("%Y-%m-%d")),
        )?;
        st.serialize_field("email", &self.email)?;
        st.serialize_field("nationality", &self.nationality)?;
        st.serialize_field("frequentFlyerNumber", &self.frequent_flyer_number)?;
        st.end()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub flight: Flight,
    pub booking_class: String,
    pub cabin: Cabin,
}

pub struct Booking {
    pub id: i64,
    pub record_locator: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub total_amount: Decimal,
    pub currency: String,
    pub travelers: Vec<Traveler>,
    pub segments: Vec<Segment>,
}

impl Serialize for Booking {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("BookingDto", 7)?;
        st.serialize_field("recordLocator", &self.record_locator)?;
        st.serialize_field("status", &self.status)?;
        st.serialize_field(
            "createdAt",
            &Display(self.created_at.and_utc().format(TIMESTAMP)),
        )?;
        st.serialize_field("travelers", &self.travelers)?;
        st.serialize_field("segments", &self.segments)?;
        st.serialize_field("totalAmount", &self.total_amount)?;
        st.serialize_field("currency", &self.currency)?;
        st.end()
    }
}
