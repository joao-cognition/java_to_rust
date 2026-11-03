//! Bean Validation rules of the generated Java DTOs (`@NotNull`, `@Size`, `@Pattern`, `@Min`,
//! `@PetAgeValidation`), reproduced with the same messages so `schemaValidationErrors` match.

use std::sync::LazyLock;

use chrono::{Months, NaiveDate, Utc};
use regex::Regex;

use crate::dto::{
    OwnerFields, PetInput, PetTypeInput, SpecialtyInput, UserInput, VetInput, VisitInput,
};
use crate::error::{AppError, ValidationMessage};

const NAME_PATTERN: &str = r"^[\p{L}]+([ '-][\p{L}]+){0,2}$";
const LAST_NAME_PATTERN: &str = r"^[\p{L}]+([ '-][\p{L}]+){0,2}\.?$";
const TELEPHONE_PATTERN: &str = r"^[0-9]*$";

static NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(NAME_PATTERN).expect("valid regex"));
static LAST_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(LAST_NAME_PATTERN).expect("valid regex"));
static TELEPHONE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(TELEPHONE_PATTERN).expect("valid regex"));
/// `Owner.telephone` entity constraint (`@Pattern(regexp = "\\d{10}")`), checked by Hibernate
/// at flush time, i.e. after the DTO validation passed.
static ENTITY_TELEPHONE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]{10}$").expect("valid regex"));

type Pattern = (&'static LazyLock<Regex>, &'static str);

/// Collects field errors in declaration order.
#[derive(Debug, Default)]
pub struct Validator {
    errors: Vec<ValidationMessage>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, field: &str, rejected: Option<&str>, message: &str) {
        self.errors
            .push(ValidationMessage::new(field, rejected, message));
    }

    /// `@NotNull`.
    pub fn not_null<T>(&mut self, field: &str, value: &Option<T>) -> bool {
        if value.is_none() {
            self.push(field, None, "must not be null");
            false
        } else {
            true
        }
    }

    /// Optional `@NotNull`, `@Size(min, max)` (UTF-16 length, like `String#length`) and `@Pattern`.
    pub fn text(
        &mut self,
        field: &str,
        value: Option<&str>,
        required: bool,
        max: usize,
        pattern: Option<Pattern>,
    ) {
        let Some(value) = value else {
            if required {
                self.push(field, None, "must not be null");
            }
            return;
        };
        let len = value.encode_utf16().count();
        if len < 1 || len > max {
            self.push(
                field,
                Some(value),
                &format!("size must be between 1 and {max}"),
            );
        }
        if let Some((re, source)) = pattern {
            if !re.is_match(value) {
                self.push(field, Some(value), &format!("must match \"{source}\""));
            }
        }
    }

    /// `@Min(0)`.
    pub fn non_negative(&mut self, field: &str, value: Option<i32>) {
        if let Some(v) = value.filter(|v| *v < 0) {
            self.push(
                field,
                Some(&v.to_string()),
                "must be greater than or equal to 0",
            );
        }
    }

    /// `@PetAgeValidation`.
    pub fn pet_age(&mut self, field: &str, value: Option<NaiveDate>) {
        let Some(date) = value else { return };
        let today = Utc::now().date_naive();
        let rejected = date.to_string();
        if date > today {
            self.push(field, Some(&rejected), "Birth date cannot be in the future");
        } else if today
            .checked_sub_months(Months::new(600))
            .is_some_and(|limit| date < limit)
        {
            self.push(
                field,
                Some(&rejected),
                "Birth date cannot be older than 50 years",
            );
        }
    }

    pub fn finish(self) -> Result<(), AppError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(self.errors))
        }
    }

    pub fn into_errors(self) -> Vec<ValidationMessage> {
        self.errors
    }
}

pub fn validate_owner_fields(fields: &OwnerFields) -> Vec<ValidationMessage> {
    let mut v = Validator::new();
    v.text(
        "firstName",
        fields.first_name.as_deref(),
        true,
        30,
        Some((&NAME_RE, NAME_PATTERN)),
    );
    v.text(
        "lastName",
        fields.last_name.as_deref(),
        true,
        30,
        Some((&LAST_NAME_RE, LAST_NAME_PATTERN)),
    );
    v.text("address", fields.address.as_deref(), true, 255, None);
    v.text("city", fields.city.as_deref(), true, 80, None);
    v.text(
        "telephone",
        fields.telephone.as_deref(),
        true,
        20,
        Some((&TELEPHONE_RE, TELEPHONE_PATTERN)),
    );
    v.into_errors()
}

/// `OwnerFields` validation followed by the `Owner` entity constraint Hibernate enforces on
/// persist, which Java reports as a `500 ConstraintViolationException`.
pub fn check_owner(fields: &OwnerFields) -> Result<(), AppError> {
    let errors = validate_owner_fields(fields);
    if !errors.is_empty() {
        return Err(AppError::Validation(errors));
    }
    if !fields
        .telephone
        .as_deref()
        .is_some_and(|t| ENTITY_TELEPHONE_RE.is_match(t))
    {
        return Err(AppError::unexpected("ConstraintViolationException"));
    }
    Ok(())
}

fn pet_type_rules(v: &mut Validator, prefix: &str, t: &PetTypeInput, id_required: bool) {
    v.text(&format!("{prefix}name"), t.name.as_deref(), true, 80, None);
    let id_field = format!("{prefix}id");
    if id_required {
        v.not_null(&id_field, &t.id);
    }
    v.non_negative(&id_field, t.id);
}

/// `PetTypeFields` (`POST /pettypes`) or `PetType` (`PUT /pettypes/{id}`, `id` required).
pub fn check_pet_type(t: &PetTypeInput, id_required: bool) -> Result<(), AppError> {
    let mut v = Validator::new();
    v.text("name", t.name.as_deref(), true, 80, None);
    if id_required {
        v.not_null("id", &t.id);
        v.non_negative("id", t.id);
    }
    v.finish()
}

fn specialty_rules(v: &mut Validator, prefix: &str, s: &SpecialtyInput) {
    v.non_negative(&format!("{prefix}id"), s.id);
    v.text(&format!("{prefix}name"), s.name.as_deref(), true, 80, None);
}

pub fn check_specialty(s: &SpecialtyInput) -> Result<(), AppError> {
    let mut v = Validator::new();
    specialty_rules(&mut v, "", s);
    v.finish()
}

fn visit_rules(v: &mut Validator, prefix: &str, visit: &VisitInput, full: bool) {
    v.not_null(&format!("{prefix}date"), &visit.date);
    v.text(
        &format!("{prefix}description"),
        visit.description.as_deref(),
        true,
        255,
        None,
    );
    if full {
        v.non_negative(&format!("{prefix}id"), visit.id);
        let pet_id = format!("{prefix}petId");
        v.not_null(&pet_id, &visit.pet_id);
        v.non_negative(&pet_id, visit.pet_id);
    }
}

/// `VisitFields` (`full == false`) or `Visit` (`full == true`).
pub fn check_visit(visit: &VisitInput, full: bool) -> Result<(), AppError> {
    let mut v = Validator::new();
    visit_rules(&mut v, "", visit, full);
    v.finish()
}

/// `PetFields` (`full == false`) or `Pet` (`full == true`, also validates `id`, `ownerId`
/// and each visit).
pub fn check_pet(pet: &PetInput, full: bool) -> Result<(), AppError> {
    let mut v = Validator::new();
    v.text("name", pet.name.as_deref(), true, 30, None);
    v.not_null("birthDate", &pet.birth_date);
    v.pet_age("birthDate", pet.birth_date);
    if v.not_null("type", &pet.pet_type) {
        if let Some(t) = &pet.pet_type {
            pet_type_rules(&mut v, "type.", t, true);
        }
    }
    if full {
        v.non_negative("id", pet.id);
        v.non_negative("ownerId", pet.owner_id);
        for (i, visit) in pet.visits.iter().flatten().enumerate() {
            visit_rules(&mut v, &format!("visits[{i}]."), visit, true);
        }
    }
    v.finish()
}

/// `VetFields` / `Vet`.
pub fn check_vet(vet: &VetInput) -> Result<(), AppError> {
    let mut v = Validator::new();
    v.text(
        "firstName",
        vet.first_name.as_deref(),
        true,
        30,
        Some((&NAME_RE, NAME_PATTERN)),
    );
    v.text(
        "lastName",
        vet.last_name.as_deref(),
        true,
        30,
        Some((&LAST_NAME_RE, LAST_NAME_PATTERN)),
    );
    v.not_null("specialties", &vet.specialties);
    for (i, s) in vet.specialties.iter().flatten().enumerate() {
        specialty_rules(&mut v, &format!("specialties[{i}]."), s);
    }
    v.non_negative("id", vet.id);
    v.finish()
}

pub fn check_user(user: &UserInput) -> Result<(), AppError> {
    let mut v = Validator::new();
    v.text("username", user.username.as_deref(), true, 80, None);
    v.text("password", user.password.as_deref(), false, 80, None);
    for (i, role) in user.roles.iter().flatten().enumerate() {
        v.text(
            &format!("roles[{i}].name"),
            role.name.as_deref(),
            true,
            80,
            None,
        );
    }
    v.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> OwnerFields {
        OwnerFields {
            first_name: Some("George".into()),
            last_name: Some("Franklin".into()),
            address: Some("110 W. Liberty St.".into()),
            city: Some("Madison".into()),
            telephone: Some("6085551023".into()),
        }
    }

    #[test]
    fn accepts_valid_owner() {
        assert!(validate_owner_fields(&valid()).is_empty());
    }

    #[test]
    fn accepts_compound_and_accented_names() {
        let mut f = valid();
        f.first_name = Some("Jean-Luc".into());
        f.last_name = Some("O'Neil Jr.".into());
        assert!(validate_owner_fields(&f).is_empty());
    }

    #[test]
    fn reports_missing_fields_as_not_null() {
        let errors = validate_owner_fields(&OwnerFields::default());
        assert_eq!(errors.len(), 5);
        assert_eq!(
            errors[0].message,
            "Field 'firstName' must not be null (rejected value: null)"
        );
        assert_eq!(errors[0].rejected_value, "null");
    }

    #[test]
    fn reports_pattern_and_size_violations() {
        let mut f = valid();
        f.telephone = Some("call-me".into());
        f.city = Some(String::new());
        let errors = validate_owner_fields(&f);
        let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        assert_eq!(
            messages,
            vec![
                "Field 'city' size must be between 1 and 80 (rejected value: )",
                "Field 'telephone' must match \"^[0-9]*$\" (rejected value: call-me)",
            ]
        );
    }

    #[test]
    fn size_counts_utf16_code_units_like_java() {
        // 40 emoji are 40 chars but 80 UTF-16 code units: exactly the @Size(max = 80) of city
        let mut f = valid();
        f.city = Some("\u{1F436}".repeat(40));
        assert!(validate_owner_fields(&f).is_empty());
        f.city = Some("\u{1F436}".repeat(41));
        let errors = validate_owner_fields(&f);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].default_message, "size must be between 1 and 80");
    }
}
