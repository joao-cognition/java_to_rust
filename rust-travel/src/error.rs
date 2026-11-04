//! RFC 9457 problem details, shaped like Spring's `ProblemDetail` (no `type` member: Spring omits `about:blank`).

use axum::{
    extract::OriginalUri,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProblemDetail {
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ProblemDetail {
    pub fn new(status: StatusCode, detail: impl Into<Option<String>>) -> Self {
        Self {
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail: detail.into(),
            instance: None,
        }
    }

    pub fn with_instance(mut self, path: &str) -> Self {
        self.instance = Some(path.to_string());
        self
    }
}

impl IntoResponse for ProblemDetail {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(self),
        )
            .into_response()
    }
}

/// Handler error: anything that can become a problem response.
#[derive(Debug)]
pub enum ApiError {
    Problem(ProblemDetail),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

impl From<ProblemDetail> for ApiError {
    fn from(p: ProblemDetail) -> Self {
        Self::Problem(p)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Problem(p) => p.into_response(),
            Self::Db(e) => {
                tracing::error!("database error: {e}");
                ProblemDetail::new(StatusCode::INTERNAL_SERVER_ERROR, None).into_response()
            }
        }
    }
}

/// Fallback for paths no route matches.
pub async fn no_resource(OriginalUri(uri): OriginalUri) -> ProblemDetail {
    let path = uri.path();
    let resource = path.trim_start_matches("/travel/");
    ProblemDetail::new(
        StatusCode::NOT_FOUND,
        Some(format!("No static resource {resource}.")),
    )
    .with_instance(path)
}
