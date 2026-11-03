//! RFC 9457 problem details shaped like Spring's `ProblemDetail` output:
//! `{"detail":…,"instance":<request path>,"status":…,"title":<reason phrase>}` with `application/problem+json`.

use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail)
    }
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, detail)
    }
    /// Bean-validation failure on a request parameter (`@Min`, `@Max`, `@NotBlank`).
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
    pub fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("database error: {e}");
        Self::internal()
    }
}

#[derive(Serialize)]
struct ProblemBody<'a> {
    detail: &'a str,
    instance: &'a str,
    status: u16,
    title: &'a str,
}

/// Rendered by [`problem_instance`], which knows the request path.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut res = self.status.into_response();
        res.extensions_mut().insert(self);
        res
    }
}

/// Turns an [`ApiError`] left in the response extensions into the problem body, with the raw request path as
/// `instance` (what Tomcat's `getRequestURI()` returns).
pub async fn problem_instance(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut res = next.run(req).await;
    let Some(err) = res.extensions_mut().remove::<ApiError>() else {
        return res;
    };
    let body = ProblemBody {
        detail: &err.detail,
        instance: &path,
        status: err.status.as_u16(),
        title: err.status.canonical_reason().unwrap_or(""),
    };
    let mut out = (err.status, serde_json::to_string(&body).unwrap_or_default()).into_response();
    out.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
    out
}

/// Spring's `NoResourceFoundException` for paths no controller maps: `No static resource api/nope.`
pub async fn no_static_resource(uri: Uri) -> ApiError {
    let path = uri.path();
    let rest = path.strip_prefix("/travel/").unwrap_or(path.trim_start_matches('/'));
    let cleaned: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    ApiError::not_found(format!("No static resource {}.", cleaned.join("/")))
}

/// Spring's `HttpRequestMethodNotSupportedException`.
pub async fn method_not_allowed(method: Method) -> ApiError {
    ApiError::new(StatusCode::METHOD_NOT_ALLOWED, format!("Method '{method}' is not supported."))
}
