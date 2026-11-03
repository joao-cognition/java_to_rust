use axum::extract::{Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use super::{AppState, Body, Id};
use crate::db;
use crate::dto::{Owner, OwnerFields};
use crate::error::AppError;
use crate::validation::validate_owner_fields;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    last_name: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Owner>>, AppError> {
    let owners = db::list_owners(&state.pool, params.last_name.as_deref()).await?;
    if owners.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(owners))
}

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<Owner>, AppError> {
    db::find_owner(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn create(
    State(state): State<AppState>,
    Body(fields): Body<OwnerFields>,
) -> Result<Response, AppError> {
    let errors = validate_owner_fields(&fields);
    if !errors.is_empty() {
        return Err(AppError::Validation(errors));
    }
    let owner = db::insert_owner(&state.pool, &fields).await?;
    // Same relative Location as the Java controller (`/api/owners/{id}`, no context path).
    let location = HeaderValue::from_str(&format!("/api/owners/{}", owner.id))
        .expect("numeric id is a valid header value");
    let mut response = (StatusCode::CREATED, Json(owner)).into_response();
    response.headers_mut().insert(header::LOCATION, location);
    Ok(response)
}
