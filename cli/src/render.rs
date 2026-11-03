//! Plain-text rendering: ANSI colours, sparklines, aligned tables. No TUI framework on purpose.

use std::sync::atomic::{AtomicBool, Ordering};

static COLOR: AtomicBool = AtomicBool::new(true);

pub fn set_color(on: bool) {
    COLOR.store(on, Ordering::Relaxed);
}

fn paint(code: &str, s: &str) -> String {
    if COLOR.load(Ordering::Relaxed) {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn green(s: &str) -> String {
    paint("32", s)
}
pub fn yellow(s: &str) -> String {
    paint("33", s)
}
pub fn red(s: &str) -> String {
    paint("31", s)
}
pub fn java(s: &str) -> String {
    paint("38;5;221", s)
}
pub fn rust(s: &str) -> String {
    paint("38;5;209", s)
}
pub fn svc(name: &str) -> String {
    match name {
        "java" => java(name),
        _ => rust(name),
    }
}

pub fn term_width() -> usize {
    terminal_size::terminal_size().map(|(w, _)| w.0 as usize).unwrap_or(120).max(60)
}

const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// `values` resampled to `width` cells, scaled against `max` (shared across the pair being compared).
pub fn sparkline(values: &[f64], width: usize, max: f64) -> String {
    if values.is_empty() {
        return " ".repeat(width);
    }
    (0..width)
        .map(|i| {
            let from = i * values.len() / width;
            let to = ((i + 1) * values.len() / width).max(from + 1);
            let bucket = &values[from..to.min(values.len())];
            let v = bucket.iter().sum::<f64>() / bucket.len() as f64;
            let idx = if max > 0.0 {
                ((v / max) * 7.0).round().clamp(0.0, 7.0) as usize
            } else {
                0
            };
            BARS[idx]
        })
        .collect()
}

pub fn fmt_num(v: f64, decimals: usize) -> String {
    format!("{v:.decimals$}")
}

pub fn pct_delta(a: f64, b: f64) -> String {
    if a == 0.0 {
        return "n/a".into();
    }
    let d = (b - a) / a * 100.0;
    format!("{}{:.0}%", if d > 0.0 { "+" } else { "" }, d)
}

/// Two text blocks side by side, each clipped to its column; falls back to stacking on narrow terminals.
pub fn side_by_side(left_title: &str, left: &str, right_title: &str, right: &str) -> String {
    let width = term_width();
    if width < 100 {
        return format!("{left_title}\n{left}\n\n{right_title}\n{right}\n");
    }
    let col = (width - 3) / 2;
    let l: Vec<&str> = left.lines().collect();
    let r: Vec<&str> = right.lines().collect();
    let mut out = String::new();
    out.push_str(&format!("{}{}{}\n", pad(left_title, col), dim(" │ "), right_title));
    out.push_str(&format!("{}{}{}\n", dim(&"─".repeat(col)), dim("─┼─"), dim(&"─".repeat(col))));
    for i in 0..l.len().max(r.len()) {
        out.push_str(&format!(
            "{}{}{}\n",
            pad(l.get(i).copied().unwrap_or(""), col),
            dim(" │ "),
            clip(r.get(i).copied().unwrap_or(""), col)
        ));
    }
    out
}

/// Visible width: ANSI escape sequences do not count.
pub fn visible_len(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for n in chars.by_ref() {
                if n == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Clip to `width` visible chars; coloured strings are only clipped when they fit entirely or are dropped to plain text.
fn clip(s: &str, width: usize) -> String {
    if visible_len(s) <= width {
        return s.to_string();
    }
    let plain = strip_ansi(s);
    let mut t: String = plain.chars().take(width.saturating_sub(1)).collect();
    t.push('…');
    t
}

fn pad(s: &str, width: usize) -> String {
    let c = clip(s, width);
    format!("{c}{}", " ".repeat(width.saturating_sub(visible_len(&c))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_scales_against_shared_max() {
        assert_eq!(sparkline(&[0.0, 1.0, 2.0, 4.0], 4, 4.0), "▁▃▅█");
        assert_eq!(sparkline(&[4.0; 8], 4, 8.0), "▅▅▅▅");
        assert_eq!(sparkline(&[], 3, 1.0), "   ");
    }

    #[test]
    fn widths_ignore_ansi() {
        set_color(true);
        assert_eq!(visible_len(&green("abc")), 3);
        assert_eq!(pad(&green("abc"), 5).len(), green("abc").len() + 2);
        assert_eq!(pct_delta(0.48, 0.13), "-73%");
        assert_eq!(pct_delta(2374.0, 6936.0), "+192%");
    }
}
