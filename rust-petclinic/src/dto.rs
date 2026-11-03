//! Wire types. Field names, nesting and ordering follow `src/main/resources/openapi.yml`
//! (the contract the Java controllers generate their DTOs from) so that responses are
//! interchangeable with the Spring Boot service.

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub owner_id: i32,
    pub visits: Vec<Visit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub telephone: Option<String>,
    pub id: i32,
    pub pets: Vec<Pet>,
}

/// Request body of `POST /api/owners` (`OwnerFields` in the OpenAPI contract).
///
/// Every field is optional at the deserialization layer so that a missing field is reported
/// as a validation error (`must not be null`), exactly like Bean Validation does in Java,
/// instead of a generic JSON parse error. Scalars are coerced to strings the way Jackson
/// does (`{"firstName": 1}` is validated as `"1"`); objects and arrays are a parse error.
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
        other => Err(D::Error::custom(format!(
            "Cannot coerce {} to String",
            match other {
                serde_json::Value::Array(_) => "Array",
                _ => "Object",
            }
        ))),
    }
}

/// Health probe payload, byte-for-byte what Spring Boot Actuator returns for
/// `/actuator/health` in the Java service (`{"groups":["liveness","readiness"],"status":"UP"}`).
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
