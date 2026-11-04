//! Integration-test helpers. Tests run against the compose PostgreSQL seeded by the Java service
//! (`DATABASE_URL`, default `postgres://travel:travel@localhost:5432/travel`) and skip when it is unreachable.

#![allow(dead_code)]

use axum::{body::Body, http::Request, Router};
use http_body_util::BodyExt;
use rust_travel::db;
use sqlx::postgres::PgConnectOptions;
use std::{str::FromStr, time::Duration};
use tower::ServiceExt;

pub const DEFAULT_DATABASE_URL: &str = "postgres://travel:travel@localhost:5432/travel";

/// The full router against the test database, or `None` (with a note on stderr) when no database answers.
pub async fn app() -> Option<Router> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.into());
    let options = match PgConnectOptions::from_str(&url) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("skipping: bad DATABASE_URL {url}: {e}");
            return None;
        }
    };
    match db::connect_with(options, Duration::from_secs(3)).await {
        Ok(pool) => Some(rust_travel::app(pool)),
        Err(e) => {
            eprintln!("skipping: no database at {url}: {e}");
            None
        }
    }
}

/// `let app = require_app!();` returns early from the test when no database is reachable.
#[macro_export]
macro_rules! require_app {
    () => {
        match common::app().await {
            Some(app) => app,
            None => return,
        }
    };
}

pub struct TestResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: serde_json::Value,
}

/// `GET <path>` (full path, e.g. `/travel/api/airports`) through the router, body parsed as JSON.
pub async fn get(app: &Router, path: &str) -> TestResponse {
    let response = app
        .clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    TestResponse {
        status,
        content_type,
        body,
    }
}
