//! Router tests. The `UP` case needs the compose PostgreSQL (or the CI service container),
//! reached through the same `POSTGRES_*` variables as the server.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use rust_petclinic::config::{connect_options, Config};
use rust_petclinic::{app, db};

async fn get(router: axum::Router, uri: &str) -> (StatusCode, String) {
    let (status, _, body) = get_with_accept(router, uri, None).await;
    (status, body)
}

async fn get_with_accept(router: axum::Router, uri: &str, accept: Option<&str>) -> (StatusCode, String, String) {
    let mut req = Request::get(uri);
    if let Some(a) = accept {
        req = req.header("accept", a);
    }
    let res = router.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, content_type, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn health_is_up_with_the_java_body() {
    let config = Config::from_env().unwrap();
    let pool = db::connect(config.db, 2).await.expect("postgres reachable via POSTGRES_URL");
    let (status, body) = get(app(pool), "/petclinic/actuator/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"groups":["liveness","readiness"],"status":"UP"}"#);
}

#[tokio::test]
async fn health_is_down_without_a_database() {
    let opts = connect_options("localhost:1/petclinic", "petclinic", "petclinic").unwrap();
    let (status, body) = get(app(db::connect_lazy(opts)), "/petclinic/actuator/health").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, r#"{"groups":["liveness","readiness"],"status":"DOWN"}"#);
}

#[tokio::test]
async fn health_is_not_served_outside_the_context_path() {
    let opts = connect_options("localhost:1/petclinic", "petclinic", "petclinic").unwrap();
    let (status, _) = get(app(db::connect_lazy(opts)), "/actuator/health").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_content_type_follows_actuator_negotiation() {
    let opts = connect_options("localhost:1/petclinic", "petclinic", "petclinic").unwrap();
    let router = app(db::connect_lazy(opts));
    let v3 = "application/vnd.spring-boot.actuator.v3+json";
    for (accept, expected) in [
        (None, v3),
        (Some("*/*"), v3),
        (Some(v3), v3),
        (Some("application/json"), "application/json"),
        (Some("application/json, */*"), "application/json"),
    ] {
        let (_, content_type, _) = get_with_accept(router.clone(), "/petclinic/actuator/health", accept).await;
        assert_eq!(content_type, expected, "accept={accept:?}");
    }
}
