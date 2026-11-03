use axum::extract::{Query, State};
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use super::{
    created, deleted, no_content_json, no_content_or_404, AppState, Body, Id, Ids, PageParams,
};
use crate::db;
use crate::dto::{Owner, OwnerFields, Page, Pet, PetInput, VisitInput};
use crate::error::AppError;
use crate::validation::{check_owner, check_pet, check_visit};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    last_name: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Owner>>, AppError> {
    let owners = db::list_owners(&state.pool, params.last_name.as_deref()).await?;
    if owners.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(Json(owners))
}

pub async fn page(
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> Result<Json<Page<Owner>>, AppError> {
    let (page, size) = params.resolve()?;
    let owners = db::owners_page(&state.pool, params.last_name.as_deref(), page, size).await?;
    Ok(Json(owners))
}

pub async fn get_one(State(state): State<AppState>, Id(id): Id) -> Result<Json<Owner>, AppError> {
    db::find_owner(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn create(
    State(state): State<AppState>,
    Body(fields): Body<OwnerFields>,
) -> Result<Response, AppError> {
    check_owner(&fields)?;
    let owner = db::insert_owner(&state.pool, &fields).await?;
    Ok(created(format!("/api/owners/{}", owner.id), owner))
}

pub async fn update(
    State(state): State<AppState>,
    Id(id): Id,
    Body(fields): Body<OwnerFields>,
) -> Result<Response, AppError> {
    check_owner(&fields)?;
    no_content_or_404(db::update_owner(&state.pool, id, &fields).await?)
}

pub async fn delete(State(state): State<AppState>, Id(id): Id) -> Result<Response, AppError> {
    deleted(db::delete_owner(&state.pool, id).await?)
}

pub async fn add_pet(
    State(state): State<AppState>,
    Id(owner_id): Id,
    Body(pet): Body<PetInput>,
) -> Result<Response, AppError> {
    check_pet(&pet, false)?;
    if !db::owner_exists(&state.pool, owner_id).await? {
        return Err(AppError::NotFound);
    }
    let pet = db::insert_pet(&state.pool, owner_id, &pet).await?;
    Ok(created(format!("/api/pets/{}", pet.id), pet))
}

/// The pet must belong to the owner, otherwise `404` (Java: `owner.getPet(petId) == null`).
pub async fn get_pet(
    State(state): State<AppState>,
    Ids((owner_id, pet_id)): Ids<(i32, i32)>,
) -> Result<Json<Pet>, AppError> {
    db::find_pet(&state.pool, pet_id)
        .await?
        .filter(|pet| pet.owner_id == owner_id)
        .map(Json)
        .ok_or(AppError::NotFound)
}

/// Java checks that both owner and pet exist but not that the pet belongs to the owner, and
/// answers a bare `204` (`ResponseEntity<Void>`, no content type).
pub async fn update_pet(
    State(state): State<AppState>,
    Ids((owner_id, pet_id)): Ids<(i32, i32)>,
    Body(pet): Body<PetInput>,
) -> Result<Response, AppError> {
    check_pet(&pet, false)?;
    if !db::owner_exists(&state.pool, owner_id).await? {
        return Err(AppError::NotFound);
    }
    if !db::update_pet(&state.pool, pet_id, &pet).await? {
        return Err(AppError::NotFound);
    }
    let mut response = no_content_json();
    response
        .headers_mut()
        .remove(axum::http::header::CONTENT_TYPE);
    Ok(response)
}

/// Java does not check the owner here; a missing pet is a foreign-key violation (`404`
/// `DataIntegrityViolationException`).
pub async fn add_visit(
    State(state): State<AppState>,
    Ids((_owner_id, pet_id)): Ids<(i32, i32)>,
    Body(visit): Body<VisitInput>,
) -> Result<Response, AppError> {
    check_visit(&visit, false)?;
    let visit = db::save_visit(&state.pool, None, Some(pet_id), &visit).await?;
    Ok(created(format!("/api/visits/{}", visit.id), visit))
}
