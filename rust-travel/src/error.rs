//! RFC 9457 problem details shaped exactly like Spring's `ProblemDetail` (no `type`, since it is `about:blank`).

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug)]
pub struct Problem {
    status: StatusCode,
    detail: String,
    instance: String,
}

#[derive(Serialize)]
struct Body<'a> {
    detail: &'a str,
    instance: &'a str,
    status: u16,
    title: &'a str,
}

impl Problem {
    pub fn new(status: StatusCode, detail: impl Into<String>, instance: &str) -> Self {
        Self {
            status,
            detail: detail.into(),
            instance: instance.to_owned(),
        }
    }

    pub fn bad_request(detail: impl Into<String>, instance: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail, instance)
    }

    pub fn not_found(detail: impl Into<String>, instance: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, detail, instance)
    }

    /// Spring's `MissingServletRequestParameterException` (also raised when a required value converts to null).
    pub fn missing(name: &str, instance: &str) -> Self {
        Self::bad_request(
            format!("Required parameter '{name}' is not present."),
            instance,
        )
    }

    /// Spring's `MethodArgumentTypeMismatchException`.
    pub fn conversion(name: &str, value: &str, instance: &str) -> Self {
        Self::bad_request(
            format!("Failed to convert '{name}' with value: '{value}'"),
            instance,
        )
    }

    /// Spring's `HandlerMethodValidationException` (`@Min`, `@Max`, `@NotBlank` on controller parameters).
    pub fn validation(instance: &str) -> Self {
        Self::bad_request("Validation failure", instance)
    }

    pub fn internal(err: impl std::fmt::Display, instance: &str) -> Self {
        tracing::error!("Unhandled error on {instance}: {err}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected error",
            instance,
        )
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let body = Body {
            detail: &self.detail,
            instance: &self.instance,
            status: self.status.as_u16(),
            title: self.status.canonical_reason().unwrap_or(""),
        };
        let bytes = serde_json::to_vec(&body).unwrap_or_default();
        let mut res = (self.status, bytes).into_response();
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        res
    }
}
