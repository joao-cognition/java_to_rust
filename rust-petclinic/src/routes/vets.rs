use axum::extract::State;
use axum::response::Response;
use axum::Json;

use super::{created, deleted, no_content_or_404, AppState, Body, Id};
use crate::db;
use crate::dto::{Vet, VetInput};
use crate::error::AppError;
use crate::validation::check_vet;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Vet>>, AppError> {
    let vets = db::list_vets(&state.pool).await?;
    if vets.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(vets))
}

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<Vet>, AppError> {
    db::find_vet(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn create(
    State(state): State<AppState>,
    Body(vet): Body<VetInput>,
) -> Result<Response, AppError> {
    check_vet(&vet)?;
    let saved = db::save_vet(&state.pool, vet.id, &vet).await?;
    Ok(created(format!("/api/vets/{}", saved.id), saved))
}

pub async fn update(
    State(state): State<AppState>,
    Id(id): Id,
    Body(vet): Body<VetInput>,
) -> Result<Response, AppError> {
    check_vet(&vet)?;
    no_content_or_404(db::update_vet(&state.pool, id, &vet).await?)
}

pub async fn delete(State(state): State<AppState>, Id(id): Id) -> Result<Response, AppError> {
    deleted(db::delete_vet(&state.pool, id).await?)
}
