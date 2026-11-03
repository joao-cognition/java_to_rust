//! HTTP routing. Paths mirror the Java service (`server.servlet.context-path=/petclinic`,
//! controllers under `/api`), so clients can point at either port.

mod owners;
mod pets;
mod pettypes;
mod specialties;
mod vets;

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::CONTEXT_PATH;
use crate::dto::Health;
use crate::error::{
    problem_response, render_problem_details, AppError, ERROR_UNEXPECTED, TITLE_TYPE_MISMATCH,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub fn app(pool: PgPool) -> Router {
    let api = Router::new()
        .route("/vets", get(vets::list))
        .route("/vets/{vet_id}", get(vets::get_one))
        .route("/specialties", get(specialties::list))
        .route("/specialties/{specialty_id}", get(specialties::get_one))
        .route("/pettypes", get(pettypes::list))
        .route("/pettypes/{pet_type_id}", get(pettypes::get_one))
        .route("/owners", get(owners::list).post(owners::create))
        .route("/owners/{owner_id}", get(owners::get_one))
        .route("/pets/{pet_id}", get(pets::get_one));

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

async fn health() -> Json<Health> {
    Json(Health::UP)
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
