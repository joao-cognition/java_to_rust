use axum::extract::State;
use axum::Json;

use super::{AppState, Id};
use crate::db;
use crate::dto::Vet;
use crate::error::AppError;

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
