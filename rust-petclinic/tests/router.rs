//! Router tests that do not need a database: routing, the health probe without a database, and the error paths
//! (404 / 400) which are answered before any query is issued.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use rust_petclinic::{app, db};
use serde_json::Value;
use tower::ServiceExt;

fn test_app() -> axum::Router {
    // Lazy pool: no connection is opened unless a handler actually queries.
    let pool = db::connect_lazy("postgres://petclinic:petclinic@127.0.0.1:1/petclinic").unwrap();
    app(pool)
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_is_down_without_database() {
    let response = test_app()
        .oneshot(
            Request::get("/petclinic/actuator/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body_json(response).await,
        serde_json::json!({"groups": ["liveness", "readiness"], "status": "DOWN"})
    );
}

#[tokio::test]
async fn unknown_route_is_problem_detail_500_like_spring() {
    let response = test_app()
        .oneshot(
            Request::get("/petclinic/api/nope")
                .header(header::HOST, "localhost:9977")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    let json = body_json(response).await;
    assert_eq!(json["status"], 500);
    assert_eq!(json["type"], "http://localhost:9977/petclinic/api/nope");
    assert_eq!(json["instance"], "/petclinic/api/nope");
    assert_eq!(json["title"], "NoResourceFoundException");
    assert_eq!(
        json["detail"],
        "An unexpected error occurred while processing your request"
    );
    assert_eq!(json["schemaValidationErrors"], serde_json::json!([]));
    assert!(json["timestamp"].is_string());
}

#[tokio::test]
async fn non_numeric_id_is_500_problem_detail_like_spring() {
    for path in [
        "/petclinic/api/owners/abc",
        "/petclinic/api/vets/abc",
        "/petclinic/api/pets/1.5",
        "/petclinic/api/pettypes/x",
        "/petclinic/api/specialties/x",
    ] {
        let response = test_app()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{path}"
        );
        let json = body_json(response).await;
        assert_eq!(json["status"], 500);
        assert_eq!(json["instance"], path);
        assert_eq!(json["title"], "MethodArgumentTypeMismatchException");
    }
}

#[tokio::test]
async fn create_owner_with_invalid_body_is_400_with_field_errors() {
    let response = test_app()
        .oneshot(
            Request::post("/petclinic/api/owners")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"firstName":"George","lastName":"Franklin","address":"x","city":"Madison","telephone":"not-a-number"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["title"], "MethodArgumentNotValidException");
    assert_eq!(
        json["detail"],
        "The request contains invalid or missing parameters"
    );
    let errors = json["schemaValidationErrors"].as_array().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["field"], "telephone");
    assert_eq!(errors[0]["rejectedValue"], "not-a-number");
    assert_eq!(errors[0]["defaultMessage"], "must match \"^[0-9]*$\"");
    assert_eq!(
        errors[0]["message"],
        "Field 'telephone' must match \"^[0-9]*$\" (rejected value: not-a-number)"
    );
}

#[tokio::test]
async fn create_owner_with_missing_fields_lists_every_field() {
    let response = test_app()
        .oneshot(
            Request::post("/petclinic/api/owners")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    let fields: Vec<&str> = json["schemaValidationErrors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    assert_eq!(
        fields,
        vec!["firstName", "lastName", "address", "city", "telephone"]
    );
}

#[tokio::test]
async fn create_owner_with_malformed_json_is_500_like_spring() {
    let response = test_app()
        .oneshot(
            Request::post("/petclinic/api/owners")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let json = body_json(response).await;
    assert_eq!(json["status"], 500);
    assert_eq!(json["title"], "HttpMessageNotReadableException");
}

#[tokio::test]
async fn create_owner_coerces_scalars_like_jackson() {
    let response = test_app()
        .oneshot(
            Request::post("/petclinic/api/owners")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"firstName": 1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["title"], "MethodArgumentNotValidException");
    let first_name = json["schemaValidationErrors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["field"] == "firstName")
        .expect("firstName error");
    assert_eq!(first_name["rejectedValue"], "1");
    assert!(first_name["defaultMessage"]
        .as_str()
        .unwrap()
        .starts_with("must match"));
}

#[tokio::test]
async fn cors_is_permissive() {
    let response = test_app()
        .oneshot(
            Request::options("/petclinic/api/vets")
                .header(header::ORIGIN, "http://localhost:4200")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
}
