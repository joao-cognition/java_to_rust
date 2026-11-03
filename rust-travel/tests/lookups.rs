//! HTTP tests for the schedule and lookup endpoints (flights, travelers, bookings) against the seeded database.

mod common;

use chrono::{NaiveDateTime, TimeZone, Utc};
use common::Api;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn flight_by_id() {
    let Some(api) = Api::connect().await else { return };
    let one = api.get("flights/1").await;
    assert_eq!(one.status, 200);
    assert_eq!(one.json["id"], 1);
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
        assert!(!one.json[key].is_null(), "{key}");
    }
    assert_eq!(api.get("flights/0x1").await.json, one.json);
    assert_eq!(api.get("flights/%201").await.json, one.json);
    api.problem("flights/abc", 400, "Failed to convert 'id' with value: 'abc'").await;
    api.problem("flights/999999999", 404, "Flight 999999999 not found").await;
}

#[tokio::test]
async fn flight_search_uses_the_local_day_at_the_origin() {
    let Some(api) = Api::connect().await else { return };
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
    let pool = PgPoolOptions::new().max_connections(1).connect(&url).await.unwrap();
    let (origin, destination, departure, tz): (String, String, NaiveDateTime, String) = sqlx::query_as(
        "SELECT f.origin_iata, f.destination_iata, f.departure, o.timezone FROM flight f \
         JOIN airport o ON o.iata = f.origin_iata ORDER BY f.id LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let tz: chrono_tz::Tz = tz.parse().unwrap();
    let date = Utc.from_utc_datetime(&departure).with_timezone(&tz).date_naive().to_string();

    let r = api
        .get(&format!(
            "flights?origin=%20{}&destination={destination}&date={date}",
            origin.to_lowercase()
        ))
        .await;
    assert_eq!(r.status, 200);
    let flights = r.json.as_array().unwrap();
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
    let repeated = api
        .get(&format!("flights?origin={origin}&destination={destination}&date={date}&date=bad"))
        .await;
    assert_eq!(repeated.json, r.json, "repeated converted parameter: first value wins");

    let missing = |p: &str| format!("Required parameter '{p}' is not present.");
    api.problem(&format!("flights?destination=NCE&date={date}"), 400, &missing("origin"))
        .await;
    api.problem(&format!("flights?origin=NCE&date={date}"), 400, &missing("destination"))
        .await;
    api.problem("flights?origin=NCE&destination=CDG", 400, &missing("date")).await;
    api.problem("flights?origin=NCE&destination=CDG&date=", 400, &missing("date")).await;
    api.problem(
        "flights?origin=NCE&destination=CDG&date=bad",
        400,
        "Failed to convert 'date' with value: 'bad'",
    )
    .await;
    api.problem(
        &format!("flights?origin=%20zz&destination=CDG&date={date}"),
        400,
        "Unknown origin airport:  zz",
    )
    .await;
    api.problem(
        &format!("flights?origin=zz&destination=yy&date={date}"),
        400,
        "Unknown origin airport: zz",
    )
    .await;
    api.problem(
        &format!("flights?origin=NCE&destination=yy&date={date}"),
        400,
        "Unknown destination airport: yy",
    )
    .await;
    api.problem(
        &format!("flights?origin=NCE&destination=nce&date={date}"),
        400,
        "origin and destination must differ",
    )
    .await;
}

#[tokio::test]
async fn travelers() {
    let Some(api) = Api::connect().await else { return };
    let one = api.get("travelers/1").await;
    assert_eq!(one.status, 200);
    assert_eq!(one.json["id"], 1);
    assert!(one.json.as_object().unwrap().contains_key("frequentFlyerNumber"));
    api.problem("travelers/x", 400, "Failed to convert 'id' with value: 'x'").await;
    api.problem("travelers/999999", 404, "Traveler 999999 not found").await;

    let prefix = one.json["lastName"].as_str().unwrap()[..3].to_lowercase();
    let found = api.get(&format!("travelers?lastName=%20{prefix}%20")).await;
    assert_eq!(found.status, 200);
    let found = found.json.as_array().unwrap().clone();
    assert!(found.iter().any(|t| t["id"] == 1));
    assert!(found
        .iter()
        .all(|t| t["lastName"].as_str().unwrap().to_lowercase().starts_with(&prefix)));

    // Spring Data escapes `StartingWith` prefixes: `%` and `_` are literal.
    assert_eq!(api.get("travelers?lastName=%25").await.json, json!([]));
    assert_eq!(api.get("travelers?lastName=_").await.json, json!([]));
    api.problem("travelers", 400, "Required parameter 'lastName' is not present.").await;
    api.problem("travelers?lastName=%20", 400, "Validation failure").await;
}

#[tokio::test]
async fn bookings() {
    let Some(api) = Api::connect().await else { return };
    let pnr = api.get("bookings/QX7K2M").await;
    assert_eq!(pnr.status, 200);
    let pnr = pnr.json;
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

    assert_eq!(api.get("bookings/%20qx7k2m%20").await.json, pnr);
    api.problem("bookings/zzzzzz", 404, "Booking zzzzzz not found").await;

    let last = pnr["travelers"][0]["lastName"].as_str().unwrap().to_uppercase();
    let found = api.get(&format!("bookings?lastName={last}")).await;
    assert_eq!(found.status, 200);
    let found = found.json.as_array().unwrap().clone();
    assert!(found.iter().any(|b| b["recordLocator"] == "QX7K2M"));
    let created: Vec<&str> = found.iter().map(|b| b["createdAt"].as_str().unwrap()).collect();
    assert!(created.windows(2).all(|w| w[0] >= w[1]), "newest first");
    let locators: std::collections::HashSet<&str> = found.iter().map(|b| b["recordLocator"].as_str().unwrap()).collect();
    assert_eq!(locators.len(), found.len(), "distinct");

    // The JPQL does not escape the prefix: `%` is a wildcard here, so it matches every booking.
    let all = api.get("bookings?lastName=%25").await;
    assert!(all.json.as_array().unwrap().len() >= found.len());
    api.problem("bookings", 400, "Required parameter 'lastName' is not present.").await;
    api.problem("bookings?lastName=", 400, "Validation failure").await;
}
