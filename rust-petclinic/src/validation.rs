//! Bean-Validation-equivalent checks for request bodies.
//!
//! Constraints come from `OwnerFields` in `openapi.yml`; messages mirror the Hibernate
//! Validator defaults so clients see the same `schemaValidationErrors` entries as with Java.

use std::sync::LazyLock;

use regex::Regex;

use crate::dto::OwnerFields;
use crate::error::ValidationMessage;

const NAME_PATTERN: &str = r"^[\p{L}]+([ '-][\p{L}]+){0,2}$";
const LAST_NAME_PATTERN: &str = r"^[\p{L}]+([ '-][\p{L}]+){0,2}\.?$";
const TELEPHONE_PATTERN: &str = r"^[0-9]*$";

static NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(NAME_PATTERN).expect("valid regex"));
static LAST_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(LAST_NAME_PATTERN).expect("valid regex"));
static TELEPHONE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(TELEPHONE_PATTERN).expect("valid regex"));

struct FieldRule {
    field: &'static str,
    max_len: usize,
    pattern: Option<(&'static LazyLock<Regex>, &'static str)>,
}

static RULES: [FieldRule; 5] = [
    FieldRule {
        field: "firstName",
        max_len: 30,
        pattern: Some((&NAME_RE, NAME_PATTERN)),
    },
    FieldRule {
        field: "lastName",
        max_len: 30,
        pattern: Some((&LAST_NAME_RE, LAST_NAME_PATTERN)),
    },
    FieldRule {
        field: "address",
        max_len: 255,
        pattern: None,
    },
    FieldRule {
        field: "city",
        max_len: 80,
        pattern: None,
    },
    FieldRule {
        field: "telephone",
        max_len: 20,
        pattern: Some((&TELEPHONE_RE, TELEPHONE_PATTERN)),
    },
];

fn value_of<'a>(fields: &'a OwnerFields, name: &str) -> Option<&'a str> {
    match name {
        "firstName" => fields.first_name.as_deref(),
        "lastName" => fields.last_name.as_deref(),
        "address" => fields.address.as_deref(),
        "city" => fields.city.as_deref(),
        "telephone" => fields.telephone.as_deref(),
        _ => None,
    }
}

/// Validates `OwnerFields`; returns one message per violated constraint, in field order.
pub fn validate_owner_fields(fields: &OwnerFields) -> Vec<ValidationMessage> {
    let mut errors = Vec::new();
    for rule in RULES.iter() {
        match value_of(fields, rule.field) {
            None => errors.push(ValidationMessage::new(rule.field, None, "must not be null")),
            Some(value) => {
                let len = value.chars().count();
                if len < 1 || len > rule.max_len {
                    errors.push(ValidationMessage::new(
                        rule.field,
                        Some(value),
                        &format!("size must be between 1 and {}", rule.max_len),
                    ));
                }
                if let Some((re, pattern)) = rule.pattern {
                    if !re.is_match(value) {
                        errors.push(ValidationMessage::new(
                            rule.field,
                            Some(value),
                            &format!("must match \"{pattern}\""),
                        ));
                    }
                }
            }
        }
    }
    errors
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
}
