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
        match self.raw(name) {
            None => Err(ApiError::missing(name)),
            Some(v) if v.is_empty() => Err(ApiError::missing(name)),
            Some(v) => parse_iso_date(&v).ok_or_else(|| ApiError::convert(name, &v)),
        }
    }

    pub fn int_or(&self, name: &str, default: i32) -> ApiResult<i32> {
        match self.raw(name) {
            None => Ok(default),
            Some(v) if v.is_empty() => Ok(default),
            Some(v) => parse_number(&v)
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| ApiError::convert(name, &v)),
        }
    }

    /// Optional enum parameter, `allowed` being the Java constant names.
    pub fn optional_enum(&self, name: &str, allowed: &[&'static str]) -> ApiResult<Option<&'static str>> {
        match self.raw(name) {
            None => Ok(None),
            Some(v) if v.trim().is_empty() => Ok(None),
            Some(v) => {
                let t = v.trim();
                allowed.iter().find(|a| **a == t).map(|a| Some(*a)).ok_or_else(|| ApiError::convert(name, &v))
            }
        }
    }
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
    let hex = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")).or_else(|| body.strip_prefix('#'));
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
}
