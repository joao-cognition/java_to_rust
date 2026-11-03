//! PostgreSQL layer (sqlx). Same tables as the Java service (`src/main/resources/db/postgres/schema.sql`);
//! timestamps are UTC wall-clock `TIMESTAMP` columns, money is `NUMERIC(…, 2)`.

use crate::dto::{at_airport, AirlineDto, AirportDto, FlightDto, TravelerDto};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct AirportRow {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl From<AirportRow> for AirportDto {
    fn from(a: AirportRow) -> Self {
        AirportDto {
            iata: a.iata,
            icao: a.icao,
            name: a.name,
            city: a.city,
            country: a.country,
            timezone: a.timezone,
            latitude: a.latitude,
            longitude: a.longitude,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AirlineRow {
    pub iata: String,
    pub icao: String,
    pub name: String,
    pub country: String,
    pub alliance: String,
}

impl From<AirlineRow> for AirlineDto {
    fn from(a: AirlineRow) -> Self {
        AirlineDto {
            iata: a.iata,
            icao: a.icao,
            name: a.name,
            country: a.country,
            alliance: a.alliance,
        }
    }
}

pub const AIRPORT_COLUMNS: &str = "iata, icao, name, city, country, timezone, latitude, longitude";
pub const AIRLINE_COLUMNS: &str = "iata, icao, name, country, alliance";

/// One flight joined with its airline and both airports (`join fetch f.airline join fetch f.origin
/// join fetch f.destination`). Select it with [`FLIGHT_SELECT`] followed by a `WHERE`/`ORDER BY`.
#[derive(Debug, Clone, FromRow)]
pub struct FlightRow {
    pub f_id: i64,
    pub f_flight_number: String,
    pub f_departure: chrono::NaiveDateTime,
    pub f_arrival: chrono::NaiveDateTime,
    pub f_aircraft: String,
    pub f_duration_minutes: i32,
    pub al_iata: String,
    pub al_icao: String,
    pub al_name: String,
    pub al_country: String,
    pub al_alliance: String,
    pub o_iata: String,
    pub o_icao: String,
    pub o_name: String,
    pub o_city: String,
    pub o_country: String,
    pub o_timezone: String,
    pub o_latitude: f64,
    pub o_longitude: f64,
    pub d_iata: String,
    pub d_icao: String,
    pub d_name: String,
    pub d_city: String,
    pub d_country: String,
    pub d_timezone: String,
    pub d_latitude: f64,
    pub d_longitude: f64,
}

/// Columns of [`FlightRow`]; `f`, `al`, `o`, `d` are the flight, airline, origin and destination aliases.
pub const FLIGHT_COLUMNS: &str = "f.id AS f_id, f.flight_number AS f_flight_number, f.departure AS f_departure, \
     f.arrival AS f_arrival, f.aircraft AS f_aircraft, f.duration_minutes AS f_duration_minutes, \
     al.iata AS al_iata, al.icao AS al_icao, al.name AS al_name, al.country AS al_country, al.alliance AS al_alliance, \
     o.iata AS o_iata, o.icao AS o_icao, o.name AS o_name, o.city AS o_city, o.country AS o_country, \
     o.timezone AS o_timezone, o.latitude AS o_latitude, o.longitude AS o_longitude, \
     d.iata AS d_iata, d.icao AS d_icao, d.name AS d_name, d.city AS d_city, d.country AS d_country, \
     d.timezone AS d_timezone, d.latitude AS d_latitude, d.longitude AS d_longitude";

/// `FROM` clause matching [`FLIGHT_COLUMNS`].
pub const FLIGHT_JOINS: &str = "flight f JOIN airline al ON al.iata = f.airline_iata \
     JOIN airport o ON o.iata = f.origin_iata JOIN airport d ON d.iata = f.destination_iata";

impl FlightRow {
    pub fn origin(&self) -> AirportDto {
        AirportDto {
            iata: self.o_iata.clone(),
            icao: self.o_icao.clone(),
            name: self.o_name.clone(),
            city: self.o_city.clone(),
            country: self.o_country.clone(),
            timezone: self.o_timezone.clone(),
            latitude: self.o_latitude,
            longitude: self.o_longitude,
        }
    }

    pub fn destination(&self) -> AirportDto {
        AirportDto {
            iata: self.d_iata.clone(),
            icao: self.d_icao.clone(),
            name: self.d_name.clone(),
            city: self.d_city.clone(),
            country: self.d_country.clone(),
            timezone: self.d_timezone.clone(),
            latitude: self.d_latitude,
            longitude: self.d_longitude,
        }
    }

    /// `ApiMapper.flight`.
    pub fn to_dto(&self) -> FlightDto {
        FlightDto {
            id: self.f_id,
            flight_number: self.f_flight_number.clone(),
            airline: AirlineDto {
                iata: self.al_iata.clone(),
                icao: self.al_icao.clone(),
                name: self.al_name.clone(),
                country: self.al_country.clone(),
                alliance: self.al_alliance.clone(),
            },
            origin: self.origin(),
            destination: self.destination(),
            departure: at_airport(self.f_departure, &self.o_timezone),
            arrival: at_airport(self.f_arrival, &self.d_timezone),
            aircraft: self.f_aircraft.clone(),
            duration_minutes: self.f_duration_minutes,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TravelerRow {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: chrono::NaiveDate,
    pub email: String,
    pub nationality: String,
    pub frequent_flyer_number: Option<String>,
}

pub const TRAVELER_COLUMNS: &str = "t.id, t.first_name, t.last_name, t.date_of_birth, t.email, t.nationality, t.frequent_flyer_number";

impl From<TravelerRow> for TravelerDto {
    fn from(t: TravelerRow) -> Self {
        TravelerDto {
            id: t.id,
            first_name: t.first_name,
            last_name: t.last_name,
            date_of_birth: t.date_of_birth,
            email: t.email,
            nationality: t.nationality,
            frequent_flyer_number: t.frequent_flyer_number,
        }
    }
}

/// `ReferenceDataService.normalize`: trimmed, upper-cased IATA code.
pub fn normalize_code(code: &str) -> String {
    code.trim().to_uppercase()
}

/// Spring Data `…StartingWithIgnoreCase` escaping: `\`, `%` and `_` are matched literally.
pub fn like_prefix_escaped(prefix: &str) -> String {
    let mut out = String::with_capacity(prefix.len() + 1);
    for c in prefix.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('%');
    out
}
