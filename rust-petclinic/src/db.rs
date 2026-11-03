//! Repository layer: hand-written SQL on top of `sqlx`.
//!
//! Every read is a single round trip. Nested collections (owner → pets → visits,
//! vet → specialties) are fetched with LEFT JOINs and folded into DTOs in memory, so an
//! owner lookup costs 1 query instead of the 1 + N + M issued by the eager JPA mappings.

use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};

use crate::dto::{Owner, OwnerFields, Pet, PetType, Specialty, Vet, Visit};

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

#[derive(FromRow)]
struct PetRow {
    pet_id: i32,
    pet_name: String,
    birth_date: NaiveDate,
    owner_id: i32,
    type_id: i32,
    type_name: Option<String>,
    visit_id: Option<i32>,
    visit_date: Option<NaiveDate>,
    visit_description: Option<String>,
}

pub async fn find_pet(pool: &PgPool, id: i32) -> Result<Option<Pet>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PetRow>(
        "SELECT p.id AS pet_id, p.name AS pet_name, p.birth_date, p.owner_id,
                t.id AS type_id, t.name AS type_name,
                vi.id AS visit_id, vi.visit_date, vi.description AS visit_description
         FROM pets p
         JOIN types t ON t.id = p.type_id
         LEFT JOIN visits vi ON vi.pet_id = p.id
         WHERE p.id = $1
         ORDER BY vi.id",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let mut pet: Option<Pet> = None;
    for row in rows {
        let current = pet.get_or_insert_with(|| Pet {
            name: row.pet_name.clone(),
            birth_date: row.birth_date,
            pet_type: PetType {
                name: row.type_name.clone(),
                id: row.type_id,
            },
            id: row.pet_id,
            owner_id: row.owner_id,
            visits: Vec::new(),
        });
        if let Some(visit_id) = row.visit_id {
            current.visits.push(Visit {
                date: row.visit_date.unwrap_or_default(),
                description: row.visit_description.unwrap_or_default(),
                id: visit_id,
                pet_id: row.pet_id,
            });
        }
    }
    if let Some(p) = pet.as_mut() {
        p.sort_children();
    }
    Ok(pet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_vet_rows_and_sorts_specialties_by_name() {
        let rows = vec![
            VetRow {
                vet_id: 3,
                first_name: Some("Linda".into()),
                last_name: Some("Douglas".into()),
                specialty_id: Some(2),
                specialty_name: Some("surgery".into()),
            },
            VetRow {
                vet_id: 3,
                first_name: Some("Linda".into()),
                last_name: Some("Douglas".into()),
                specialty_id: Some(3),
                specialty_name: Some("dentistry".into()),
            },
            VetRow {
                vet_id: 6,
                first_name: Some("Sharon".into()),
                last_name: Some("Jenkins".into()),
                specialty_id: None,
                specialty_name: None,
            },
        ];
        let vets = fold_vets(rows);
        assert_eq!(vets.len(), 2);
        let names: Vec<_> = vets[0]
            .specialties
            .iter()
            .map(|s| s.name.clone().unwrap())
            .collect();
        assert_eq!(names, vec!["dentistry", "surgery"]);
        assert!(vets[1].specialties.is_empty());
    }

    #[test]
    fn folds_owner_rows_into_pets_and_visits() {
        let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        let base = |pet_id, pet_name: &str, visit_id, visit_date| OwnerRow {
            owner_id: 6,
            first_name: Some("Jean".into()),
            last_name: Some("Coleman".into()),
            address: Some("105 N. Lake St.".into()),
            city: Some("Monona".into()),
            telephone: Some("6085552654".into()),
            pet_id: Some(pet_id),
            pet_name: Some(pet_name.into()),
            birth_date: Some(d("1995-09-04")),
            type_id: Some(1),
            type_name: Some("cat".into()),
            visit_id,
            visit_date,
            visit_description: visit_id.map(|_| "checkup".to_string()),
        };
        let rows = vec![
            base(7, "Samantha", Some(1), Some(d("2010-03-04"))),
            base(7, "Samantha", Some(4), Some(d("2008-09-04"))),
            base(8, "Max", Some(2), Some(d("2011-03-04"))),
            base(8, "Max", Some(3), Some(d("2009-06-04"))),
        ];
        let owners = fold_owners(rows);
        assert_eq!(owners.len(), 1);
        let owner = &owners[0];
        // pets sorted by name (case-insensitive): Max before Samantha
        assert_eq!(owner.pets[0].name, "Max");
        assert_eq!(owner.pets[1].name, "Samantha");
        // visits sorted most recent first
        assert_eq!(owner.pets[0].visits[0].id, 2);
        assert_eq!(owner.pets[0].visits[1].id, 3);
        assert_eq!(owner.pets[1].visits[0].id, 1);
        assert_eq!(owner.pets[1].visits[1].id, 4);
    }

    #[test]
    fn owner_without_pets_has_empty_list() {
        let rows = vec![OwnerRow {
            owner_id: 11,
            first_name: Some("New".into()),
            last_name: Some("Owner".into()),
            address: None,
            city: None,
            telephone: None,
            pet_id: None,
            pet_name: None,
            birth_date: None,
            type_id: None,
            type_name: None,
            visit_id: None,
            visit_date: None,
            visit_description: None,
        }];
        let owners = fold_owners(rows);
        assert_eq!(owners.len(), 1);
        assert!(owners[0].pets.is_empty());
    }
}
