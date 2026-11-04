//! `j2r api`: one request to each service, raw bodies side by side, structural diff, optional burst.

use crate::render::{self, bold, dim, green, red, svc, yellow};
use serde_json::Value;
use std::time::{Duration, Instant};

pub struct Bases {
    pub java: String,
    pub rust: String,
}

impl Bases {
    pub fn url(&self, service: crate::Service, path: &str) -> String {
        let base = match service {
            crate::Service::Java => &self.java,
            crate::Service::Rust => &self.rust,
        };
        format!("{}/{}", base, path.trim_start_matches('/'))
    }
}

#[derive(Debug, Clone)]
pub struct Probe {
    pub status: u16,
    pub ms: f64,
    pub bytes: usize,
    pub content_type: String,
    pub body: Option<Value>,
    pub text: String,
    pub error: Option<String>,
}

impl Probe {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(256)
        .build()
        .expect("http client")
}

pub async fn probe(client: &reqwest::Client, url: &str) -> Probe {
    let t0 = Instant::now();
    match client.get(url).header("accept", "application/json").send().await {
        Ok(res) => {
            let status = res.status().as_u16();
            let content_type = res
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let text = match res.text().await {
                Ok(t) => t,
                Err(e) => {
                    return Probe {
                        status: 0,
                        ms: t0.elapsed().as_secs_f64() * 1000.0,
                        bytes: 0,
                        content_type,
                        body: None,
                        text: String::new(),
                        error: Some(format!("body read failed after HTTP {status}: {e}")),
                    }
                }
            };
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let body = serde_json::from_str::<Value>(&text).ok();
            Probe {
                status,
                ms,
                bytes: text.len(),
                content_type,
                body,
                text,
                error: None,
            }
        }
        Err(e) => Probe {
            status: 0,
            ms: t0.elapsed().as_secs_f64() * 1000.0,
            bytes: 0,
            content_type: String::new(),
            body: None,
            text: String::new(),
            error: Some(if e.is_connect() {
                "connection refused — nothing listening".into()
            } else {
                e.to_string()
            }),
        },
    }
}

/// serde_json's default `Map` is a BTreeMap, so `to_string` already has sorted keys: field order never counts as a diff.
pub fn canonical(v: &Value) -> String {
    v.to_string()
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// JSON paths of leaf differences, capped.
pub fn diff_paths(a: &Value, b: &Value, path: &str, out: &mut Vec<String>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    match (a, b) {
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                out.push(format!("{path}.length {} != {}", x.len(), y.len()));
            }
            for (i, (p, q)) in x.iter().zip(y.iter()).enumerate() {
                diff_paths(p, q, &format!("{path}[{i}]"), out, cap);
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            let mut keys: Vec<&String> = x.keys().chain(y.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                match (x.get(k), y.get(k)) {
                    (None, _) => out.push(format!("{path}.{k} missing in java")),
                    (_, None) => out.push(format!("{path}.{k} missing in rust")),
                    (Some(p), Some(q)) => diff_paths(p, q, &format!("{path}.{k}"), out, cap),
                }
                if out.len() >= cap {
                    return;
                }
            }
        }
        _ if kind(a) != kind(b) => out.push(format!("{path}: {} in java, {} in rust", kind(a), kind(b))),
        _ if a != b => out.push(format!("{path}: {a} != {b}")),
        _ => {}
    }
}

fn status_line(service: crate::Service, p: &Probe) -> String {
    let st = if p.ok() {
        green(&p.status.to_string())
    } else if p.status == 0 {
        red("down")
    } else {
        red(&format!("HTTP {}", p.status))
    };
    if p.status == 0 {
        format!("{} {}  {}", bold(&svc(service.name())), st, dim(p.error.as_deref().unwrap_or("")))
    } else {
        format!(
            "{} {}  {:.1}ms  {}",
            bold(&svc(service.name())),
            st,
            p.ms,
            dim(&format!("{}B", p.bytes))
        )
    }
}

fn body_text(p: &Probe) -> String {
    if p.status == 0 {
        return dim("# no response");
    }
    match &p.body {
        Some(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
        None => p.text.clone(),
    }
}

pub async fn compare(bases: &Bases, path: &str, burst: usize, show_body: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = client();
    let (ju, ru) = (bases.url(crate::Service::Java, path), bases.url(crate::Service::Rust, path));
    let (j, r) = tokio::join!(probe(&client, &ju), probe(&client, &ru));

    println!("{} {}", dim("GET"), ju);
    println!("{} {}", dim("GET"), ru);
    println!();

    if show_body {
        print!(
            "{}",
            render::side_by_side(
                &status_line(crate::Service::Java, &j),
                &body_text(&j),
                &status_line(crate::Service::Rust, &r),
                &body_text(&r)
            )
        );
    } else {
        println!("{}", status_line(crate::Service::Java, &j));
        println!("{}", status_line(crate::Service::Rust, &r));
    }

    println!();
    match (j.ok(), r.ok()) {
        (false, _) => println!(
            "{}",
            yellow(&format!(
                "diff: n/a — java {}",
                if j.status == 0 {
                    "not reachable".to_string()
                } else {
                    format!("HTTP {}", j.status)
                }
            ))
        ),
        (_, false) => println!(
            "{}",
            yellow(&format!(
                "diff: n/a — rust {}",
                if r.status == 0 {
                    "not reachable".to_string()
                } else {
                    format!("HTTP {}", r.status)
                }
            ))
        ),
        (true, true) => match (&j.body, &r.body) {
            (Some(a), Some(b)) if canonical(a) == canonical(b) => {
                println!("{}", green(&format!("diff: identical ({}B, key order ignored)", j.bytes)));
            }
            (Some(a), Some(b)) => {
                let mut out = Vec::new();
                diff_paths(a, b, "$", &mut out, 12);
                if out.is_empty() {
                    out.push("$: bodies differ".into());
                }
                println!(
                    "{}",
                    yellow(&format!("diff: {} difference{}", out.len(), if out.len() > 1 { "s" } else { "" }))
                );
                for d in out {
                    println!("  {d}");
                }
            }
            _ if j.text == r.text => println!("{}", green(&format!("diff: identical ({}B, non-JSON body)", j.bytes))),
            _ => println!(
                "{}",
                yellow(&format!("diff: non-JSON bodies differ ({}B vs {}B)", j.bytes, r.bytes))
            ),
        },
    }

    if burst > 0 {
        println!();
        println!(
            "{}",
            dim(&format!(
                "{burst} sequential requests per side, GET /{}",
                path.trim_start_matches('/')
            ))
        );
        let run = |url: String| {
            let client = client.clone();
            async move {
                let mut lat = Vec::with_capacity(burst);
                let mut errors = 0usize;
                for _ in 0..burst {
                    let p = probe(&client, &url).await;
                    if p.ok() {
                        lat.push(p.ms);
                    } else {
                        errors += 1;
                    }
                }
                (lat, errors)
            }
        };
        let ((jl, je), (rl, re)) = tokio::join!(run(ju.clone()), run(ru.clone()));
        for (name, lat, errors) in [("java", jl, je), ("rust", rl, re)] {
            if lat.is_empty() {
                println!("  {}  n/a ({errors} errors)", svc(name));
            } else {
                println!(
                    "  {}  p50 {:>7.1} ms   p99 {:>7.1} ms   min {:>6.1} ms   errors {errors}",
                    svc(name),
                    percentile(&lat, 0.5),
                    percentile(&lat, 0.99),
                    lat.iter().cloned().fold(f64::MAX, f64::min)
                );
            }
        }
    }
    Ok(())
}

pub fn percentile(samples: &[f64], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((p * s.len() as f64) as usize).min(s.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_ignores_key_order() {
        let a: Value = serde_json::from_str(r#"{"id":1,"name":"cat"}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"name":"cat","id":1}"#).unwrap();
        assert_eq!(canonical(&a), canonical(&b));
    }

    #[test]
    fn diff_reports_paths_and_type_mismatches() {
        let mut out = Vec::new();
        diff_paths(
            &json!({"a": [1, 2], "b": {"c": "x"}}),
            &json!({"a": [1, 3], "b": {"c": "x", "d": 1}}),
            "$",
            &mut out,
            12,
        );
        assert_eq!(out, vec!["$.a[1]: 2 != 3", "$.b.d missing in java"]);

        let mut out = Vec::new();
        diff_paths(&json!([]), &json!({}), "$", &mut out, 12);
        assert_eq!(out, vec!["$: array in java, object in rust"]);
    }

    #[test]
    fn percentile_picks_upper_tail() {
        let v: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        assert_eq!(percentile(&v, 0.5), 51.0);
        assert_eq!(percentile(&v, 0.99), 100.0);
        assert_eq!(percentile(&[], 0.5), 0.0);
    }
}
