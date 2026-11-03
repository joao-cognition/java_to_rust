//! Request-parameter binding with Spring MVC's semantics and error messages: repeated parameters are joined with
//! `,`, empty values of converted types count as absent, numbers are parsed like `NumberUtils.parseNumber`
//! (whitespace stripped, `0x`/`#` hex, leading sign), dates are strict ISO `yyyy-MM-dd`, enums are trimmed and
//! matched by exact constant name.

use crate::error::{ApiError, ApiResult};
use chrono::NaiveDate;

#[derive(Debug, Default, Clone)]
pub struct Params(Vec<(String, String)>);

impl Params {
    pub fn parse(query: Option<&str>) -> Self {
        Self(form_urlencoded::parse(query.unwrap_or("").as_bytes()).into_owned().collect())
    }

    /// All values of a parameter joined with `,` (Spring's `String[]` → `String` conversion).
    pub fn raw(&self, name: &str) -> Option<String> {
        let values: Vec<&str> = self.0.iter().filter(|(k, _)| k == name).map(|(_, v)| v.as_str()).collect();
        if values.is_empty() {
            None
        } else {
            Some(values.join(","))
        }
    }

    fn values(&self, name: &str) -> Vec<&str> {
        self.0.iter().filter(|(k, _)| k == name).map(|(_, v)| v.as_str()).collect()
    }

    /// A converted (non-`String`) parameter: Spring converts only the first of repeated values; when that fails the
    /// detail shows the `String[]` itself (`[Ljava.lang.String;@<identity hash>`).
    fn converted(&self, name: &str) -> Option<Converted<'_>> {
        let values = self.values(name);
        let first = *values.first()?;
        let shown = if values.len() == 1 {
            first.to_owned()
        } else {
            java_array_string(&values)
        };
        Some(Converted {
            first,
            repeated: values.len() > 1,
            shown,
        })
    }

    pub fn required_string(&self, name: &str) -> ApiResult<String> {
        self.raw(name).ok_or_else(|| ApiError::missing(name))
    }

    /// `@RequestParam @NotBlank String`: missing → 400 missing, blank → 400 validation failure.
    pub fn required_not_blank(&self, name: &str) -> ApiResult<String> {
        let v = self.required_string(name)?;
        if v.trim().is_empty() {
            return Err(ApiError::validation());
        }
        Ok(v)
    }

    pub fn required_date(&self, name: &str) -> ApiResult<NaiveDate> {
        match self.converted(name) {
            None => Err(ApiError::missing(name)),
            Some(c) if c.first.is_empty() => Err(ApiError::missing(name)),
            Some(c) => parse_iso_date(c.first).ok_or_else(|| ApiError::convert(name, &c.shown)),
        }
    }

    pub fn int_or(&self, name: &str, default: i32) -> ApiResult<i32> {
        match self.converted(name) {
            None => Ok(default),
            Some(c) if c.first.is_empty() && !c.repeated => Ok(default),
            Some(c) => parse_number(c.first)
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| ApiError::convert(name, &c.shown)),
        }
    }

    /// Optional enum parameter, `allowed` being the Java constant names.
    pub fn optional_enum(&self, name: &str, allowed: &[&'static str]) -> ApiResult<Option<&'static str>> {
        match self.converted(name) {
            None => Ok(None),
            Some(c) if c.first.trim().is_empty() => Ok(None),
            Some(c) => {
                let t = c.first.trim();
                allowed
                    .iter()
                    .find(|a| **a == t)
                    .map(|a| Some(*a))
                    .ok_or_else(|| ApiError::convert(name, &c.shown))
            }
        }
    }
}

struct Converted<'a> {
    first: &'a str,
    repeated: bool,
    shown: String,
}

/// `String[].toString()`; the identity hash is arbitrary in Java, here a stable hash of the values.
fn java_array_string(values: &[&str]) -> String {
    let hash = values
        .iter()
        .flat_map(|v| v.bytes().chain([0]))
        .fold(0x811c9dc5u32, |h, b| (h ^ b as u32).wrapping_mul(0x01000193));
    format!("[Ljava.lang.String;@{hash:x}")
}

/// `@PathVariable Long`.
pub fn path_long(name: &str, value: &str) -> ApiResult<i64> {
    parse_number(value).ok_or_else(|| ApiError::convert(name, value))
}

/// Spring `NumberUtils.parseNumber` for `Long`: all whitespace removed, optional sign, `0x`/`0X`/`#` hex,
/// otherwise decimal digits only; out-of-range values fail.
pub fn parse_number(text: &str) -> Option<i64> {
    let s: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let (neg, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.as_str()),
    };
    let hex = body
        .strip_prefix("0x")
        .or_else(|| body.strip_prefix("0X"))
        .or_else(|| body.strip_prefix('#'));
    if let Some(digits) = hex {
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let signed = if neg { format!("-{digits}") } else { digits.to_owned() };
        return i64::from_str_radix(&signed, 16).ok();
    }
    let digits = if neg { body } else { body.strip_prefix('+').unwrap_or(body) };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let signed = if neg { format!("-{digits}") } else { digits.to_owned() };
    signed.parse().ok()
}

/// `DateTimeFormatter.ISO_LOCAL_DATE` with strict resolution: exactly `yyyy-MM-dd`, real calendar dates only.
pub fn parse_iso_date(s: &str) -> Option<NaiveDate> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let digits = |r: std::ops::Range<usize>| -> Option<u32> {
        let part = &s[r];
        if part.bytes().all(|c| c.is_ascii_digit()) {
            part.parse().ok()
        } else {
            None
        }
    };
    NaiveDate::from_ymd_opt(digits(0..4)? as i32, digits(5..7)?, digits(8..10)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_parse_like_spring() {
        assert_eq!(parse_number(" 1"), Some(1));
        assert_eq!(parse_number("0x10"), Some(16));
        assert_eq!(parse_number("#10"), Some(16));
        assert_eq!(parse_number("+5"), Some(5));
        assert_eq!(parse_number("-1"), Some(-1));
        assert_eq!(parse_number("1.5"), None);
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("99999999999999999999"), None);
    }

    #[test]
    fn dates_are_strict_iso() {
        assert!(parse_iso_date("2026-11-04").is_some());
        assert!(parse_iso_date("2026-11-4").is_none());
        assert!(parse_iso_date(" 2026-11-04").is_none());
        assert!(parse_iso_date("2026-02-30").is_none());
    }

    #[test]
    fn repeated_parameters_join_with_comma() {
        let p = Params::parse(Some("origin=NCE&origin=CDG&x="));
        assert_eq!(p.raw("origin").as_deref(), Some("NCE,CDG"));
        assert_eq!(p.raw("x").as_deref(), Some(""));
        assert_eq!(p.required_string("y"), Err(ApiError::missing("y")));
    }

    #[test]
    fn enums_and_defaults() {
        let p = Params::parse(Some("cabin=%20BUSINESS&adults="));
        assert_eq!(p.optional_enum("cabin", &["ECONOMY", "BUSINESS"]), Ok(Some("BUSINESS")));
        assert_eq!(p.int_or("adults", 1), Ok(1));
        let bad = Params::parse(Some("cabin=economy&adults=x"));
        assert_eq!(bad.optional_enum("cabin", &["ECONOMY"]), Err(ApiError::convert("cabin", "economy")));
        assert_eq!(bad.int_or("adults", 1), Err(ApiError::convert("adults", "x")));
    }

    #[test]
    fn repeated_converted_parameters_use_the_first_value() {
        let p = Params::parse(Some("date=2026-11-04&date=bad&adults=2&adults=x&cabin=%20BUSINESS&cabin=x"));
        assert_eq!(p.required_date("date"), Ok(NaiveDate::from_ymd_opt(2026, 11, 4).unwrap()));
        assert_eq!(p.int_or("adults", 1), Ok(2));
        assert_eq!(p.optional_enum("cabin", &["BUSINESS"]), Ok(Some("BUSINESS")));
        let empty_first = Params::parse(Some("date=&date=2026-11-04&adults=&adults=2&cabin=&cabin=x"));
        assert_eq!(empty_first.required_date("date"), Err(ApiError::missing("date")));
        assert!(empty_first
            .int_or("adults", 1)
            .unwrap_err()
            .detail
            .contains("'[Ljava.lang.String;@"));
        assert_eq!(empty_first.optional_enum("cabin", &["BUSINESS"]), Ok(None));
        let bad_first = Params::parse(Some("date=bad&date=2026-11-04"));
        assert!(bad_first
            .required_date("date")
            .unwrap_err()
            .detail
            .starts_with("Failed to convert 'date' with value: '[Ljava.lang.String;@"));
    }
}
