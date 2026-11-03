use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::{AppState, Body};
use crate::db;
use crate::dto::UserInput;
use crate::error::AppError;
use crate::validation::check_user;

/// `POST /users`: at least one role is required (Java throws `IllegalArgumentException`, a
/// 500) and role names get the `ROLE_` prefix. Java answers `201` without a `Location`.
pub async fn create(
    State(state): State<AppState>,
    Body(user): Body<UserInput>,
) -> Result<Response, AppError> {
    check_user(&user)?;
    let mut roles: Vec<String> = Vec::new();
    for name in user
        .roles
        .iter()
        .flatten()
        .filter_map(|r| r.name.as_deref())
    {
        let name = if name.starts_with("ROLE_") {
            name.to_string()
        } else {
            format!("ROLE_{name}")
        };
        roles.push(name);
    }
    if roles.is_empty() {
        return Err(AppError::unexpected("IllegalArgumentException"));
    }
    let saved = db::insert_user(&state.pool, &user, &roles).await?;
    Ok((StatusCode::CREATED, Json(saved)).into_response())
}
