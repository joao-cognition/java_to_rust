//! Shared helpers for the HTTP integration tests: they drive `rust_travel::app` in-process against the PostgreSQL
//! database seeded by the Java service (`DATABASE_URL`, default `postgres://travel:travel@localhost:5432/travel`).
//! When the database is unreachable the tests print a note and pass, so `cargo test` stays usable offline.
#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rust_travel::{app, AppState};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

pub struct Api {
    app: axum::Router,
}

pub struct Reply {
    pub status: StatusCode,
    pub content_type: String,
    pub json: Value,
}

impl Api {
    pub async fn connect() -> Option<Api> {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
        let pool = match PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&url)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skipping: database unreachable ({e})");
                return None;
            }
        };
        if sqlx::query("SELECT 1 FROM flight LIMIT 1")
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .is_none()
        {
            eprintln!("skipping: database not seeded (start the Java service once with --spring.profiles.active=postgres)");
            return None;
        }
        Some(Api {
            app: app(AppState { pool }),
        })
    }

    pub async fn get(&self, path: &str) -> Reply {
        let req = Request::get(format!("/travel/api/{path}")).body(Body::empty()).unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let content_type = res
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap().to_owned())
            .unwrap_or_default();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        Reply {
            status,
            content_type,
            json,
        }
    }

    /// Asserts a Spring-style problem response.
    pub async fn problem(&self, path: &str, status: u16, detail: &str) {
        let r = self.get(path).await;
        assert_eq!(r.status.as_u16(), status, "{path}: {}", r.json);
        assert_eq!(r.content_type, "application/problem+json", "{path}");
        let title = r.status.canonical_reason().unwrap();
        let instance = format!("/travel/api/{}", path.split('?').next().unwrap());
        assert_eq!(
            r.json,
            serde_json::json!({"detail": detail, "instance": instance, "status": status, "title": title}),
            "{path}"
        );
    }
}

/// First schedule day (tomorrow relative to the Java seeding run).
pub async fn first_day(api: &Api) -> String {
    let r = api.get("flights/1").await;
    r.json["departure"].as_str().unwrap()[..10].to_owned()
}
