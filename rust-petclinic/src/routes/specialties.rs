use axum::extract::State;
use axum::Json;

use super::{AppState, Id};
use crate::db;
use crate::dto::Specialty;
use crate::error::AppError;

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
