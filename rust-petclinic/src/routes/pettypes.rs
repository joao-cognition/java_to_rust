use axum::extract::State;
use axum::response::Response;
use axum::Json;

use super::{created, deleted, no_content_or_404, AppState, Body, Id};
use crate::db;
use crate::dto::{PetType, PetTypeFieldsInput, PetTypeInput};
use crate::error::AppError;
use crate::validation::check_pet_type;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<PetType>>, AppError> {
    let types = db::list_pet_types(&state.pool).await?;
    if types.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(types))
}

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<PetType>, AppError> {
    db::find_pet_type(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

/// `PetTypeFields`: any `id` in the body is ignored.
pub async fn create(
    State(state): State<AppState>,
    Body(fields): Body<PetTypeFieldsInput>,
) -> Result<Response, AppError> {
    let pet_type = PetTypeInput {
        name: fields.name,
        id: None,
    };
    check_pet_type(&pet_type, false)?;
    let name = pet_type.name.as_deref().unwrap_or_default();
    let saved = db::insert_pet_type(&state.pool, name).await?;
    Ok(created(format!("/api/pettypes/{}", saved.id), saved))
}

/// `PetType`: the body must carry an `id` (validated, then ignored in favour of the path).
pub async fn update(
    State(state): State<AppState>,
    Id(id): Id,
    Body(pet_type): Body<PetTypeInput>,
) -> Result<Response, AppError> {
    check_pet_type(&pet_type, true)?;
    let name = pet_type.name.as_deref().unwrap_or_default();
    no_content_or_404(db::update_pet_type(&state.pool, id, name).await?)
}

pub async fn delete(State(state): State<AppState>, Id(id): Id) -> Result<Response, AppError> {
    deleted(db::delete_pet_type(&state.pool, id).await?)
}
