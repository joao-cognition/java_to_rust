//! Error type shared by all handlers and its mapping to HTTP responses.
//!
//! The Java service answers `404` with an empty body (`new ResponseEntity<>(HttpStatus.NOT_FOUND)`)
//! and every other failure with an RFC 9457 `ProblemDetail` built by `ExceptionControllerAdvice`.
//! That advice only maps Bean Validation errors to `400` (and `DataIntegrityViolationException`
//! to `404`); everything else, including a non-numeric path id, an unknown route or an
//! unreadable JSON body, falls into its `@ExceptionHandler(Exception.class)` and is reported
//! as `500` with the exception's simple class name as `title`. The Rust service reproduces
//! those observable responses so that `bench/parity.sh` can diff both services.
//! The problem's `type` is the full request URL, which only the request-aware middleware knows,
//! so handlers attach a [`ProblemDetail`] to the response extensions and
//! [`render_problem_details`] turns it into the JSON body.

use axum::extract::{OriginalUri, Request};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

pub const ERROR_UNEXPECTED: &str = "An unexpected error occurred while processing your request";
pub const ERROR_INVALID_REQUEST: &str = "The request contains invalid or missing parameters";
pub const ERROR_DATA_INTEGRITY: &str =
    "The requested resource could not be processed due to a data constraint violation";

/// Problem titles. Java uses the simple class name of the thrown exception; clients that
/// switch on `title` keep working against the Rust service.
pub const TITLE_UNEXPECTED: &str = "DataAccessException";
pub const TITLE_VALIDATION: &str = "MethodArgumentNotValidException";
pub const TITLE_MALFORMED_REQUEST: &str = "HttpMessageNotReadableException";
pub const TITLE_TYPE_MISMATCH: &str = "MethodArgumentTypeMismatchException";
pub const TITLE_DATA_INTEGRITY: &str = "DataIntegrityViolationException";
pub const TITLE_METHOD_NOT_SUPPORTED: &str = "HttpRequestMethodNotSupportedException";

/// One entry of `schemaValidationErrors` (`ValidationMessage` in the contract, with the
/// extra properties Java adds for field errors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationMessage {
    pub message: String,
    pub field: String,
    pub rejected_value: String,
    pub default_message: String,
}

impl ValidationMessage {
    pub fn new(field: &str, rejected_value: Option<&str>, default_message: &str) -> Self {
        let rejected_value = rejected_value.unwrap_or("null").to_string();
        Self {
            message: format!(
                "Field '{field}' {default_message} (rejected value: {rejected_value})"
            ),
            field: field.to_string(),
            rejected_value,
            default_message: default_message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDetail {
    pub detail: String,
    pub instance: String,
    pub status: u16,
    pub title: String,
    #[serde(rename = "type")]
    pub type_url: String,
    pub timestamp: String,
    pub schema_validation_errors: Vec<ValidationMessage>,
}

impl ProblemDetail {
    pub fn new(status: StatusCode, title: &str, detail: &str) -> Self {
        Self {
            detail: detail.to_string(),
            instance: String::new(),
            status: status.as_u16(),
            title: title.to_string(),
            type_url: String::new(),
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            schema_validation_errors: Vec::new(),
        }
    }

    pub fn with_errors(mut self, errors: Vec<ValidationMessage>) -> Self {
        self.schema_validation_errors = errors;
        self
    }

    fn into_response_for(mut self, request_url: String, request_path: String) -> Response {
        self.type_url = request_url;
        self.instance = request_path;
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, axum::Json(self)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("validation failed")]
    Validation(Vec<ValidationMessage>),
    #[error("malformed request: {0}")]
    MalformedRequest(String),
    /// Constraint violations and data exceptions (SQLSTATE classes 22/23): Java's advice
    /// answers `404` with a problem body.
    #[error("data integrity violation")]
    DataIntegrity,
    /// Any other exception the Java service lets escape, reported as `500` titled with the
    /// Java exception class name (e.g. `ConstraintViolationException`).
    #[error("unexpected error: {0}")]
    Unexpected(&'static str),
    #[error("database error: {0}")]
    Database(sqlx::Error),
}

impl AppError {
    pub fn unexpected(title: &'static str) -> Self {
        AppError::Unexpected(title)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        let integrity = matches!(
            &err,
            sqlx::Error::Database(db) if db.code().is_some_and(|c| c.starts_with("22") || c.starts_with("23"))
        );
        if integrity {
            AppError::DataIntegrity
        } else {
            AppError::Database(err)
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let problem = match self {
            AppError::NotFound => return StatusCode::NOT_FOUND.into_response(),
            AppError::Validation(errors) => ProblemDetail::new(
                StatusCode::BAD_REQUEST,
                TITLE_VALIDATION,
                ERROR_INVALID_REQUEST,
            )
            .with_errors(errors),
            AppError::MalformedRequest(reason) => {
                tracing::debug!(%reason, "malformed request");
                ProblemDetail::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    TITLE_MALFORMED_REQUEST,
                    ERROR_UNEXPECTED,
                )
            }
            AppError::DataIntegrity => ProblemDetail::new(
                StatusCode::NOT_FOUND,
                TITLE_DATA_INTEGRITY,
                ERROR_DATA_INTEGRITY,
            ),
            AppError::Unexpected(title) => {
                ProblemDetail::new(StatusCode::INTERNAL_SERVER_ERROR, title, ERROR_UNEXPECTED)
            }
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                ProblemDetail::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    TITLE_UNEXPECTED,
                    ERROR_UNEXPECTED,
                )
            }
        };
        let mut response = StatusCode::from_u16(problem.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response();
        response.extensions_mut().insert(problem);
        response
    }
}

/// Request path (`/petclinic/api/...`) like `HttpServletRequest#getRequestURI`.
fn request_path(request: &Request) -> String {
    request
        .extensions()
        .get::<OriginalUri>()
        .map(|u| u.0.path().to_string())
        .unwrap_or_else(|| request.uri().path().to_string())
}

/// Absolute request URL (`http://host/petclinic/api/...`) like `HttpServletRequest#getRequestURL`.
fn request_url(request: &Request, path: &str) -> String {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    format!("http://{host}{path}")
}

/// Middleware that renders a [`ProblemDetail`] left in the response extensions by
/// [`AppError::into_response`] (or by the fallback handlers) into the JSON body.
pub async fn render_problem_details(request: Request, next: Next) -> Response {
    let path = request_path(&request);
    let url = request_url(&request, &path);
    let mut response = next.run(request).await;
    match response.extensions_mut().remove::<ProblemDetail>() {
        Some(problem) => problem.into_response_for(url, path),
        None => response,
    }
}

/// Wraps a status code + problem into a response the middleware will render.
pub fn problem_response(status: StatusCode, title: &str, detail: &str) -> Response {
    let mut response = status.into_response();
    response
        .extensions_mut()
        .insert(ProblemDetail::new(status, title, detail));
    response
}
