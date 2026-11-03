//! HTTP routing. Paths mirror the Java service (`server.servlet.context-path=/petclinic`,
//! controllers under `/api`), so clients can point at either port.

mod owners;
mod pets;
mod pettypes;
mod specialties;
mod users;
mod vets;
mod visits;

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::CONTEXT_PATH;
use crate::dto::Health;
use crate::error::{
    problem_response, render_problem_details, AppError, ERROR_UNEXPECTED,
    TITLE_METHOD_NOT_SUPPORTED, TITLE_TYPE_MISMATCH,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub fn app(pool: PgPool) -> Router {
    let api = Router::new()
        // Hot path first (the `j2r bench` mix), then the rest of the OpenAPI contract.
        .route("/owners", get(owners::list).post(owners::create))
        .route(
            "/owners/{owner_id}",
            get(owners::get_one)
                .put(owners::update)
                .delete(owners::delete),
        )
        .route("/owners/{owner_id}/pets", post(owners::add_pet))
        .route(
            "/owners/{owner_id}/pets/{pet_id}",
            get(owners::get_pet).put(owners::update_pet),
        )
        .route(
            "/owners/{owner_id}/pets/{pet_id}/visits",
            post(owners::add_visit),
        )
        .route("/v2/owners", get(owners::page))
        .route("/vets", get(vets::list).post(vets::create))
        .route(
            "/vets/{vet_id}",
            get(vets::get_one).put(vets::update).delete(vets::delete),
        )
        .route(
            "/specialties",
            get(specialties::list).post(specialties::create),
        )
        .route(
            "/specialties/{specialty_id}",
            get(specialties::get_one)
                .put(specialties::update)
                .delete(specialties::delete),
        )
        .route("/pettypes", get(pettypes::list).post(pettypes::create))
        .route(
            "/pettypes/{pet_type_id}",
            get(pettypes::get_one)
                .put(pettypes::update)
                .delete(pettypes::delete),
        )
        .route("/pets", get(pets::list))
        .route(
            "/pets/{pet_id}",
            get(pets::get_one).put(pets::update).delete(pets::delete),
        )
        .route("/v2/pets", get(pets::page))
        .route("/visits", get(visits::list).post(visits::create))
        .route(
            "/visits/{visit_id}",
            get(visits::get_one)
                .put(visits::update)
                .delete(visits::delete),
        )
        .route("/users", post(users::create))
        .method_not_allowed_fallback(method_not_supported);

    let petclinic = Router::new()
        .route("/actuator/health", get(health))
        .nest("/api", api);

    Router::new()
        .nest(CONTEXT_PATH, petclinic)
        .fallback(not_found)
        .layer(axum::middleware::from_fn(render_problem_details))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(AppState { pool })
}

/// Like Actuator's aggregate health (which includes the `DataSource` indicator): `UP` only
/// while the database answers within [`HEALTH_DB_TIMEOUT`], otherwise `503` + `DOWN`.
const HEALTH_DB_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// Actuator's media type, sent by the Java service for `/actuator/health`.
const ACTUATOR_JSON: &str = "application/vnd.spring-boot.actuator.v3+json";

async fn health(State(state): State<AppState>) -> Response {
    let (status, body) = health_status(&state).await;
    let mut response = (status, body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(ACTUATOR_JSON),
    );
    response
}

async fn health_status(state: &AppState) -> (StatusCode, Json<Health>) {
    let ping = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.pool);
    match tokio::time::timeout(HEALTH_DB_TIMEOUT, ping).await {
        Ok(Ok(_)) => (StatusCode::OK, Json(Health::UP)),
        Ok(Err(err)) => {
            tracing::warn!(error = %err, "health check: database unavailable");
            (StatusCode::SERVICE_UNAVAILABLE, Json(Health::DOWN))
        }
        Err(_) => {
            tracing::warn!("health check: database did not answer within {HEALTH_DB_TIMEOUT:?}");
            (StatusCode::SERVICE_UNAVAILABLE, Json(Health::DOWN))
        }
    }
}

/// Unknown route. Spring raises `NoResourceFoundException`, which `ExceptionControllerAdvice`
/// reports as a `500` problem (its generic `Exception` handler), so the Rust service does too.
async fn not_found() -> Response {
    problem_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "NoResourceFoundException",
        ERROR_UNEXPECTED,
    )
}

/// Known route, unsupported method: Spring's `HttpRequestMethodNotSupportedException` → 500.
async fn method_not_supported() -> Response {
    problem_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        TITLE_METHOD_NOT_SUPPORTED,
        ERROR_UNEXPECTED,
    )
}

/// `201 Created` with the relative `Location` the Java controllers set (no context path).
pub fn created<T: serde::Serialize>(location: String, body: T) -> Response {
    let mut response = (StatusCode::CREATED, Json(body)).into_response();
    if let Ok(value) = HeaderValue::from_str(&location) {
        response.headers_mut().insert(header::LOCATION, value);
    }
    response
}

/// `204` as returned by Java handlers typed `ResponseEntity<XxxDto>`: empty body but a JSON
/// content type.
pub fn no_content_json() -> Response {
    (
        StatusCode::NO_CONTENT,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
    )
        .into_response()
}

/// `true` → [`no_content_json`], `false` → empty `404`.
pub fn no_content_or_404(found: bool) -> Result<Response, AppError> {
    if found {
        Ok(no_content_json())
    } else {
        Err(AppError::NotFound)
    }
}

/// `204` of the Java `delete*` handlers (`ResponseEntity<Void>`: no content type), or `404`.
pub fn deleted(found: bool) -> Result<Response, AppError> {
    if found {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AppError::NotFound)
    }
}

/// Multi-segment path id extractor (`/owners/{ownerId}/pets/{petId}`), same rejection as [`Id`].
pub struct Ids<T>(pub T);

impl<S: Send + Sync, T> FromRequestParts<S> for Ids<T>
where
    T: serde::de::DeserializeOwned + Send,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(axum::extract::Path(ids)) => Ok(Ids(ids)),
            Err(rejection) => Err(type_mismatch(rejection)),
        }
    }
}

/// `page` / `size` query parameters of the `/api/v2` endpoints, validated like the
/// generated Java interface (`@Min(0)` page, `@Min(1) @Max(100)` size, defaults 0 / 20).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
    page: Option<String>,
    size: Option<String>,
    pub last_name: Option<String>,
}

impl PageParams {
    pub fn resolve(&self) -> Result<(i32, i32), AppError> {
        let parse = |value: &Option<String>, default: i32| match value {
            None => Ok(default),
            Some(v) => v
                .trim()
                .parse::<i32>()
                .map_err(|_| AppError::unexpected(TITLE_TYPE_MISMATCH)),
        };
        let page = parse(&self.page, 0)?;
        let size = parse(&self.size, 20)?;
        if page < 0 || !(1..=100).contains(&size) {
            return Err(AppError::unexpected("ConstraintViolationException"));
        }
        Ok((page, size))
    }
}

/// Path id extractor whose rejection is reported as a `ProblemDetail`
/// (Spring: `MethodArgumentTypeMismatchException`, a `500` through the generic handler)
/// instead of axum's plain-text 400.
pub struct Id(pub i32);

impl<S: Send + Sync> FromRequestParts<S> for Id {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<i32>::from_request_parts(parts, state).await {
            Ok(axum::extract::Path(id)) => Ok(Id(id)),
            Err(rejection) => Err(type_mismatch(rejection)),
        }
    }
}

fn type_mismatch(rejection: PathRejection) -> Response {
    tracing::debug!(%rejection, "path parameter rejected");
    problem_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        TITLE_TYPE_MISMATCH,
        ERROR_UNEXPECTED,
    )
}

/// JSON body extractor that converts axum's rejection into an [`AppError`].
pub struct Body<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for Body<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Body(value)),
            Err(rejection) => Err(malformed_body(rejection)),
        }
    }
}

fn malformed_body(rejection: JsonRejection) -> Response {
    AppError::MalformedRequest(rejection.body_text()).into_response()
}
