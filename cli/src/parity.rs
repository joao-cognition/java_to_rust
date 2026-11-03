//! `j2r parity`: replays a catalogue of read requests (every GET endpoint × a matrix of routes, dates, cabins,
//! passenger counts, existing/missing ids, sold-out and error cases) against both services and requires identical
//! status, content type and JSON body. Key order is ignored; values, array order and number text (BigDecimal scale)
//! must match. Writes `parity.json` and `parity.md` to the report directory.

use crate::api::{self, Bases, Probe};
use crate::render::{bold, dim, green, red};
use crate::Service;
use chrono::{Duration, NaiveDate};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

const ROUTES: [(&str, &str); 9] = [
    ("NCE", "LHR"),
    ("NCE", "CDG"),
    ("CDG", "JFK"),
    ("LHR", "SIN"),
    ("MAD", "FRA"),
    ("AMS", "LIS"),
    ("FCO", "MUC"),
    ("BCN", "DXB"),
    ("DXB", "BOM"),
];
const CABINS: [&str; 5] = ["", "ECONOMY", "PREMIUM_ECONOMY", "BUSINESS", "FIRST"];

#[derive(Serialize)]
struct CaseResult {
    group: &'static str,
    path: String,
    identical: bool,
    java_status: u16,
    rust_status: u16,
    java_ms: f64,
    rust_ms: f64,
    bytes: usize,
    diffs: Vec<String>,
}

#[derive(Serialize)]
struct Report<'a> {
    java: &'a str,
    rust: &'a str,
    first_day: String,
    cases: usize,
    identical: usize,
    different: usize,
    results: &'a [CaseResult],
}

/// The first schedule day is the local departure date of flight 1 (the seeder starts tomorrow).
async fn first_day(client: &reqwest::Client, bases: &Bases) -> Result<NaiveDate, String> {
    let p = api::probe(client, &bases.url(Service::Java, "flights/1")).await;
    let dep = p
        .body
        .as_ref()
        .and_then(|b| b["departure"].as_str())
        .ok_or("Java service not reachable or not seeded (GET flights/1)")?;
    NaiveDate::parse_from_str(&dep[..10], "%Y-%m-%d").map_err(|e| e.to_string())
}

fn add(cases: &mut Vec<(&'static str, String)>, group: &'static str, path: impl Into<String>) {
    cases.push((group, path.into()));
}

async fn catalogue(client: &reqwest::Client, bases: &Bases, d0: NaiveDate) -> Vec<(&'static str, String)> {
    let day = |n: i64| (d0 + Duration::days(n)).format("%Y-%m-%d").to_string();
    let (d, d7, d29) = (day(0), day(7), day(29));
    let mut c = Vec::new();

    for p in [
        "airports",
        "airports/NCE",
        "airports/lhr",
        "airports/%20nce%20",
        "airports/ZZZ",
        "airports/",
        "airports/NCE/x",
    ] {
        add(&mut c, "airports", p);
    }
    for p in ["airlines", "airlines/AF", "airlines/ek", "airlines/%20ba", "airlines/XX"] {
        add(&mut c, "airlines", p);
    }

    for (o, de) in ROUTES {
        for date in [&d, &d7, &d29] {
            add(&mut c, "flights", format!("flights?origin={o}&destination={de}&date={date}"));
        }
    }
    for p in [
        format!("flights?origin=LHR&destination=NCE&date={d}"),
        format!("flights?origin=NCE&destination=LHR&date={}", day(30)),
        format!("flights?origin=NCE&destination=LHR&date={}", day(-1)),
        format!("flights?origin=nce&destination=%20lhr&date={d}"),
        format!("flights?destination=LHR&date={d}"),
        format!("flights?origin=NCE&date={d}"),
        "flights?origin=NCE&destination=LHR".into(),
        "flights?origin=NCE&destination=LHR&date=bad".into(),
        "flights?origin=NCE&destination=LHR&date=2026-02-30".into(),
        "flights?origin=NCE&destination=LHR&date=2026-11-4".into(),
        format!("flights?origin=NCE&destination=nce&date={d}"),
        format!("flights?origin=QQQ&destination=LHR&date={d}"),
        format!("flights?origin=NCE&destination=QQQ&date={d}"),
        format!("flights?origin=NCE&origin=CDG&destination=LHR&date={d}"),
        format!("flights?origin=NCE&destination=LHR&date={d}&date={}", day(1)),
    ] {
        add(&mut c, "flights", p);
    }
    for p in [
        "flights/1",
        "flights/2",
        "flights/3120",
        "flights/3121",
        "flights/0",
        "flights/-1",
        "flights/abc",
        "flights/0x10",
        "flights/%201",
        "flights/+5",
        "flights/1.5",
        "flights/99999999999999999999",
    ] {
        add(&mut c, "flight", p);
    }

    for (o, de) in ROUTES {
        for date in [&d, &d7] {
            for adults in [1, 2, 4, 9] {
                for cabin in CABINS {
                    let cabin = if cabin.is_empty() {
                        String::new()
                    } else {
                        format!("&cabin={cabin}")
                    };
                    add(
                        &mut c,
                        "offers",
                        format!("offers?origin={o}&destination={de}&date={date}&adults={adults}{cabin}"),
                    );
                }
            }
        }
        add(&mut c, "offers", format!("offers?origin={o}&destination={de}&date={d29}"));
    }
    for p in [
        format!("offers?origin=NCE&destination=LHR&date={d}"),
        format!("offers?origin=LHR&destination=NCE&date={d}"),
        format!("offers?origin=nce&destination=lhr&date={d}&adults=%202&cabin=%20BUSINESS"),
        format!("offers?origin=NCE&destination=LHR&date={d}&adults=&cabin="),
        format!("offers?origin=NCE&destination=LHR&date={d}&adults=0"),
        format!("offers?origin=NCE&destination=LHR&date={d}&adults=10"),
        format!("offers?origin=NCE&destination=LHR&date={d}&adults=x"),
        format!("offers?origin=NCE&destination=LHR&date={d}&adults=1.5"),
        format!("offers?origin=NCE&destination=LHR&date={d}&adults=2&adults=4&cabin=BUSINESS&cabin=FIRST"),
        format!("offers?origin=NCE&destination=LHR&date={d}&cabin=COACH"),
        format!("offers?origin=NCE&destination=LHR&date={d}&cabin=economy"),
        format!("offers?destination=LHR&date={d}"),
        "offers?origin=NCE&destination=LHR".into(),
        "offers?origin=NCE&destination=LHR&date=bad".into(),
        format!("offers?origin=NCE&destination=NCE&date={d}"),
        format!("offers?origin=QQQ&destination=LHR&date={d}"),
        format!("offers?origin=NCE&destination=LHR&date={}", day(30)),
    ] {
        add(&mut c, "offers", p);
    }

    for p in [
        "travelers/1",
        "travelers/2",
        "travelers/50",
        "travelers/51",
        "travelers/0",
        "travelers/abc",
        "travelers/0x1",
    ] {
        add(&mut c, "traveler", p);
    }
    for q in ["Mar", "mar", "%20Mar%20", "M", "a", "B", "Zzz", "%25", "_", "", "%20"] {
        add(&mut c, "travelers", format!("travelers?lastName={q}"));
    }
    add(&mut c, "travelers", "travelers");

    for p in ["bookings/QX7K2M", "bookings/qx7k2m", "bookings/%20qx7k2m", "bookings/ZZZZZZ"] {
        add(&mut c, "booking", p);
    }
    let mut locators = BTreeSet::new();
    for letter in 'A'..='Z' {
        let path = format!("bookings?lastName={letter}");
        if let Some(Value::Array(list)) = api::probe(client, &bases.url(Service::Java, &path)).await.body {
            if !list.is_empty() {
                locators.extend(list.iter().filter_map(|b| b["recordLocator"].as_str().map(str::to_owned)));
                add(&mut c, "bookings", path);
            }
        }
    }
    for q in ["Mar", "mar", "%20Mar", "Zzz", "%25", "", "%20"] {
        add(&mut c, "bookings", format!("bookings?lastName={q}"));
    }
    add(&mut c, "bookings", "bookings");
    for l in locators {
        add(&mut c, "booking", format!("bookings/{l}"));
    }

    add(&mut c, "errors", "nope");
    c
}

fn compare(group: &'static str, path: String, j: &Probe, r: &Probe) -> CaseResult {
    let mut diffs = Vec::new();
    if let Some(e) = r.error.as_ref().or(j.error.as_ref()) {
        diffs.push(e.clone());
    }
    if j.status != r.status {
        diffs.push(format!("status {} != {}", j.status, r.status));
    }
    if j.error.is_none() && r.error.is_none() && j.content_type != r.content_type {
        diffs.push(format!("content-type {} != {}", j.content_type, r.content_type));
    }
    match (&j.body, &r.body) {
        (Some(a), Some(b)) => api::diff_paths(a, b, "$", &mut diffs, 8),
        _ if j.text != r.text => diffs.push("body is not JSON on one side".into()),
        _ => {}
    }
    CaseResult {
        group,
        path,
        identical: diffs.is_empty(),
        java_status: j.status,
        rust_status: r.status,
        java_ms: j.ms,
        rust_ms: r.ms,
        bytes: j.bytes,
        diffs,
    }
}

pub async fn run(bases: &Bases, only: Option<&str>, out: &Path, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = api::client();
    let d0 = first_day(&client, bases).await?;
    let cases: Vec<_> = catalogue(&client, bases, d0)
        .await
        .into_iter()
        .filter(|(_, p)| only.is_none_or(|o| p.contains(o)))
        .collect();
    println!(
        "{}",
        bold(&format!(
            "parity  {} cases  java {}  rust {}  first day {d0}",
            cases.len(),
            bases.java,
            bases.rust
        ))
    );

    let mut results = Vec::with_capacity(cases.len());
    for (group, path) in cases {
        let j = api::probe(&client, &bases.url(Service::Java, &path)).await;
        let r = api::probe(&client, &bases.url(Service::Rust, &path)).await;
        let res = compare(group, path, &j, &r);
        if !res.identical || verbose {
            let mark = if res.identical { green("identical") } else { red("DIFF     ") };
            println!("  {mark}  {:>3} {:>3}  {}", res.java_status, res.rust_status, res.path);
            for d in &res.diffs {
                println!("      {}", dim(d));
            }
        }
        results.push(res);
    }

    let identical = results.iter().filter(|r| r.identical).count();
    let groups: Vec<&'static str> = results.iter().map(|r| r.group).fold(Vec::new(), |mut v, g| {
        if !v.contains(&g) {
            v.push(g);
        }
        v
    });
    println!();
    println!(
        "  {:<10} {:>6} {:>10} {:>6} {:>8}",
        "endpoint", "cases", "identical", "diff", "rust"
    );
    let mut md = format!(
        "# Parity report\n\nJava `{}` vs Rust `{}`, schedule starting {d0}. Every case: same HTTP status, same content type, same JSON body \
         (key order ignored; values, array order and number text compared exactly).\n\n\
         | endpoint | cases | identical | diff | rust |\n|---|---:|---:|---:|---|\n",
        bases.java, bases.rust
    );
    for g in &groups {
        let rs: Vec<&CaseResult> = results.iter().filter(|r| r.group == *g).collect();
        let ok = rs.iter().filter(|r| r.identical).count();
        let rust = if rs.iter().all(|r| r.rust_status == 0) { "down" } else { "up" };
        let line = format!("  {:<10} {:>6} {:>10} {:>6} {:>8}", g, rs.len(), ok, rs.len() - ok, rust);
        println!("{}", if ok == rs.len() { line } else { red(&line) });
        md.push_str(&format!("| {g} | {} | {ok} | {} | {rust} |\n", rs.len(), rs.len() - ok));
    }
    let verdict = if identical == results.len() { "identical" } else { "DIFFERENT" };
    md.push_str(&format!(
        "| **all** | **{}** | **{identical}** | **{}** | |\n\nverdict: **{verdict}**\n\n",
        results.len(),
        results.len() - identical
    ));
    md.push_str("## Cases\n\n| result | status java/rust | path | diff |\n|---|---|---|---|\n");
    for r in &results {
        md.push_str(&format!(
            "| {} | {}/{} | `{}` | {} |\n",
            if r.identical { "identical" } else { "**DIFF**" },
            r.java_status,
            r.rust_status,
            r.path,
            r.diffs.join("; ").replace('|', "\\|")
        ));
    }
    std::fs::create_dir_all(out)?;
    std::fs::write(out.join("parity.md"), md)?;
    let report = Report {
        java: &bases.java,
        rust: &bases.rust,
        first_day: d0.to_string(),
        cases: results.len(),
        identical,
        different: results.len() - identical,
        results: &results,
    };
    std::fs::write(out.join("parity.json"), serde_json::to_string_pretty(&report)?)?;
    println!();
    let summary = format!(
        "parity: {identical}/{} identical  →  {}",
        results.len(),
        out.join("parity.md").display()
    );
    println!(
        "{}",
        if identical == results.len() {
            green(&summary)
        } else {
            red(&summary)
        }
    );
    if identical == results.len() {
        Ok(())
    } else {
        Err(format!("{} of {} cases differ", results.len() - identical, results.len()).into())
    }
}
