//! Wire types. Field names follow `src/main/resources/openapi.yml` and the field order follows
//! the DTOs the Java service generates from it, so the JSON is byte-compatible with the Spring
//! Boot service (Jackson writes properties in declaration order and keeps `null`s).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Specialty {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct PetType {
    pub name: Option<String>,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vet {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub specialties: Vec<Specialty>,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Visit {
    pub date: NaiveDate,
    pub description: String,
    pub id: i32,
    pub pet_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pet {
    pub name: String,
    pub birth_date: NaiveDate,
    #[serde(rename = "type")]
    pub pet_type: PetType,
    pub id: i32,
    pub visits: Vec<Visit>,
    pub owner_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub telephone: Option<String>,
    pub pets: Vec<Pet>,
    pub id: i32,
}

/// `OwnerPage` / `PetPage` of the `/api/v2` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub content: Vec<T>,
    pub page: i32,
    pub size: i32,
    pub total_elements: i64,
    pub total_pages: i32,
}

impl<T> Page<T> {
    pub fn new(content: Vec<T>, page: i32, size: i32, total_elements: i64) -> Self {
        let size_i64 = i64::from(size.max(1));
        Self {
            content,
            page,
            size,
            total_elements,
            total_pages: ((total_elements + size_i64 - 1) / size_i64) as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub enabled: Option<bool>,
    pub password: Option<String>,
    pub roles: Vec<Role>,
}

// ---------------------------------------------------------------------------------------
// Request bodies. Every field is optional at the deserialization layer so that a missing
// field is reported as a validation error (`must not be null`) like Bean Validation does,
// instead of a JSON parse error. Strings accept scalars the way Jackson coerces them.
// ---------------------------------------------------------------------------------------

/// `OwnerFields`: body of `POST /owners` and `PUT /owners/{id}`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerFields {
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub first_name: Option<String>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub last_name: Option<String>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub address: Option<String>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub city: Option<String>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub telephone: Option<String>,
}

/// `PetType` / `PetTypeFields` (the `id` is ignored where the contract uses the fields DTO).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetTypeInput {
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "lenient_i32")]
    pub id: Option<i32>,
}

/// `Specialty` (body of `POST/PUT /specialties`, items of `Vet.specialties`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialtyInput {
    #[serde(default, deserialize_with = "lenient_i32")]
    pub id: Option<i32>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub name: Option<String>,
}

/// `Visit` / `VisitFields`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitInput {
    #[serde(default)]
    pub date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "lenient_i32")]
    pub id: Option<i32>,
    #[serde(default, deserialize_with = "lenient_i32")]
    pub pet_id: Option<i32>,
}

/// `Pet` / `PetFields`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetInput {
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub name: Option<String>,
    #[serde(default)]
    pub birth_date: Option<NaiveDate>,
    #[serde(default, rename = "type")]
    pub pet_type: Option<PetTypeInput>,
    #[serde(default, deserialize_with = "lenient_i32")]
    pub id: Option<i32>,
    #[serde(default, deserialize_with = "lenient_i32")]
    pub owner_id: Option<i32>,
    #[serde(default)]
    pub visits: Option<Vec<VisitInput>>,
}

/// `Vet` / `VetFields`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VetInput {
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub first_name: Option<String>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub last_name: Option<String>,
    #[serde(default)]
    pub specialties: Option<Vec<SpecialtyInput>>,
    #[serde(default, deserialize_with = "lenient_i32")]
    pub id: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleInput {
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub name: Option<String>,
}

/// `User`: body of `POST /users`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInput {
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub username: Option<String>,
    #[serde(default, deserialize_with = "scalar_as_string")]
    pub password: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub roles: Option<Vec<RoleInput>>,
}

fn scalar_as_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Number(n) => Ok(Some(n.to_string())),
        serde_json::Value::Bool(b) => Ok(Some(b.to_string())),
        serde_json::Value::Array(_) => Err(D::Error::custom("Cannot coerce Array to String")),
        serde_json::Value::Object(_) => Err(D::Error::custom("Cannot coerce Object to String")),
    }
}

/// Integer ids the way Jackson reads an `Integer`: numbers (floats truncated toward zero) and
/// numeric strings (`" 2"`, `"+1"`; `""` is null) within `i32` range; anything else is unreadable.
fn lenient_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let unreadable = || D::Error::custom("Cannot coerce value to Integer");
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => {
            let value = match n.as_i64() {
                Some(i) => i,
                None => n
                    .as_f64()
                    .map(f64::trunc)
                    .filter(|f| f.abs() < 1e15)
                    .ok_or_else(unreadable)? as i64,
            };
            i32::try_from(value).map(Some).map_err(|_| unreadable())
        }
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let digits = trimmed.strip_prefix(['+', '-']).unwrap_or(trimmed);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return Err(unreadable());
            }
            trimmed.parse::<i32>().map(Some).map_err(|_| unreadable())
        }
        _ => Err(unreadable()),
    }
}

/// Health probe payload, byte-for-byte what Spring Boot Actuator returns for
/// `/actuator/health` in the Java service (`{"groups":["liveness","readiness"],"status":"UP"}`,
/// or `"DOWN"` with HTTP 503 when the database health indicator fails).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub groups: [&'static str; 2],
    pub status: &'static str,
}

impl Health {
    pub const UP: Health = Health {
        groups: ["liveness", "readiness"],
        status: "UP",
    };
    pub const DOWN: Health = Health {
        groups: ["liveness", "readiness"],
        status: "DOWN",
    };
}

impl Owner {
    /// Sort order used by the Java model (`Owner.getPets()`): pet name, case-insensitive.
    pub fn sort_children(&mut self) {
        self.pets
            .sort_by_cached_key(|p| (p.name.to_lowercase(), p.id));
        for pet in &mut self.pets {
            pet.sort_children();
        }
    }
}

impl Pet {
    /// Sort order used by the Java model (`Pet.getVisits()`): most recent visit first.
    pub fn sort_children(&mut self) {
        self.visits
            .sort_by(|a, b| b.date.cmp(&a.date).then(a.id.cmp(&b.id)));
    }
}

impl Vet {
    /// Sort order used by the Java model (`Vet.getSpecialties()`): name, case-insensitive.
    pub fn sort_children(&mut self) {
        self.specialties
            .sort_by_cached_key(|s| (s.name.as_deref().unwrap_or_default().to_lowercase(), s.id));
    }
}
