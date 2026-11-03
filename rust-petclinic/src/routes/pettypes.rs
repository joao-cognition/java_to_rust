use axum::extract::State;
use axum::Json;

use super::{AppState, Id};
use crate::db;
use crate::dto::PetType;
use crate::error::AppError;

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
