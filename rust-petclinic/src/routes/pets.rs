use axum::extract::{Query, State};
use axum::response::Response;
use axum::Json;

use super::{deleted, no_content_or_404, AppState, Body, Id, PageParams};
use crate::db;
use crate::dto::{Page, Pet, PetInput};
use crate::error::AppError;
use crate::validation::check_pet;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Pet>>, AppError> {
    let pets = db::list_pets(&state.pool).await?;
    if pets.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(pets))
}

pub async fn page(
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Result<Json<Page<Pet>>, AppError> {
    let (page, size) = params.resolve()?;
    Ok(Json(db::pets_page(&state.pool, page, size).await?))
}

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<Pet>, AppError> {
    db::find_pet(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn update(
    State(state): State<AppState>,
    Id(id): Id,
    Body(pet): Body<PetInput>,
) -> Result<Response, AppError> {
    check_pet(&pet, true)?;
    no_content_or_404(db::update_pet(&state.pool, id, &pet).await?)
}

pub async fn delete(State(state): State<AppState>, Id(id): Id) -> Result<Response, AppError> {
    deleted(db::delete_pet(&state.pool, id).await?)
}
