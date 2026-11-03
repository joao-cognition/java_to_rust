use axum::extract::State;
use axum::response::Response;
use axum::Json;

use super::{created, deleted, no_content_or_404, AppState, Body, Id};
use crate::db;
use crate::dto::{Specialty, SpecialtyInput};
use crate::error::AppError;
use crate::validation::check_specialty;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Specialty>>, AppError> {
    let specialties = db::list_specialties(&state.pool).await?;
    if specialties.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(specialties))
}

pub async fn get_one(
    State(state): State<AppState>,
    Id(id): Id,
) -> Result<Json<Specialty>, AppError> {
    db::find_specialty(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn create(
    State(state): State<AppState>,
    Body(specialty): Body<SpecialtyInput>,
) -> Result<Response, AppError> {
    check_specialty(&specialty)?;
    let name = specialty.name.as_deref().unwrap_or_default();
    let saved = db::save_specialty(&state.pool, specialty.id, name).await?;
    Ok(created(format!("/api/specialties/{}", saved.id), saved))
}

pub async fn update(
    State(state): State<AppState>,
    Id(id): Id,
    Body(specialty): Body<SpecialtyInput>,
) -> Result<Response, AppError> {
    check_specialty(&specialty)?;
    let name = specialty.name.as_deref().unwrap_or_default();
    match db::save_specialty(&state.pool, Some(id), name).await {
        Ok(_) => no_content_or_404(true),
        Err(AppError::Unexpected(_)) => no_content_or_404(false),
        Err(err) => Err(err),
    }
}

pub async fn delete(State(state): State<AppState>, Id(id): Id) -> Result<Response, AppError> {
    deleted(db::delete_specialty(&state.pool, id).await?)
}
