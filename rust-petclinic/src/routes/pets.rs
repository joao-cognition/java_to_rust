use axum::extract::State;
use axum::Json;

use super::{AppState, Id};
use crate::db;
use crate::dto::Pet;
use crate::error::AppError;

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<Pet>, AppError> {
    db::find_pet(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}
