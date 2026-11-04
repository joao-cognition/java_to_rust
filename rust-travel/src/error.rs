//! RFC 9457 problem details shaped like Spring's `ProblemDetail` output, served as `application/problem+json`.

use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub detail: String,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    /// `BadRequestException`.
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail)
    }

    /// `NotFoundException`.
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, detail)
    }

    /// `HandlerMethodValidationException` (`@Min`, `@Max`, `@NotBlank` on a request parameter).
    pub fn validation() -> Self {
        Self::bad_request("Validation failure")
    }

    /// `MissingServletRequestParameterException`.
    pub fn missing(name: &str) -> Self {
        Self::bad_request(format!("Required parameter '{name}' is not present."))
    }

    /// `MethodArgumentTypeMismatchException`.
    pub fn convert(name: &str, value: &str) -> Self {
        Self::bad_request(format!("Failed to convert '{name}' with value: '{value}'"))
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("database error: {e}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
    }
}

#[derive(Serialize)]
struct Problem<'a> {
    title: &'a str,
    status: u16,
    detail: &'a str,
    instance: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut res = self.status.into_response();
        res.extensions_mut().insert(self);
        res
    }
}

/// Renders an [`ApiError`] left by a handler, with the request path as `instance` like Spring does.
pub async fn render_problems(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut res = next.run(req).await;
    let Some(err) = res.extensions_mut().remove::<ApiError>() else {
        return res;
    };
    let body = Problem {
        title: err.status.canonical_reason().unwrap_or(""),
        status: err.status.as_u16(),
        detail: &err.detail,
        instance: &path,
    };
    let mut out = (err.status, serde_json::to_string(&body).unwrap_or_default()).into_response();
    out.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    out
}

/// Spring's `NoResourceFoundException` for paths no controller maps.
pub async fn no_resource(uri: Uri) -> ApiError {
    let rest = uri
        .path()
        .strip_prefix("/travel/")
        .unwrap_or(uri.path())
        .trim_start_matches('/');
    ApiError::not_found(format!("No static resource {rest}."))
}
