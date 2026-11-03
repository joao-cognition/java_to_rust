//! HTTP tests for the schedule and lookup endpoints against the seeded database (Java's `DemoDataSeeder` data).
//! Skipped with a note when PostgreSQL is unreachable.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::{NaiveDateTime, TimeZone, Utc};
use http_body_util::BodyExt;
use rust_travel::{app, AppState};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tower::ServiceExt;

async fn pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
    let pool = PgPoolOptions::new().acquire_timeout(Duration::from_secs(3)).connect(&url).await;
    match pool {
        Ok(p) if sqlx::query("SELECT 1 FROM booking LIMIT 1").execute(&p).await.is_ok() => Some(p),
        _ => {
            eprintln!("note: skipping, seeded PostgreSQL not reachable at {url}");
            None
        }
    }
}

async fn call(app: &Router, uri: &str) -> (StatusCode, String, Value) {
    let res = app.clone().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, content_type, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn problem(status: u16, title: &str, detail: &str, instance: &str) -> Value {
    json!({"detail": detail, "instance": instance, "status": status, "title": title})
}

async fn assert_problem(app: &Router, uri: &str, status: u16, detail: &str) {
    let (code, content_type, body) = call(app, uri).await;
    let instance = uri.split('?').next().unwrap();
    let title = StatusCode::from_u16(status).unwrap().canonical_reason().unwrap();
    assert_eq!(code.as_u16(), status, "{uri}");
    assert_eq!(content_type, "application/problem+json", "{uri}");
    assert_eq!(body, problem(status, title, detail, instance), "{uri}");
}

#[tokio::test]
async fn flight_by_id() {
    let Some(pool) = pool().await else { return };
    let app = app(AppState { pool });
    let (code, _, body) = call(&app, "/travel/api/flights/1").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["id"], 1);
    for key in [
        "flightNumber",
        "airline",
        "origin",
        "destination",
        "departure",
        "arrival",
        "aircraft",
        "durationMinutes",
    ] {
        assert!(!body[key].is_null(), "{key}");
    }
    let (_, _, hex) = call(&app, "/travel/api/flights/0x1").await;
    assert_eq!(hex, body);
    let (_, _, spaced) = call(&app, "/travel/api/flights/%201").await;
    assert_eq!(spaced, body);
    assert_problem(&app, "/travel/api/flights/abc", 400, "Failed to convert 'id' with value: 'abc'").await;
    assert_problem(&app, "/travel/api/flights/999999999", 404, "Flight 999999999 not found").await;
}

#[tokio::test]
async fn flight_search_uses_the_local_day_at_the_origin() {
    let Some(pool) = pool().await else { return };
    let (origin, destination, departure, tz): (String, String, NaiveDateTime, String) = sqlx::query_as(
        "SELECT f.origin_iata, f.destination_iata, f.departure, o.timezone FROM flight f \
         JOIN airport o ON o.iata = f.origin_iata ORDER BY f.id LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let tz: chrono_tz::Tz = tz.parse().unwrap();
    let date = Utc.from_utc_datetime(&departure).with_timezone(&tz).date_naive().to_string();
    let app = app(AppState { pool });

    let uri = format!(
        "/travel/api/flights?origin=%20{}&destination={}&date={date}",
        origin.to_lowercase(),
        destination
    );
    let (code, _, body) = call(&app, &uri).await;
    assert_eq!(code, StatusCode::OK);
    let flights = body.as_array().unwrap();
    assert!(flights.iter().any(|f| f["id"] == 1));
    let mut previous: Option<(chrono::DateTime<chrono::FixedOffset>, i64)> = None;
    for f in flights {
        assert_eq!(f["origin"]["iata"], origin.as_str());
        assert_eq!(f["destination"]["iata"], destination.as_str());
        let dep = f["departure"].as_str().unwrap();
        assert!(dep.starts_with(&date), "{dep} is not on {date}");
        let key = (chrono::DateTime::parse_from_rfc3339(dep).unwrap(), f["id"].as_i64().unwrap());
        if let Some(p) = previous {
            assert!(p < key, "not ordered by departure, id");
        }
        previous = Some(key);
    }

    let base = "/travel/api/flights";
    assert_problem(
        &app,
        &format!("{base}?destination=NCE&date={date}"),
        400,
        "Required parameter 'origin' is not present.",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=NCE&date={date}"),
        400,
        "Required parameter 'destination' is not present.",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=NCE&destination=CDG"),
        400,
        "Required parameter 'date' is not present.",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=NCE&destination=CDG&date="),
        400,
        "Required parameter 'date' is not present.",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=NCE&destination=CDG&date=bad"),
        400,
        "Failed to convert 'date' with value: 'bad'",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=%20zz&destination=CDG&date={date}"),
        400,
        "Unknown origin airport:  zz",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=zz&destination=yy&date={date}"),
        400,
        "Unknown origin airport: zz",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=NCE&destination=yy&date={date}"),
        400,
        "Unknown destination airport: yy",
    )
    .await;
    assert_problem(
        &app,
        &format!("{base}?origin=NCE&destination=nce&date={date}"),
        400,
        "origin and destination must differ",
    )
    .await;
}

#[tokio::test]
async fn travelers() {
    let Some(pool) = pool().await else { return };
    let app = app(AppState { pool });
    let (code, _, one) = call(&app, "/travel/api/travelers/1").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(one["id"], 1);
    assert!(one.as_object().unwrap().contains_key("frequentFlyerNumber"));
    assert_problem(&app, "/travel/api/travelers/x", 400, "Failed to convert 'id' with value: 'x'").await;
    assert_problem(&app, "/travel/api/travelers/999999", 404, "Traveler 999999 not found").await;

    let prefix = one["lastName"].as_str().unwrap()[..3].to_lowercase();
    let (code, _, found) = call(&app, &format!("/travel/api/travelers?lastName=%20{prefix}%20")).await;
    assert_eq!(code, StatusCode::OK);
    let found = found.as_array().unwrap();
    assert!(found.iter().any(|t| t["id"] == 1));
    assert!(found
        .iter()
        .all(|t| t["lastName"].as_str().unwrap().to_lowercase().starts_with(&prefix)));

    let (_, _, literal) = call(&app, "/travel/api/travelers?lastName=%25").await;
    assert_eq!(literal, json!([]));
    let (_, _, literal) = call(&app, "/travel/api/travelers?lastName=_").await;
    assert_eq!(literal, json!([]));
    assert_problem(&app, "/travel/api/travelers", 400, "Required parameter 'lastName' is not present.").await;
    assert_problem(&app, "/travel/api/travelers?lastName=%20", 400, "Validation failure").await;
}

#[tokio::test]
async fn bookings() {
    let Some(pool) = pool().await else { return };
    let app = app(AppState { pool });
    let (code, _, pnr) = call(&app, "/travel/api/bookings/QX7K2M").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(pnr["recordLocator"], "QX7K2M");
    assert!(pnr["createdAt"].as_str().unwrap().ends_with("+00:00"));
    let total = serde_json::to_string(&pnr["totalAmount"]).unwrap();
    assert_eq!(total.split('.').nth(1).map(str::len), Some(2), "money scale: {total}");
    let ids: Vec<i64> = pnr["travelers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_i64().unwrap())
        .collect();
    assert!(
        !ids.is_empty() && ids.windows(2).all(|w| w[0] < w[1]),
        "travelers ordered by id: {ids:?}"
    );
    assert!(!pnr["segments"].as_array().unwrap().is_empty());

    let (_, _, normalized) = call(&app, "/travel/api/bookings/%20qx7k2m%20").await;
    assert_eq!(normalized, pnr);
    assert_problem(&app, "/travel/api/bookings/zzzzzz", 404, "Booking zzzzzz not found").await;

    let last = pnr["travelers"][0]["lastName"].as_str().unwrap().to_owned();
    let (code, _, found) = call(&app, &format!("/travel/api/bookings?lastName={}", last.to_uppercase())).await;
    assert_eq!(code, StatusCode::OK);
    let found = found.as_array().unwrap();
    assert!(found.iter().any(|b| b["recordLocator"] == "QX7K2M"));
    let keys: Vec<String> = found.iter().map(|b| b["createdAt"].as_str().unwrap().to_owned()).collect();
    assert!(keys.windows(2).all(|w| w[0] >= w[1]), "newest first");
    let locators: std::collections::HashSet<&str> = found.iter().map(|b| b["recordLocator"].as_str().unwrap()).collect();
    assert_eq!(locators.len(), found.len(), "distinct");

    // The JPQL does not escape the prefix: `%` is a wildcard, so it matches every booking.
    let (_, _, all) = call(&app, "/travel/api/bookings?lastName=%25").await;
    assert!(all.as_array().unwrap().len() >= found.len());
    assert_problem(&app, "/travel/api/bookings", 400, "Required parameter 'lastName' is not present.").await;
    assert_problem(&app, "/travel/api/bookings?lastName=", 400, "Validation failure").await;
}
