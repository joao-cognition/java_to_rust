//! End-to-end tests against a real PostgreSQL loaded with the Java repo's seed data
//! (`scripts/init-db.sh`). They are `#[ignore]`d by default; run them with
//! `DATABASE_URL=postgres://petclinic:petclinic@localhost:5432/petclinic cargo test -- --ignored`.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use rust_petclinic::{app, db};
use serde_json::Value;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must point at a seeded petclinic database");
    app(db::connect(&url, 2).await.expect("connect"))
}

async fn get_json(app: axum::Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, json)
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn health_is_up_with_database() {
    let (status, json) = get_json(test_app().await, "/petclinic/actuator/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        json,
        serde_json::json!({"groups": ["liveness", "readiness"], "status": "UP"})
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn vets_are_listed_with_sorted_specialties() {
    let (status, json) = get_json(test_app().await, "/petclinic/api/vets").await;
    assert_eq!(status, StatusCode::OK);
    let vets = json.as_array().unwrap();
    assert_eq!(vets.len(), 6);
    let linda = vets.iter().find(|v| v["id"] == 3).unwrap();
    let names: Vec<_> = linda["specialties"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["dentistry", "surgery"]);
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn owner_lookup_nests_pets_and_visits() {
    let (status, json) = get_json(test_app().await, "/petclinic/api/owners/6").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["lastName"], "Coleman");
    let pets = json["pets"].as_array().unwrap();
    assert_eq!(pets[0]["name"], "Max");
    assert_eq!(pets[1]["name"], "Samantha");
    assert_eq!(pets[0]["visits"][0]["date"], "2011-03-04");
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn owner_search_is_prefix_match_and_404_when_empty() {
    let (status, json) = get_json(test_app().await, "/petclinic/api/owners?lastName=Dav").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 2);

    let (status, body) = get_json(test_app().await, "/petclinic/api/owners?lastName=Zzz").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.is_null());
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn missing_entities_are_404_with_empty_body() {
    for path in [
        "/petclinic/api/owners/99999",
        "/petclinic/api/pets/99999",
        "/petclinic/api/vets/99999",
        "/petclinic/api/pettypes/99999",
        "/petclinic/api/specialties/99999",
    ] {
        let (status, body) = get_json(test_app().await, path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(body.is_null(), "{path}");
    }
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn create_owner_returns_201_location_and_body() {
    let response = test_app()
        .await
        .oneshot(
            Request::post("/petclinic/api/owners")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"firstName":"Test","lastName":"Rustacean","address":"1 Crab St.","city":"Ferris","telephone":"0123456789"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let location = response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_string();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(location, format!("/api/owners/{}", json["id"]));
    assert_eq!(json["pets"], serde_json::json!([]));
    assert_eq!(json["lastName"], "Rustacean");
}
