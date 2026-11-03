//! HTTP routing. Endpoint handlers are added under `api()` as they are ported.

use std::time::Duration;

use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::PgPool;

use crate::config::CONTEXT_PATH;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub fn app(pool: PgPool) -> Router {
    let petclinic = Router::new().route("/actuator/health", get(health)).nest("/api", api());
    Router::new().nest(CONTEXT_PATH, petclinic).with_state(AppState { pool })
}

fn api() -> Router<AppState> {
    Router::new()
}

/// Field order matches Spring Boot Actuator's response byte for byte.
#[derive(Serialize)]
pub struct Health {
    groups: [&'static str; 2],
    status: &'static str,
}

const GROUPS: [&str; 2] = ["liveness", "readiness"];
/// Actuator's `db` indicator is part of the aggregate status, so health is `UP` only while
/// PostgreSQL answers.
const DB_TIMEOUT: Duration = Duration::from_secs(1);

const ACTUATOR_V3_JSON: &str = "application/vnd.spring-boot.actuator.v3+json";

/// Actuator answers `application/json` only when the client asks for it before any wildcard
/// or the actuator media type; otherwise (no `Accept`, `*/*`) it uses its own v3 media type.
fn health_content_type(headers: &HeaderMap) -> &'static str {
    let accept = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok()).unwrap_or("");
    for media in accept.split(',').map(|m| m.split(';').next().unwrap_or("").trim()) {
        match media {
            "application/json" => return "application/json",
            ACTUATOR_V3_JSON | "*/*" | "application/*" => return ACTUATOR_V3_JSON,
            _ => {}
        }
    }
    ACTUATOR_V3_JSON
}

async fn health(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let ping = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.pool);
    let up = match tokio::time::timeout(DB_TIMEOUT, ping).await {
        Ok(Ok(_)) => true,
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "health: database unavailable");
            false
        }
        Err(_) => {
            tracing::warn!("health: database did not answer within {DB_TIMEOUT:?}");
            false
        }
    };
    let (code, status) = if up {
        (StatusCode::OK, "UP")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "DOWN")
    };
    let mut res = (code, Json(Health { groups: GROUPS, status })).into_response();
    res.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(health_content_type(&headers)));
    res
}
