//! Spring MVC request-parameter binding rules, so that the same query string yields the same value or error.

use chrono::NaiveDate;
use std::borrow::Cow;

pub struct Params<'a> {
    pairs: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Params<'a> {
    pub fn parse(raw: Option<&'a str>) -> Self {
        Self {
            pairs: raw
                .map(|q| form_urlencoded::parse(q.as_bytes()).collect())
                .unwrap_or_default(),
        }
    }

    /// First value: what Spring converts for non-String parameters (`date`, `adults`, `cabin`).
    pub fn first(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_ref())
    }

    /// String parameters: repeated values are joined with commas, like Spring's String[] → String conversion.
    pub fn string(&self, name: &str) -> Option<Cow<'_, str>> {
        let mut values = self
            .pairs
            .iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v.as_ref());
        let first = values.next()?;
        match values.next() {
            None => Some(Cow::Borrowed(first)),
            Some(second) => {
                let mut joined = format!("{first},{second}");
                for v in values {
                    joined.push(',');
                    joined.push_str(v);
                }
                Some(Cow::Owned(joined))
            }
        }
    }
}

/// `String.trim()`: strips leading/trailing chars <= U+0020.
pub fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= ' ')
}

/// `StringUtils.hasText`.
pub fn has_text(s: &str) -> bool {
    s.chars().any(|c| !c.is_whitespace())
}

/// `NumberUtils.parseNumber(text, Long.class)`: all whitespace removed, optional sign, decimal or `0x`/`0X`/`#` hex.
pub fn parse_java_long(text: &str) -> Option<i64> {
    let s: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let (neg, body) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, &s[..]),
    };
    let (digits, radix) =
        if let Some(h) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
            (h, 16)
        } else if let Some(h) = body.strip_prefix('#') {
            (h, 16)
        } else {
            (body, 10)
        };
    if digits.is_empty() || digits.starts_with(['+', '-']) {
        return None;
    }
    if radix == 16 && s.starts_with('+') {
        // Long.decode accepts '+', but NumberUtils only treats "-0x"/"0x"/"#" as hex.
        return None;
    }
    let magnitude = i128::from_str_radix(digits, radix).ok()?;
    let value = if neg { -magnitude } else { magnitude };
    i64::try_from(value).ok()
}

pub fn parse_java_int(text: &str) -> Option<i32> {
    parse_java_long(text).and_then(|v| i32::try_from(v).ok())
}

/// `DateTimeFormatter.ISO_DATE` (strict): `yyyy-MM-dd` with an optional `Z` or `+HH:MM[:SS]` offset that is
/// parsed but ignored, as for a `LocalDate`.
pub fn parse_iso_date(text: &str) -> Option<NaiveDate> {
    let b = text.as_bytes();
    let mut i = 0;
    let sign = match b.first() {
        Some(b'+') => {
            i = 1;
            1
        }
        Some(b'-') => {
            i = 1;
            -1
        }
        _ => 0,
    };
    let year_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let year_len = i - year_start;
    if !(4..=10).contains(&year_len) || (year_len > 4 && sign == 0) || (sign == 1 && year_len == 4)
    {
        return None;
    }
    let year: i64 = text[year_start..i].parse().ok()?;
    let year = i32::try_from(if sign < 0 { -year } else { year }).ok()?;
    let two = |at: usize| -> Option<u32> {
        let s = b.get(at..at + 2)?;
        if s.iter().all(u8::is_ascii_digit) {
            Some(u32::from(s[0] - b'0') * 10 + u32::from(s[1] - b'0'))
        } else {
            None
        }
    };
    if b.get(i) != Some(&b'-') {
        return None;
    }
    let month = two(i + 1)?;
    if b.get(i + 3) != Some(&b'-') {
        return None;
    }
    let day = two(i + 4)?;
    let rest = &b[i + 6..];
    let offset_ok = match rest {
        [] | [b'Z'] => true,
        [s, ..] if *s == b'+' || *s == b'-' => {
            let n = rest.len();
            let hh = two(i + 7);
            let mm = if rest.get(3) == Some(&b':') {
                two(i + 10)
            } else {
                None
            };
            let ss = if n == 9 && rest.get(6) == Some(&b':') {
                two(i + 13)
            } else {
                None
            };
            match (n, hh, mm, ss) {
                (6, Some(h), Some(m), _) => h <= 23 && m <= 59,
                (9, Some(h), Some(m), Some(s)) => h <= 23 && m <= 59 && s <= 59,
                _ => false,
            }
        }
        _ => false,
    };
    if !offset_ok {
        return None;
    }
    NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_number_binding() {
        assert_eq!(parse_java_long("16"), Some(16));
        assert_eq!(parse_java_long("0x10"), Some(16));
        assert_eq!(parse_java_long("#10"), Some(16));
        assert_eq!(parse_java_long(" 1 "), Some(1));
        assert_eq!(parse_java_long("+5"), Some(5));
        assert_eq!(parse_java_long("-0x10"), Some(-16));
        assert_eq!(parse_java_long("abc"), None);
        assert_eq!(parse_java_long(""), None);
        assert_eq!(parse_java_long("1.5"), None);
        assert_eq!(parse_java_long("99999999999999999999"), None);
        assert_eq!(parse_java_int("3000000000"), None);
    }

    #[test]
    fn iso_date() {
        let d = NaiveDate::from_ymd_opt(2026, 11, 5);
        assert_eq!(parse_iso_date("2026-11-05"), d);
        assert_eq!(parse_iso_date("2026-11-05Z"), d);
        assert_eq!(parse_iso_date("2026-11-05+01:00"), d);
        assert_eq!(parse_iso_date("2026-11-05+23:59:59"), d);
        assert_eq!(parse_iso_date("2026-11-05+24:00"), None);
        assert_eq!(parse_iso_date("2026-02-30"), None);
        assert_eq!(parse_iso_date(" 2026-11-05"), None);
        assert_eq!(parse_iso_date("26-11-05"), None);
        assert_eq!(parse_iso_date("2026-1-05"), None);
    }

    #[test]
    fn repeated_strings_join() {
        let p = Params::parse(Some("origin=NCE&origin=LHR&date=a&date=b"));
        assert_eq!(p.string("origin").as_deref(), Some("NCE,LHR"));
        assert_eq!(p.first("date"), Some("a"));
        assert_eq!(p.first("missing"), None);
    }
}
