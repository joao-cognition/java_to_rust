//! Repository layer: hand-written SQL on top of `sqlx`.
//!
//! Every read is a single round trip. Nested collections (owner → pets → visits,
//! vet → specialties) are fetched with LEFT JOINs and folded into DTOs in memory, so an
//! owner lookup costs 1 query instead of the 1 + N + M issued by the eager JPA mappings.

use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};

use crate::dto::{
    Owner, OwnerFields, Page, Pet, PetInput, PetType, Specialty, User, UserInput, Vet, VetInput,
    Visit, VisitInput,
};
use crate::error::AppError;

/// Java's `save(entity)` with an `id` that is not in the table: `merge` fails with an
/// optimistic-locking error that the controller advice reports as a 500.
const STALE_MERGE: &str = "ObjectOptimisticLockingFailureException";

pub async fn connect(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

/// Pool that does not open a connection until first use; lets the router be exercised
/// in tests without a database.
pub fn connect_lazy(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(database_url)
}

// ---------------------------------------------------------------------------------------
// Vets
// ---------------------------------------------------------------------------------------

#[derive(FromRow)]
struct VetRow {
    vet_id: i32,
    first_name: Option<String>,
    last_name: Option<String>,
    specialty_id: Option<i32>,
    specialty_name: Option<String>,
}

const VETS_SQL: &str = "\
SELECT v.id AS vet_id, v.first_name, v.last_name, s.id AS specialty_id, s.name AS specialty_name
FROM vets v
LEFT JOIN vet_specialties vs ON vs.vet_id = v.id
LEFT JOIN specialties s ON s.id = vs.specialty_id";

fn fold_vets(rows: Vec<VetRow>) -> Vec<Vet> {
    let mut vets: Vec<Vet> = Vec::new();
    for row in rows {
        if vets.last().map(|v| v.id) != Some(row.vet_id) {
            vets.push(Vet {
                first_name: row.first_name,
                last_name: row.last_name,
                specialties: Vec::new(),
                id: row.vet_id,
            });
        }
        if let Some(id) = row.specialty_id {
            let vet = vets.last_mut().expect("vet pushed above");
            vet.specialties.push(Specialty {
                id,
                name: row.specialty_name,
            });
        }
    }
    vets.iter_mut().for_each(Vet::sort_children);
    vets
}

pub async fn list_vets(pool: &PgPool) -> Result<Vec<Vet>, sqlx::Error> {
    let rows = sqlx::query_as::<_, VetRow>(&format!("{VETS_SQL} ORDER BY v.id, s.id"))
        .fetch_all(pool)
        .await?;
    Ok(fold_vets(rows))
}

pub async fn find_vet(pool: &PgPool, id: i32) -> Result<Option<Vet>, sqlx::Error> {
    let rows = sqlx::query_as::<_, VetRow>(&format!("{VETS_SQL} WHERE v.id = $1 ORDER BY s.id"))
        .bind(id)
        .fetch_all(pool)
        .await?;
    Ok(fold_vets(rows).pop())
}

// ---------------------------------------------------------------------------------------
// Specialties / pet types (flat reference data)
// ---------------------------------------------------------------------------------------

pub async fn list_specialties(pool: &PgPool) -> Result<Vec<Specialty>, sqlx::Error> {
    sqlx::query_as::<_, Specialty>("SELECT id, name FROM specialties ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn find_specialty(pool: &PgPool, id: i32) -> Result<Option<Specialty>, sqlx::Error> {
    sqlx::query_as::<_, Specialty>("SELECT id, name FROM specialties WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_pet_types(pool: &PgPool) -> Result<Vec<PetType>, sqlx::Error> {
    sqlx::query_as::<_, PetType>("SELECT id, name FROM types ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn find_pet_type(pool: &PgPool, id: i32) -> Result<Option<PetType>, sqlx::Error> {
    sqlx::query_as::<_, PetType>("SELECT id, name FROM types WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

// ---------------------------------------------------------------------------------------
// Owners / pets / visits
// ---------------------------------------------------------------------------------------

#[derive(FromRow)]
struct OwnerRow {
    owner_id: i32,
    first_name: Option<String>,
    last_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    telephone: Option<String>,
    pet_id: Option<i32>,
    pet_name: Option<String>,
    birth_date: Option<NaiveDate>,
    type_id: Option<i32>,
    type_name: Option<String>,
    visit_id: Option<i32>,
    visit_date: Option<NaiveDate>,
    visit_description: Option<String>,
}

const OWNERS_SQL: &str = "\
SELECT o.id AS owner_id, o.first_name, o.last_name, o.address, o.city, o.telephone,
       p.id AS pet_id, p.name AS pet_name, p.birth_date, t.id AS type_id, t.name AS type_name,
       vi.id AS visit_id, vi.visit_date, vi.description AS visit_description
FROM owners o
LEFT JOIN pets p ON p.owner_id = o.id
LEFT JOIN types t ON t.id = p.type_id
LEFT JOIN visits vi ON vi.pet_id = p.id";

fn fold_owners(rows: Vec<OwnerRow>) -> Vec<Owner> {
    let mut owners: Vec<Owner> = Vec::new();
    for row in rows {
        if owners.last().map(|o| o.id) != Some(row.owner_id) {
            owners.push(Owner {
                first_name: row.first_name,
                last_name: row.last_name,
                address: row.address,
                city: row.city,
                telephone: row.telephone,
                id: row.owner_id,
                pets: Vec::new(),
            });
        }
        let Some(pet_id) = row.pet_id else { continue };
        let owner = owners.last_mut().expect("owner pushed above");
        if owner.pets.last().map(|p| p.id) != Some(pet_id) {
            owner.pets.push(Pet {
                name: row.pet_name.unwrap_or_default(),
                birth_date: row.birth_date.unwrap_or_default(),
                pet_type: PetType {
                    name: row.type_name,
                    id: row.type_id.unwrap_or_default(),
                },
                id: pet_id,
                owner_id: row.owner_id,
                visits: Vec::new(),
            });
        }
        if let Some(visit_id) = row.visit_id {
            let pet = owner.pets.last_mut().expect("pet pushed above");
            pet.visits.push(Visit {
                date: row.visit_date.unwrap_or_default(),
                description: row.visit_description.unwrap_or_default(),
                id: visit_id,
                pet_id,
            });
        }
    }
    owners.iter_mut().for_each(Owner::sort_children);
    owners
}

const OWNER_ORDER: &str = "ORDER BY o.id, p.id, vi.id";

pub async fn list_owners(
    pool: &PgPool,
    last_name: Option<&str>,
) -> Result<Vec<Owner>, sqlx::Error> {
    let rows = match last_name {
        // Prefix match, same as the JPQL `owner.lastName LIKE :lastName%` in Java.
        Some(prefix) => {
            sqlx::query_as::<_, OwnerRow>(&format!(
                "{OWNERS_SQL} WHERE o.last_name LIKE $1 || '%' {OWNER_ORDER}"
            ))
            .bind(prefix)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, OwnerRow>(&format!("{OWNERS_SQL} {OWNER_ORDER}"))
                .fetch_all(pool)
                .await?
        }
    };
    Ok(fold_owners(rows))
}

pub async fn find_owner(pool: &PgPool, id: i32) -> Result<Option<Owner>, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, OwnerRow>(&format!("{OWNERS_SQL} WHERE o.id = $1 {OWNER_ORDER}"))
            .bind(id)
            .fetch_all(pool)
            .await?;
    Ok(fold_owners(rows).pop())
}

pub async fn insert_owner(pool: &PgPool, fields: &OwnerFields) -> Result<Owner, sqlx::Error> {
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO owners (first_name, last_name, address, city, telephone) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(&fields.first_name)
    .bind(&fields.last_name)
    .bind(&fields.address)
    .bind(&fields.city)
    .bind(&fields.telephone)
    .fetch_one(pool)
    .await?;
    Ok(Owner {
        first_name: fields.first_name.clone(),
        last_name: fields.last_name.clone(),
        address: fields.address.clone(),
        city: fields.city.clone(),
        telephone: fields.telephone.clone(),
        id,
        pets: Vec::new(),
    })
}

pub async fn owners_page(
    pool: &PgPool,
    last_name: Option<&str>,
    page: i32,
    size: i32,
) -> Result<Page<Owner>, sqlx::Error> {
    let prefix = last_name.unwrap_or("");
    let total: i64 =
        sqlx::query_scalar("SELECT count(*) FROM owners WHERE last_name LIKE $1 || '%'")
            .bind(prefix)
            .fetch_one(pool)
            .await?;
    let rows = sqlx::query_as::<_, OwnerRow>(&format!(
        "{OWNERS_SQL} WHERE o.id IN (SELECT id FROM owners WHERE last_name LIKE $1 || '%' \
         ORDER BY id LIMIT $2 OFFSET $3) {OWNER_ORDER}"
    ))
    .bind(prefix)
    .bind(i64::from(size))
    .bind(i64::from(page) * i64::from(size))
    .fetch_all(pool)
    .await?;
    Ok(Page::new(fold_owners(rows), page, size, total))
}

pub async fn update_owner(
    pool: &PgPool,
    id: i32,
    fields: &OwnerFields,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE owners SET first_name = $2, last_name = $3, address = $4, city = $5, \
         telephone = $6 WHERE id = $1",
    )
    .bind(id)
    .bind(&fields.first_name)
    .bind(&fields.last_name)
    .bind(&fields.address)
    .bind(&fields.city)
    .bind(&fields.telephone)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Deletes an owner. Java does not cascade to pets here: an owner that still has pets is a
/// foreign-key violation (`404 DataIntegrityViolationException`).
pub async fn delete_owner(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    delete_by_id(pool, "DELETE FROM owners WHERE id = $1", id).await
}

pub async fn owner_exists(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    exists(
        pool,
        "SELECT EXISTS (SELECT 1 FROM owners WHERE id = $1)",
        id,
    )
    .await
}

async fn exists(pool: &PgPool, sql: &str, id: i32) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(sql).bind(id).fetch_one(pool).await
}

// ---------------------------------------------------------------------------------------
// Pets
// ---------------------------------------------------------------------------------------

#[derive(FromRow)]
struct PetRow {
    pet_id: i32,
    pet_name: String,
    birth_date: NaiveDate,
    owner_id: Option<i32>,
    type_id: i32,
    type_name: Option<String>,
    visit_id: Option<i32>,
    visit_date: Option<NaiveDate>,
    visit_description: Option<String>,
}

const PETS_SQL: &str = "\
SELECT p.id AS pet_id, p.name AS pet_name, p.birth_date, p.owner_id,
       t.id AS type_id, t.name AS type_name,
       vi.id AS visit_id, vi.visit_date, vi.description AS visit_description
FROM pets p
JOIN types t ON t.id = p.type_id
LEFT JOIN visits vi ON vi.pet_id = p.id";

const PET_ORDER: &str = "ORDER BY p.id, vi.id";

fn fold_pets(rows: Vec<PetRow>) -> Vec<Pet> {
    let mut pets: Vec<Pet> = Vec::new();
    for row in rows {
        if pets.last().map(|p| p.id) != Some(row.pet_id) {
            pets.push(Pet {
                name: row.pet_name,
                birth_date: row.birth_date,
                pet_type: PetType {
                    name: row.type_name,
                    id: row.type_id,
                },
                id: row.pet_id,
                visits: Vec::new(),
                owner_id: row.owner_id.unwrap_or_default(),
            });
        }
        if let Some(visit_id) = row.visit_id {
            let pet = pets.last_mut().expect("pet pushed above");
            pet.visits.push(Visit {
                date: row.visit_date.unwrap_or_default(),
                description: row.visit_description.unwrap_or_default(),
                id: visit_id,
                pet_id: row.pet_id,
            });
        }
    }
    pets.iter_mut().for_each(Pet::sort_children);
    pets
}

pub async fn find_pet(pool: &PgPool, id: i32) -> Result<Option<Pet>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PetRow>(&format!("{PETS_SQL} WHERE p.id = $1 {PET_ORDER}"))
        .bind(id)
        .fetch_all(pool)
        .await?;
    Ok(fold_pets(rows).pop())
}

pub async fn list_pets(pool: &PgPool) -> Result<Vec<Pet>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PetRow>(&format!("{PETS_SQL} {PET_ORDER}"))
        .fetch_all(pool)
        .await?;
    Ok(fold_pets(rows))
}

pub async fn pets_page(pool: &PgPool, page: i32, size: i32) -> Result<Page<Pet>, sqlx::Error> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM pets")
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query_as::<_, PetRow>(&format!(
        "{PETS_SQL} WHERE p.id IN (SELECT id FROM pets ORDER BY id LIMIT $1 OFFSET $2) {PET_ORDER}"
    ))
    .bind(i64::from(size))
    .bind(i64::from(page) * i64::from(size))
    .fetch_all(pool)
    .await?;
    Ok(Page::new(fold_pets(rows), page, size, total))
}

/// Inserts a pet for an owner. A missing pet type surfaces as a foreign-key violation, which
/// maps to Java's `DataIntegrityViolationException` response.
pub async fn insert_pet(pool: &PgPool, owner_id: i32, pet: &PetInput) -> Result<Pet, AppError> {
    let type_id = pet.pet_type.as_ref().and_then(|t| t.id);
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO pets (name, birth_date, type_id, owner_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&pet.name)
    .bind(pet.birth_date)
    .bind(type_id)
    .bind(owner_id)
    .fetch_one(pool)
    .await?;
    find_pet(pool, id).await?.ok_or(AppError::NotFound)
}

/// Updates name, birth date and type of a pet (the owner and visits are left untouched).
pub async fn update_pet(pool: &PgPool, id: i32, pet: &PetInput) -> Result<bool, AppError> {
    let type_id = pet.pet_type.as_ref().and_then(|t| t.id);
    let result =
        sqlx::query("UPDATE pets SET name = $2, birth_date = $3, type_id = $4 WHERE id = $1")
            .bind(id)
            .bind(&pet.name)
            .bind(pet.birth_date)
            .bind(type_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_pet(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM visits WHERE pet_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let deleted = sqlx::query("DELETE FROM pets WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    tx.commit().await?;
    Ok(deleted > 0)
}

// ---------------------------------------------------------------------------------------
// Visits
// ---------------------------------------------------------------------------------------

const VISITS_SQL: &str = "SELECT visit_date AS date, description, id, pet_id FROM visits";

pub async fn list_visits(pool: &PgPool) -> Result<Vec<Visit>, sqlx::Error> {
    sqlx::query_as::<_, Visit>(&format!("{VISITS_SQL} ORDER BY id"))
        .fetch_all(pool)
        .await
}

pub async fn find_visit(pool: &PgPool, id: i32) -> Result<Option<Visit>, sqlx::Error> {
    sqlx::query_as::<_, Visit>(&format!("{VISITS_SQL} WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// `visitService.saveVisit`: inserts, or merges into an existing row when the body carries
/// an `id` (Java's `save` → `merge`).
pub async fn save_visit(
    pool: &PgPool,
    id: Option<i32>,
    pet_id: Option<i32>,
    visit: &VisitInput,
) -> Result<Visit, AppError> {
    let saved = match id {
        Some(id) => sqlx::query_as::<_, Visit>(
            "UPDATE visits SET visit_date = $2, description = $3, pet_id = $4 WHERE id = $1 \
             RETURNING visit_date AS date, description, id, pet_id",
        )
        .bind(id)
        .bind(visit.date)
        .bind(&visit.description)
        .bind(pet_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::unexpected(STALE_MERGE))?,
        None => {
            sqlx::query_as::<_, Visit>(
                "INSERT INTO visits (visit_date, description, pet_id) VALUES ($1, $2, $3) \
             RETURNING visit_date AS date, description, id, pet_id",
            )
            .bind(visit.date)
            .bind(&visit.description)
            .bind(pet_id)
            .fetch_one(pool)
            .await?
        }
    };
    Ok(saved)
}

pub async fn update_visit(pool: &PgPool, id: i32, visit: &VisitInput) -> Result<bool, AppError> {
    let result = sqlx::query("UPDATE visits SET visit_date = $2, description = $3 WHERE id = $1")
        .bind(id)
        .bind(visit.date)
        .bind(&visit.description)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_visit(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    delete_by_id(pool, "DELETE FROM visits WHERE id = $1", id).await
}

async fn delete_by_id(pool: &PgPool, sql: &str, id: i32) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query(sql)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        > 0)
}

// ---------------------------------------------------------------------------------------
// Pet types and specialties
// ---------------------------------------------------------------------------------------

pub async fn insert_pet_type(pool: &PgPool, name: &str) -> Result<PetType, sqlx::Error> {
    sqlx::query_as::<_, PetType>("INSERT INTO types (name) VALUES ($1) RETURNING name, id")
        .bind(name)
        .fetch_one(pool)
        .await
}

pub async fn update_pet_type(pool: &PgPool, id: i32, name: &str) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("UPDATE types SET name = $2 WHERE id = $1")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await?
        .rows_affected()
        > 0)
}

/// Deletes a pet type together with the pets of that type and their visits, like the Java
/// repository does before removing the type.
pub async fn delete_pet_type(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM visits WHERE pet_id IN (SELECT id FROM pets WHERE type_id = $1)")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM pets WHERE type_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let deleted = sqlx::query("DELETE FROM types WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    tx.commit().await?;
    Ok(deleted > 0)
}

/// Inserts a specialty, or merges into an existing one when an `id` is given.
pub async fn save_specialty(
    pool: &PgPool,
    id: Option<i32>,
    name: &str,
) -> Result<Specialty, AppError> {
    let saved = match id {
        Some(id) => sqlx::query_as::<_, Specialty>(
            "UPDATE specialties SET name = $2 WHERE id = $1 RETURNING id, name",
        )
        .bind(id)
        .bind(name)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::unexpected(STALE_MERGE))?,
        None => {
            sqlx::query_as::<_, Specialty>(
                "INSERT INTO specialties (name) VALUES ($1) RETURNING id, name",
            )
            .bind(name)
            .fetch_one(pool)
            .await?
        }
    };
    Ok(saved)
}

/// Deletes a specialty. Like Java, a specialty still assigned to a vet is a foreign-key
/// violation (`404 DataIntegrityViolationException`).
pub async fn delete_specialty(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    delete_by_id(pool, "DELETE FROM specialties WHERE id = $1", id).await
}

// ---------------------------------------------------------------------------------------
// Vets (writes)
// ---------------------------------------------------------------------------------------

/// Inserts (or, with an `id`, merges) a vet. Specialties are resolved by name and unknown
/// names are dropped, as `specialtyService.findSpecialtiesByNameIn` does in Java.
pub async fn save_vet(pool: &PgPool, id: Option<i32>, vet: &VetInput) -> Result<Vet, AppError> {
    let mut tx = pool.begin().await?;
    let vet_id: i32 = match id {
        Some(id) => sqlx::query_scalar(
            "UPDATE vets SET first_name = $2, last_name = $3 WHERE id = $1 RETURNING id",
        )
        .bind(id)
        .bind(&vet.first_name)
        .bind(&vet.last_name)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::unexpected(STALE_MERGE))?,
        None => {
            sqlx::query_scalar(
                "INSERT INTO vets (first_name, last_name) VALUES ($1, $2) RETURNING id",
            )
            .bind(&vet.first_name)
            .bind(&vet.last_name)
            .fetch_one(&mut *tx)
            .await?
        }
    };
    replace_vet_specialties(&mut tx, vet_id, vet).await?;
    tx.commit().await?;
    find_vet(pool, vet_id).await?.ok_or(AppError::NotFound)
}

/// Updates a vet's names and specialty set. Returns `false` when the vet does not exist.
pub async fn update_vet(pool: &PgPool, id: i32, vet: &VetInput) -> Result<bool, AppError> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query("UPDATE vets SET first_name = $2, last_name = $3 WHERE id = $1")
        .bind(id)
        .bind(&vet.first_name)
        .bind(&vet.last_name)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if updated == 0 {
        return Ok(false);
    }
    replace_vet_specialties(&mut tx, id, vet).await?;
    tx.commit().await?;
    Ok(true)
}

async fn replace_vet_specialties(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    vet_id: i32,
    vet: &VetInput,
) -> Result<(), sqlx::Error> {
    let names: Vec<String> = vet
        .specialties
        .iter()
        .flatten()
        .filter_map(|s| s.name.clone())
        .collect();
    sqlx::query("DELETE FROM vet_specialties WHERE vet_id = $1")
        .bind(vet_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "INSERT INTO vet_specialties (vet_id, specialty_id) \
         SELECT $1, id FROM specialties WHERE name = ANY($2)",
    )
    .bind(vet_id)
    .bind(&names)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_vet(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM vet_specialties WHERE vet_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let deleted = sqlx::query("DELETE FROM vets WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    tx.commit().await?;
    Ok(deleted > 0)
}

// ---------------------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------------------

/// Inserts a user and its roles in one transaction. An existing username is a unique
/// violation, reported like Java's `DataIntegrityViolationException`.
pub async fn insert_user(
    pool: &PgPool,
    user: &UserInput,
    roles: &[String],
) -> Result<User, AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO users (username, password, enabled) VALUES ($1, $2, $3)")
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.enabled)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO roles (username, role) SELECT $1, unnest($2::varchar[])")
        .bind(&user.username)
        .bind(roles)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(User {
        username: user.username.clone().unwrap_or_default(),
        enabled: user.enabled,
        password: user.password.clone(),
        roles: roles
            .iter()
            .map(|name| crate::dto::Role { name: name.clone() })
            .collect(),
    })
}
