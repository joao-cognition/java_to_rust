//! HTTP tests of `GET /travel/api/offers` against the seeded PostgreSQL database (start the Java service once with
//! `--spring.profiles.active=postgres` to seed it). Skipped with a note when the database is unreachable or empty.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rust_travel::{app, AppState};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;

async fn pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
    let pool = match PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skipping offers HTTP tests: database unreachable ({e})");
            return None;
        }
    };
    match sqlx::query_scalar::<_, i64>("SELECT count(*) FROM fare").fetch_one(&pool).await {
        Ok(n) if n > 0 => Some(pool),
        _ => {
            eprintln!("skipping offers HTTP tests: database not seeded");
            None
        }
    }
}

/// A local date (at the origin) with flights on the route, a few days into the seeded schedule.
async fn route_date(pool: &PgPool, origin: &str, destination: &str) -> String {
    sqlx::query_scalar::<_, String>(
        "SELECT to_char((f.departure AT TIME ZONE 'UTC' AT TIME ZONE o.timezone)::date, 'YYYY-MM-DD') \
         FROM flight f JOIN airport o ON o.iata = f.origin_iata \
         WHERE f.origin_iata = $1 AND f.destination_iata = $2 ORDER BY f.departure OFFSET 40 LIMIT 1",
    )
    .bind(origin)
    .bind(destination)
    .fetch_one(pool)
    .await
    .expect("seeded route")
}

async fn get(pool: &PgPool, uri: &str) -> (StatusCode, String, Value) {
    let res = app(AppState { pool: pool.clone() })
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let ct = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, ct, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn num(v: &Value) -> String {
    v.to_string()
}

fn cents(v: &Value) -> i64 {
    let s = num(v);
    let (int, frac) = s.split_once('.').expect("two decimals");
    assert_eq!(frac.len(), 2, "money must keep scale 2: {s}");
    int.parse::<i64>().unwrap() * 100 + frac.parse::<i64>().unwrap() * if s.starts_with('-') { -1 } else { 1 }
}

fn check_offers(offers: &[Value], adults: i64) {
    let mut previous: Option<(i64, String, i64, String)> = None;
    for o in offers {
        let flight = &o["flight"];
        let fare = &o["fare"];
        let p = &o["pricing"];
        let class = fare["bookingClass"].as_str().unwrap().to_owned();
        assert_eq!(o["offerId"].as_str().unwrap(), format!("{}-{}", flight["id"], class));
        assert!(fare["seatsAvailable"].as_i64().unwrap() >= adults);
        assert_eq!(p["passengers"].as_i64().unwrap(), adults);
        let (base, taxes, per) = (
            cents(&p["basePerPassenger"]),
            cents(&p["taxesPerPassenger"]),
            cents(&p["perPassenger"]),
        );
        assert_eq!(base, cents(&fare["baseAmount"]));
        assert_eq!(per, base + taxes);
        assert_eq!(cents(&p["base"]), base * adults);
        assert_eq!(cents(&p["taxes"]), taxes * adults);
        assert_eq!(cents(&p["total"]), per * adults);
        assert_eq!(p["currency"], fare["currency"]);
        let key = (
            cents(&p["total"]),
            flight["departure"].as_str().unwrap().to_owned(),
            flight["id"].as_i64().unwrap(),
            class,
        );
        if let Some(prev) = &previous {
            assert!(prev.0 <= key.0, "sorted by total");
            if prev.0 == key.0 {
                assert!((prev.2, &prev.3) != (key.2, &key.3));
            }
        }
        previous = Some(key);
    }
}

#[tokio::test]
async fn offers_are_priced_sorted_and_carry_two_decimal_money() {
    let Some(pool) = pool().await else { return };
    let date = route_date(&pool, "NCE", "LHR").await;
    let (status, ct, body) = get(&pool, &format!("/travel/api/offers?origin=NCE&destination=LHR&date={date}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ct, "application/json");
    let offers = body.as_array().unwrap();
    assert!(offers.len() >= 4);
    check_offers(offers, 1);
    // NCE→LHR: FR-DEP 7.40 + GB-ARR 0.80 + NCE-PSC 12.80 + BA/AF fuel band 0-1500 km (24.00 / 22.00) × cabin
    for o in offers {
        let economy = o["fare"]["cabin"] == "ECONOMY";
        let expected = match (o["flight"]["airline"]["iata"].as_str().unwrap(), economy) {
            ("BA", true) => "45.00",
            ("AF", true) => "43.00",
            _ => continue,
        };
        assert_eq!(num(&o["pricing"]["taxesPerPassenger"]), expected);
    }
    // lower-case / padded codes resolve the same airports
    let (_, _, again) = get(&pool, &format!("/travel/api/offers?origin=%20nce&destination=lhr&date={date}")).await;
    assert_eq!(again, body);
}

#[tokio::test]
async fn domestic_route_adds_vat() {
    let Some(pool) = pool().await else { return };
    let date = route_date(&pool, "NCE", "CDG").await;
    let (status, _, body) = get(
        &pool,
        &format!("/travel/api/offers?origin=NCE&destination=CDG&date={date}&cabin=ECONOMY"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let offers = body.as_array().unwrap();
    assert!(!offers.is_empty());
    check_offers(offers, 1);
    for o in offers {
        // 7.40 + 1.20 + 12.80 + AF 22.00 + 10% VAT of the base fare, rounded half-even
        let base = cents(&o["fare"]["baseAmount"]);
        let vat = (base as f64 / 10.0).round_ties_even() as i64;
        let expected = 4340 + vat;
        assert_eq!(cents(&o["pricing"]["taxesPerPassenger"]), expected, "{}", o["offerId"]);
    }
}

#[tokio::test]
async fn each_cabin_filters_fares() {
    let Some(pool) = pool().await else { return };
    let date = route_date(&pool, "CDG", "JFK").await;
    let (_, _, all) = get(&pool, &format!("/travel/api/offers?origin=CDG&destination=JFK&date={date}")).await;
    let all = all.as_array().unwrap().clone();
    let mut total = 0;
    for cabin in ["ECONOMY", "PREMIUM_ECONOMY", "BUSINESS", "FIRST"] {
        let (status, _, body) = get(
            &pool,
            &format!("/travel/api/offers?origin=CDG&destination=JFK&date={date}&cabin={cabin}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let offers = body.as_array().unwrap();
        assert!(!offers.is_empty(), "{cabin}");
        assert!(offers.iter().all(|o| o["fare"]["cabin"] == cabin));
        check_offers(offers, 1);
        let expected: Vec<&Value> = all.iter().filter(|o| o["fare"]["cabin"] == cabin).collect();
        assert_eq!(offers.iter().collect::<Vec<_>>(), expected);
        total += offers.len();
    }
    assert_eq!(total, all.len());
}

#[tokio::test]
async fn adults_multiply_totals_and_drop_sold_out_fares() {
    let Some(pool) = pool().await else { return };
    let date = route_date(&pool, "LHR", "SIN").await;
    let (_, _, one) = get(&pool, &format!("/travel/api/offers?origin=LHR&destination=SIN&date={date}")).await;
    let one = one.as_array().unwrap().clone();
    for adults in 1..=9_i64 {
        let (status, _, body) = get(
            &pool,
            &format!("/travel/api/offers?origin=LHR&destination=SIN&date={date}&adults={adults}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let offers = body.as_array().unwrap();
        check_offers(offers, adults);
        let mut expected: Vec<String> = one
            .iter()
            .filter(|o| o["fare"]["seatsAvailable"].as_i64().unwrap() >= adults)
            .map(|o| o["offerId"].as_str().unwrap().to_owned())
            .collect();
        let mut got: Vec<String> = offers.iter().map(|o| o["offerId"].as_str().unwrap().to_owned()).collect();
        expected.sort();
        got.sort();
        assert_eq!(got, expected, "adults={adults}");
    }
    let sold_out = one.iter().filter(|o| o["fare"]["seatsAvailable"].as_i64().unwrap() < 9).count();
    assert!(sold_out > 0, "the seeded day has fares with fewer than 9 seats");
}

#[tokio::test]
async fn invalid_parameters_are_problems() {
    let Some(pool) = pool().await else { return };
    let date = route_date(&pool, "NCE", "LHR").await;
    let route = format!("origin=NCE&destination=LHR&date={date}");
    let cases: Vec<(String, &str)> = vec![
        (format!("{route}&adults=0"), "Validation failure"),
        (format!("{route}&adults=10"), "Validation failure"),
        (format!("{route}&adults=x"), "Failed to convert 'adults' with value: 'x'"),
        (format!("{route}&cabin=COACH"), "Failed to convert 'cabin' with value: 'COACH'"),
        ("origin=NCE&destination=LHR".into(), "Required parameter 'date' is not present."),
        (
            format!("destination=LHR&date={date}"),
            "Required parameter 'origin' is not present.",
        ),
        (
            format!("origin=NCE&date={date}"),
            "Required parameter 'destination' is not present.",
        ),
        (
            "origin=NCE&destination=LHR&date=tomorrow".into(),
            "Failed to convert 'date' with value: 'tomorrow'",
        ),
        (
            format!("origin=NCE&destination=nce&date={date}"),
            "origin and destination must differ",
        ),
        (
            format!("origin=NCE&destination=XXX&date={date}"),
            "Unknown destination airport: XXX",
        ),
        (format!("origin=x%20&destination=XXX&date={date}"), "Unknown origin airport: x "),
    ];
    for (query, detail) in cases {
        let (status, ct, body) = get(&pool, &format!("/travel/api/offers?{query}")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
        assert_eq!(ct, "application/problem+json");
        assert_eq!(body["detail"], detail, "{query}");
        assert_eq!(body["instance"], "/travel/api/offers");
        assert_eq!(body["status"], 400);
        assert_eq!(body["title"], "Bad Request");
    }
    let (status, _, body) = get(&pool, "/travel/api/offers?origin=NCE&destination=LHR&date=2000-01-01").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}
