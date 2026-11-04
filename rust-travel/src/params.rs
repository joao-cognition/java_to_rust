//! Query/path parameter binding with Spring MVC semantics: repeated String params are joined with `,`, converted
//! params use the first value, empty converted values count as absent, numbers parse like `NumberUtils.parseNumber`.

use crate::error::{ApiError, ApiResult};
use chrono::NaiveDate;

pub struct Params(Vec<(String, String)>);

impl Params {
    pub fn parse(query: Option<&str>) -> Self {
        Self(
            form_urlencoded::parse(query.unwrap_or("").as_bytes())
                .into_owned()
                .collect(),
        )
    }

    fn values(&self, name: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// `@RequestParam String`.
    pub fn string(&self, name: &str) -> ApiResult<String> {
        let values = self.values(name);
        if values.is_empty() {
            return Err(ApiError::missing(name));
        }
        Ok(values.join(","))
    }

    /// `@RequestParam @NotBlank String`.
    pub fn not_blank(&self, name: &str) -> ApiResult<String> {
        let v = self.string(name)?;
        if v.trim().is_empty() {
            return Err(ApiError::validation());
        }
        Ok(v)
    }

    /// `@RequestParam @DateTimeFormat(iso = DATE) LocalDate`.
    pub fn date(&self, name: &str) -> ApiResult<NaiveDate> {
        match self.values(name).first() {
            None | Some(&"") => Err(ApiError::missing(name)),
            Some(v) => parse_iso_date(v).ok_or_else(|| ApiError::convert(name, v)),
        }
    }

    /// `@RequestParam(defaultValue = ..) int`.
    pub fn int_or(&self, name: &str, default: i32) -> ApiResult<i32> {
        match self.values(name).first() {
            None | Some(&"") => Ok(default),
            Some(v) => parse_number(v)
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| ApiError::convert(name, v)),
        }
    }

    /// `@RequestParam(required = false) <Enum>`: trimmed, exact constant name.
    pub fn optional_enum(
        &self,
        name: &str,
        allowed: &[&'static str],
    ) -> ApiResult<Option<&'static str>> {
        match self.values(name).first() {
            None => Ok(None),
            Some(v) if v.trim().is_empty() => Ok(None),
            Some(v) => allowed
                .iter()
                .find(|a| **a == v.trim())
                .map(|a| Some(*a))
                .ok_or_else(|| ApiError::convert(name, v)),
        }
    }
}

/// `@PathVariable Long`.
pub fn path_long(name: &str, value: &str) -> ApiResult<i64> {
    parse_number(value).ok_or_else(|| ApiError::convert(name, value))
}

/// Spring `NumberUtils.parseNumber` for `Long`: whitespace removed, optional sign, `0x`/`0X`/`#` hex, else decimal.
pub fn parse_number(text: &str) -> Option<i64> {
    let s: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let (neg, body) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s.as_str()),
    };
    let (radix, digits) =
        if let Some(h) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
            (16, h)
        } else if let Some(h) = body.strip_prefix('#') {
            (16, h)
        } else {
            (10, body)
        };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let signed = if neg {
        format!("-{digits}")
    } else {
        digits.to_owned()
    };
    i64::from_str_radix(&signed, radix).ok()
}

/// `DateTimeFormatter.ISO_DATE` (strict): `yyyy-MM-dd` with an optional, ignored offset.
fn parse_iso_date(text: &str) -> Option<NaiveDate> {
    let b = text.as_bytes();
    if b.len() < 10 || b[4] != b'-' || b[7] != b'-' || !text.is_char_boundary(10) {
        return None;
    }
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if !digits(0..4) || !digits(5..7) || !digits(8..10) {
        return None;
    }
    let rest = &text[10..];
    if !(rest.is_empty() || rest == "Z" || is_offset(rest)) {
        return None;
    }
    NaiveDate::parse_from_str(&text[..10], "%Y-%m-%d").ok()
}

/// `+HH:MM[:SS]` with hours 0-23 and minutes/seconds 0-59 (the range `ISO_DATE` checks while parsing).
fn is_offset(s: &str) -> bool {
    let b = s.as_bytes();
    let field = |i: usize, max: u8| {
        b[i].is_ascii_digit()
            && b[i + 1].is_ascii_digit()
            && (b[i] - b'0') * 10 + (b[i + 1] - b'0') <= max
    };
    matches!(b.first(), Some(b'+' | b'-'))
        && match b.len() {
            6 => field(1, 23) && b[3] == b':' && field(4, 59),
            9 => field(1, 23) && b[3] == b':' && field(4, 59) && b[6] == b':' && field(7, 59),
            _ => false,
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_parse_like_spring() {
        assert_eq!(parse_number("42"), Some(42));
        assert_eq!(parse_number(" 4 2 "), Some(42));
        assert_eq!(parse_number("+5"), Some(5));
        assert_eq!(parse_number("-0x10"), Some(-16));
        assert_eq!(parse_number("#ff"), Some(255));
        assert_eq!(parse_number("1.5"), None);
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("99999999999999999999"), None);
    }

    #[test]
    fn dates_are_strict_iso() {
        assert!(parse_iso_date("2026-11-05").is_some());
        assert!(parse_iso_date("2026-11-05+01:00").is_some());
        assert!(parse_iso_date("2026-02-30").is_none());
        assert!(parse_iso_date("2026-1-05").is_none());
        assert!(parse_iso_date("05/11/2026").is_none());
        assert!(parse_iso_date("2026-11-05+23:59:59").is_some());
        assert!(parse_iso_date("2026-11-05+99:99").is_none());
        assert!(parse_iso_date("2026-11-05+24:00").is_none());
        assert!(parse_iso_date("2026-11-05+00:60").is_none());
    }
}
