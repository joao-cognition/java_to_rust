//! flights: `FlightController` / `FlightService` (route search by local departure date, flight by id).

use crate::db::{normalize_code, FlightRow, FLIGHT_COLUMNS, FLIGHT_JOINS};
use crate::dto::FlightDto;
use crate::error::{ApiError, ApiResult};
use crate::params::{path_long, Params};
use crate::AppState;
use axum::extract::{Path, RawQuery, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, LocalResult, NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use sqlx::PgPool;

pub fn router() -> Router<AppState> {
    Router::new().route("/flights", get(search)).route("/flights/{id}", get(get_flight))
}

async fn search(State(state): State<AppState>, RawQuery(query): RawQuery) -> ApiResult<Json<Vec<FlightDto>>> {
    let params = Params::parse(query.as_deref());
    let origin = params.required_string("origin")?;
    let destination = params.required_string("destination")?;
    let date = params.required_date("date")?;
    let flights = find_route(&state.pool, &origin, &destination, date).await?;
    Ok(Json(flights.iter().map(FlightRow::to_dto).collect()))
}

async fn get_flight(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<Json<FlightDto>> {
    let id = path_long("id", &id)?;
    let sql = format!("SELECT {FLIGHT_COLUMNS} FROM {FLIGHT_JOINS} WHERE f.id = $1");
    let row = sqlx::query_as::<_, FlightRow>(&sql).bind(id).fetch_optional(&state.pool).await?;
    row.map(|f| Json(f.to_dto()))
        .ok_or_else(|| ApiError::not_found(format!("Flight {id} not found")))
}

/// `FlightService.search`: flights leaving `origin` for `destination` on `date`, local date at the origin.
pub async fn find_route(pool: &PgPool, origin: &str, destination: &str, date: NaiveDate) -> ApiResult<Vec<FlightRow>> {
    let (from, from_tz) = airport(pool, origin, "origin").await?;
    let (to, _) = airport(pool, destination, "destination").await?;
    if from == to {
        return Err(ApiError::bad_request("origin and destination must differ"));
    }
    let tz: Tz = from_tz.parse().map_err(|_| ApiError::internal())?;
    let (from_utc, to_utc) = utc_window(date, tz);
    let sql = format!(
        "SELECT {FLIGHT_COLUMNS} FROM {FLIGHT_JOINS} \
         WHERE f.origin_iata = $1 AND f.destination_iata = $2 AND f.departure >= $3 AND f.departure < $4 \
         ORDER BY f.departure, f.id"
    );
    Ok(sqlx::query_as::<_, FlightRow>(&sql)
        .bind(&from)
        .bind(&to)
        .bind(from_utc)
        .bind(to_utc)
        .fetch_all(pool)
        .await?)
}

/// `RouteDay.lookup`: the airport (IATA, timezone) or 400 `Unknown <role> airport: <raw code>`.
async fn airport(pool: &PgPool, raw: &str, role: &str) -> ApiResult<(String, String)> {
    sqlx::query_as::<_, (String, String)>("SELECT iata, timezone FROM airport WHERE iata = $1")
        .bind(normalize_code(raw))
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::bad_request(format!("Unknown {role} airport: {raw}")))
}

/// UTC wall-clock bounds of one local day: `[date.atStartOfDay(zone), date.plusDays(1).atStartOfDay(zone))`.
pub fn utc_window(date: NaiveDate, tz: Tz) -> (NaiveDateTime, NaiveDateTime) {
    (start_of_day_utc(date, tz), start_of_day_utc(date + Duration::days(1), tz))
}

/// `LocalDate.atStartOfDay(ZoneId)`: midnight, the earlier offset in an overlap, the end of the gap when
/// midnight does not exist (DST switch at 00:00).
fn start_of_day_utc(date: NaiveDate, tz: Tz) -> NaiveDateTime {
    let mut local = date.and_hms_opt(0, 0, 0).expect("midnight");
    loop {
        match tz.from_local_datetime(&local) {
            LocalResult::Single(t) => return t.naive_utc(),
            LocalResult::Ambiguous(earliest, _) => return earliest.naive_utc(),
            LocalResult::None => local += Duration::minutes(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn ts(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn window_is_the_local_day_in_utc() {
        assert_eq!(
            utc_window(d("2026-11-04"), chrono_tz::Europe::Paris),
            (ts("2026-11-03 23:00"), ts("2026-11-04 23:00"))
        );
        assert_eq!(
            utc_window(d("2026-11-04"), chrono_tz::Asia::Kolkata),
            (ts("2026-11-03 18:30"), ts("2026-11-04 18:30"))
        );
        assert_eq!(
            utc_window(d("2026-11-04"), chrono_tz::America::New_York),
            (ts("2026-11-04 05:00"), ts("2026-11-05 05:00"))
        );
        assert_eq!(
            utc_window(d("2026-11-04"), chrono_tz::UTC),
            (ts("2026-11-04 00:00"), ts("2026-11-05 00:00"))
        );
    }

    #[test]
    fn dst_days_are_23_or_25_hours() {
        assert_eq!(
            utc_window(d("2026-03-29"), chrono_tz::Europe::Paris),
            (ts("2026-03-28 23:00"), ts("2026-03-29 22:00"))
        );
        assert_eq!(
            utc_window(d("2026-10-25"), chrono_tz::Europe::Paris),
            (ts("2026-10-24 22:00"), ts("2026-10-25 23:00"))
        );
    }

    #[test]
    fn midnight_gap_starts_the_day_after_the_gap() {
        // Santiago springs forward at 24:00 (00:00 → 01:00): Java's atStartOfDay gives 01:00-03:00.
        let (from, _) = utc_window(d("2026-09-06"), chrono_tz::America::Santiago);
        assert_eq!(from, ts("2026-09-06 04:00"));
    }
}
