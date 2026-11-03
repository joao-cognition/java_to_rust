use axum::extract::State;
use axum::response::Response;
use axum::Json;

use super::{created, deleted, no_content_or_404, AppState, Body, Id};
use crate::db;
use crate::dto::{Visit, VisitInput};
use crate::error::AppError;
use crate::validation::check_visit;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Visit>>, AppError> {
    let visits = db::list_visits(&state.pool).await?;
    if visits.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(visits))
}

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<Visit>, AppError> {
    db::find_visit(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn create(
    State(state): State<AppState>,
    Body(visit): Body<VisitInput>,
) -> Result<Response, AppError> {
    check_visit(&visit, true)?;
    let saved = db::save_visit(&state.pool, visit.id, visit.pet_id, &visit).await?;
    Ok(created(format!("/api/visits/{}", saved.id), saved))
}

pub async fn update(
    State(state): State<AppState>,
    Id(id): Id,
    Body(visit): Body<VisitInput>,
) -> Result<Response, AppError> {
    check_visit(&visit, false)?;
    no_content_or_404(db::update_visit(&state.pool, id, &visit).await?)
}

pub async fn delete(State(state): State<AppState>, Id(id): Id) -> Result<Response, AppError> {
    deleted(db::delete_visit(&state.pool, id).await?)
}
