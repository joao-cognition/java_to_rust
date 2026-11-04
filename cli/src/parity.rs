//! `j2r parity`: replays a catalogue of GET requests (every endpoint × routes, dates, cabins, passenger counts,
//! existing and missing ids, sold-out and error cases) against both services and requires the same status,
//! `Content-Type` and body. Bodies are compared with the `j2r api` diff (key order ignored; values, array order and
//! number text such as BigDecimal scale compared exactly); for RFC 9457 problem bodies every field (`type`, `title`,
//! `status`, `detail`, `instance`) is checked by name.
//!
//! `--record` also writes the Java responses to the golden directory; `--golden` replays those goldens as the Java
//! side so the Rust service can be checked without Java running. Results go to `results.json` and `report.md`.

use crate::api::{self, Bases, Probe};
use crate::render::{bold, dim, green, red, yellow};
use crate::Service;
use chrono::{Duration, Local, NaiveDate};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Every GET endpoint in scope, in report order. Cases outside these templates are grouped as `unknown route`.
pub const ENDPOINTS: [&str; 11] = [
    "airports",
    "airports/{iata}",
    "airlines",
    "airlines/{iata}",
    "flights",
    "flights/{id}",
    "offers",
    "travelers",
    "travelers/{id}",
    "bookings",
    "bookings/{recordLocator}",
];
const UNKNOWN: &str = "unknown route";

/// Seeded city pairs (`DemoDataSeeder.ROUTES`): NCE-CDG is domestic (VAT); the rest spread over every
/// fuel-surcharge distance band, short to ultra long haul.
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
const AIRPORTS: [&str; 14] = [
    "NCE", "LHR", "CDG", "JFK", "SIN", "MAD", "FRA", "AMS", "LIS", "FCO", "MUC", "BCN", "DXB", "BOM",
];
const AIRLINES: [&str; 11] = ["AF", "BA", "DL", "SQ", "IB", "LH", "KL", "TP", "AZ", "EK", "AI"];
const CABINS: [&str; 5] = ["", "ECONOMY", "PREMIUM_ECONOMY", "BUSINESS", "FIRST"];
const PROBLEM_FIELDS: [&str; 5] = ["type", "title", "status", "detail", "instance"];
const MANIFEST: &str = "manifest.json";
const GOLDENS: &str = "goldens.json.gz";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Case {
    pub endpoint: String,
    pub label: String,
    pub path: String,
}

/// Data-dependent inputs of the catalogue, read from the reference (Java) service.
#[derive(Debug, Default)]
pub struct Facts {
    /// First schedule day (the seeder starts the day after the server starts).
    pub first_day: NaiveDate,
    pub last_flight: u64,
    pub last_traveler: u64,
    /// Record locators of every seeded booking.
    pub locators: BTreeSet<String>,
    /// Fares with few seats left: (route index, cabin, seats) → an `adults = seats + 1` request excludes them.
    pub sold_out: Vec<(usize, String, u64)>,
}

pub fn slug(label: &str) -> String {
    let mut s = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.is_empty() && !s.ends_with('-') {
            s.push('-');
        }
    }
    s.trim_end_matches('-').to_string()
}

struct Builder {
    d0: NaiveDate,
    cases: Vec<Case>,
}

impl Builder {
    fn add(&mut self, endpoint: &str, label: impl Into<String>, path: impl Into<String>) {
        self.cases.push(Case {
            endpoint: endpoint.into(),
            label: label.into(),
            path: path.into(),
        });
    }
    /// Date of schedule day `n` (0 = first day), as sent in the query.
    fn day(&self, n: i64) -> String {
        (self.d0 + Duration::days(n)).format("%Y-%m-%d").to_string()
    }
}

fn day_label(n: i64) -> String {
    if n < 0 {
        format!("d{n}")
    } else {
        format!("d+{n}")
    }
}

/// The full case catalogue. Dates are relative to `facts.first_day`; labels never contain a date, so golden keys
/// stay stable from one recording day to the next.
pub fn catalogue(facts: &Facts) -> Vec<Case> {
    let mut b = Builder {
        d0: facts.first_day,
        cases: Vec::new(),
    };

    b.add("airports", "airports list", "airports");
    for iata in AIRPORTS {
        b.add("airports/{iata}", format!("airport {iata}"), format!("airports/{iata}"));
    }
    for (label, path) in [
        ("airport lowercase lhr", "airports/lhr"),
        ("airport mixed case Nce", "airports/Nce"),
        ("airport padded nce", "airports/%20nce%20"),
        ("airport missing ZZZ", "airports/ZZZ"),
        ("airport too long NCEX", "airports/NCEX"),
        ("airport numeric 123", "airports/123"),
        ("airport trailing slash", "airports/"),
        ("airport extra segment", "airports/NCE/x"),
    ] {
        b.add("airports/{iata}", label, path);
    }

    b.add("airlines", "airlines list", "airlines");
    for iata in AIRLINES {
        b.add("airlines/{iata}", format!("airline {iata}"), format!("airlines/{iata}"));
    }
    for (label, path) in [
        ("airline lowercase ek", "airlines/ek"),
        ("airline padded ba", "airlines/%20ba"),
        ("airline missing XX", "airlines/XX"),
        ("airline too long AFX", "airlines/AFX"),
    ] {
        b.add("airlines/{iata}", label, path);
    }

    for (o, de) in ROUTES {
        for (from, to) in [(o, de), (de, o)] {
            for n in [0, 7, 15, 29] {
                let path = format!("flights?origin={from}&destination={to}&date={}", b.day(n));
                b.add("flights", format!("flights {from}-{to} {}", day_label(n)), path);
            }
        }
    }
    let d = b.day(0);
    for (label, path) in [
        (
            "flights NCE-LHR d+30 outside window",
            format!("flights?origin=NCE&destination=LHR&date={}", b.day(30)),
        ),
        (
            "flights NCE-LHR d-1 before window",
            format!("flights?origin=NCE&destination=LHR&date={}", b.day(-1)),
        ),
        ("flights lowercase iata", format!("flights?origin=nce&destination=lhr&date={d}")),
        ("flights padded iata", format!("flights?origin=%20NCE&destination=LHR%20&date={d}")),
        (
            "flights unserved route NCE-JFK",
            format!("flights?origin=NCE&destination=JFK&date={d}"),
        ),
        ("flights unknown origin", format!("flights?origin=QQQ&destination=LHR&date={d}")),
        (
            "flights unknown destination",
            format!("flights?origin=NCE&destination=QQQ&date={d}"),
        ),
        (
            "flights same origin and destination",
            format!("flights?origin=NCE&destination=nce&date={d}"),
        ),
        (
            "flights repeated origin",
            format!("flights?origin=NCE&origin=CDG&destination=LHR&date={d}"),
        ),
        ("flights missing origin", format!("flights?destination=LHR&date={d}")),
        ("flights missing destination", format!("flights?origin=NCE&date={d}")),
        ("flights missing date", "flights?origin=NCE&destination=LHR".into()),
        ("flights no params", "flights".into()),
        ("flights malformed date word", "flights?origin=NCE&destination=LHR&date=bad".into()),
        (
            "flights malformed date feb 30",
            "flights?origin=NCE&destination=LHR&date=2026-02-30".into(),
        ),
        (
            "flights malformed date short day",
            "flights?origin=NCE&destination=LHR&date=2026-11-4".into(),
        ),
        (
            "flights malformed date compact",
            "flights?origin=NCE&destination=LHR&date=20261105".into(),
        ),
        ("flights malformed date empty", "flights?origin=NCE&destination=LHR&date=".into()),
    ] {
        b.add("flights", label, path);
    }

    let last = facts.last_flight.max(2);
    for (label, path) in [
        ("flight first".to_string(), "flights/1".to_string()),
        ("flight second".into(), "flights/2".into()),
        ("flight middle".into(), format!("flights/{}", last / 2)),
        ("flight last".into(), format!("flights/{last}")),
        ("flight missing after last".into(), format!("flights/{}", last + 1)),
        ("flight zero".into(), "flights/0".into()),
        ("flight negative".into(), "flights/-1".into()),
        ("flight non numeric".into(), "flights/abc".into()),
        ("flight decimal".into(), "flights/1.5".into()),
        ("flight overflow".into(), "flights/99999999999999999999".into()),
    ] {
        b.add("flights/{id}", label, path);
    }

    for (o, de) in ROUTES {
        for n in [0, 7] {
            for adults in [1, 2, 5, 9] {
                for cabin in CABINS {
                    let (cl, cq) = if cabin.is_empty() {
                        ("any".to_string(), String::new())
                    } else {
                        (cabin.to_string(), format!("&cabin={cabin}"))
                    };
                    b.add(
                        "offers",
                        format!("offers {o}-{de} {} adults={adults} cabin={cl}", day_label(n)),
                        format!("offers?origin={o}&destination={de}&date={}&adults={adults}{cq}", b.day(n)),
                    );
                }
            }
        }
        for n in [15, 29] {
            b.add(
                "offers",
                format!("offers {o}-{de} {} defaults", day_label(n)),
                format!("offers?origin={o}&destination={de}&date={}", b.day(n)),
            );
        }
        b.add(
            "offers",
            format!("offers {de}-{o} d+0 defaults"),
            format!("offers?origin={de}&destination={o}&date={d}"),
        );
    }
    for (o, de) in [("NCE", "LHR"), ("CDG", "JFK")] {
        for adults in ["0", "10", "-1"] {
            b.add(
                "offers",
                format!("offers {o}-{de} d+0 adults={adults} out of range"),
                format!("offers?origin={o}&destination={de}&date={d}&adults={adults}"),
            );
        }
    }
    for &(route, ref cabin, seats) in &facts.sold_out {
        let (o, de) = ROUTES[route];
        let adults = seats + 1;
        b.add(
            "offers",
            format!("offers sold out {o}-{de} d+0 adults={adults} cabin={cabin}"),
            format!("offers?origin={o}&destination={de}&date={d}&adults={adults}&cabin={cabin}"),
        );
    }
    for (label, path) in [
        (
            "offers NCE-LHR d+30 outside window",
            format!("offers?origin=NCE&destination=LHR&date={}", b.day(30)),
        ),
        (
            "offers NCE-LHR d-1 before window",
            format!("offers?origin=NCE&destination=LHR&date={}", b.day(-1)),
        ),
        ("offers lowercase iata", format!("offers?origin=nce&destination=lhr&date={d}")),
        (
            "offers padded adults and cabin",
            format!("offers?origin=NCE&destination=LHR&date={d}&adults=%202&cabin=%20BUSINESS"),
        ),
        (
            "offers empty adults and cabin",
            format!("offers?origin=NCE&destination=LHR&date={d}&adults=&cabin="),
        ),
        (
            "offers adults non numeric",
            format!("offers?origin=NCE&destination=LHR&date={d}&adults=x"),
        ),
        (
            "offers adults decimal",
            format!("offers?origin=NCE&destination=LHR&date={d}&adults=1.5"),
        ),
        (
            "offers cabin unknown",
            format!("offers?origin=NCE&destination=LHR&date={d}&cabin=COACH"),
        ),
        (
            "offers cabin lowercase",
            format!("offers?origin=NCE&destination=LHR&date={d}&cabin=economy"),
        ),
        (
            "offers unserved route NCE-JFK",
            format!("offers?origin=NCE&destination=JFK&date={d}"),
        ),
        ("offers unknown origin", format!("offers?origin=QQQ&destination=LHR&date={d}")),
        ("offers unknown destination", format!("offers?origin=NCE&destination=QQQ&date={d}")),
        (
            "offers same origin and destination",
            format!("offers?origin=NCE&destination=NCE&date={d}"),
        ),
        ("offers missing origin", format!("offers?destination=LHR&date={d}")),
        ("offers missing destination", format!("offers?origin=NCE&date={d}")),
        ("offers missing date", "offers?origin=NCE&destination=LHR".into()),
        ("offers no params", "offers".into()),
        ("offers malformed date word", "offers?origin=NCE&destination=LHR&date=bad".into()),
        (
            "offers malformed date feb 30",
            "offers?origin=NCE&destination=LHR&date=2026-02-30".into(),
        ),
    ] {
        b.add("offers", label, path);
    }

    let last = facts.last_traveler.max(2);
    for (label, path) in [
        ("traveler first".to_string(), "travelers/1".to_string()),
        ("traveler second".into(), "travelers/2".into()),
        ("traveler middle".into(), format!("travelers/{}", last / 2)),
        ("traveler last".into(), format!("travelers/{last}")),
        ("traveler missing after last".into(), format!("travelers/{}", last + 1)),
        ("traveler zero".into(), "travelers/0".into()),
        ("traveler non numeric".into(), "travelers/abc".into()),
    ] {
        b.add("travelers/{id}", label, path);
    }
    for letter in 'A'..='Z' {
        b.add(
            "travelers",
            format!("travelers lastName {letter}"),
            format!("travelers?lastName={letter}"),
        );
    }
    for (label, q) in [
        ("travelers lastName Mar", "Mar"),
        ("travelers lastName mar lowercase", "mar"),
        ("travelers lastName padded Mar", "%20Mar%20"),
        ("travelers lastName unknown Zzz", "Zzz"),
        ("travelers lastName percent", "%25"),
        ("travelers lastName underscore", "_"),
        ("travelers lastName empty", ""),
        ("travelers lastName blank", "%20"),
    ] {
        b.add("travelers", label, format!("travelers?lastName={q}"));
    }
    b.add("travelers", "travelers missing lastName", "travelers");

    for (label, path) in [
        ("booking QX7K2M", "bookings/QX7K2M"),
        ("booking lowercase qx7k2m", "bookings/qx7k2m"),
        ("booking padded qx7k2m", "bookings/%20qx7k2m"),
        ("booking missing ZZZZZZ", "bookings/ZZZZZZ"),
        ("booking too short QX7", "bookings/QX7"),
    ] {
        b.add("bookings/{recordLocator}", label, path);
    }
    for l in facts.locators.iter().filter(|l| l.as_str() != "QX7K2M") {
        b.add("bookings/{recordLocator}", format!("booking {l}"), format!("bookings/{l}"));
    }
    for letter in 'A'..='Z' {
        b.add(
            "bookings",
            format!("bookings lastName {letter}"),
            format!("bookings?lastName={letter}"),
        );
    }
    for (label, q) in [
        ("bookings lastName Mar", "Mar"),
        ("bookings lastName mar lowercase", "mar"),
        ("bookings lastName padded Mar", "%20Mar"),
        ("bookings lastName unknown Zzz", "Zzz"),
        ("bookings lastName percent", "%25"),
        ("bookings lastName underscore", "_"),
        ("bookings lastName empty", ""),
        ("bookings lastName blank", "%20"),
    ] {
        b.add("bookings", label, format!("bookings?lastName={q}"));
    }
    b.add("bookings", "bookings missing lastName", "bookings");

    b.add(UNKNOWN, "unknown path", "nope");
    b.add(UNKNOWN, "unknown sub resource", "flights/1/fares");
    b.cases
}

/// Labels must slugify uniquely: the slug is the golden key.
pub fn check_unique(cases: &[Case]) -> Result<(), String> {
    let mut seen = BTreeMap::new();
    for c in cases {
        if let Some(prev) = seen.insert(slug(&c.label), &c.label) {
            return Err(format!("case labels {prev:?} and {:?} have the same slug", c.label));
        }
    }
    Ok(())
}

async fn status(client: &reqwest::Client, bases: &Bases, path: &str) -> u16 {
    api::probe(client, &bases.url(Service::Java, path)).await.status
}

/// Highest id `n` for which `GET {prefix}/n` is 200 (ids are contiguous from 1 on a freshly seeded schema).
async fn last_id(client: &reqwest::Client, bases: &Bases, prefix: &str) -> u64 {
    if status(client, bases, &format!("{prefix}/1")).await != 200 {
        return 0;
    }
    let mut hi = 2u64;
    while hi < 1 << 40 && status(client, bases, &format!("{prefix}/{hi}")).await == 200 {
        hi *= 2;
    }
    let mut lo = hi / 2;
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if status(client, bases, &format!("{prefix}/{mid}")).await == 200 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

fn tomorrow() -> NaiveDate {
    Local::now().date_naive() + Duration::days(1)
}

/// Reads the catalogue facts from the Java service; falls back to "tomorrow" and no data-driven cases when it is down.
async fn facts(client: &reqwest::Client, bases: &Bases) -> Facts {
    let first = api::probe(client, &bases.url(Service::Java, "flights/1")).await;
    let Some(first_day) = first
        .body
        .as_ref()
        .and_then(|b| b["departure"].as_str())
        .and_then(|d| NaiveDate::parse_from_str(d.get(..10)?, "%Y-%m-%d").ok())
    else {
        eprintln!(
            "{}",
            yellow("java: GET flights/1 failed — dates computed from tomorrow, data-driven cases (PNRs, sold out) skipped")
        );
        return Facts {
            first_day: tomorrow(),
            last_flight: 0,
            last_traveler: 0,
            ..Facts::default()
        };
    };
    let mut f = Facts {
        first_day,
        last_flight: last_id(client, bases, "flights").await,
        last_traveler: last_id(client, bases, "travelers").await,
        ..Facts::default()
    };
    for letter in 'A'..='Z' {
        let path = format!("bookings?lastName={letter}");
        if let Some(Value::Array(list)) = api::probe(client, &bases.url(Service::Java, &path)).await.body {
            f.locators
                .extend(list.iter().filter_map(|b| b["recordLocator"].as_str().map(str::to_owned)));
        }
    }
    // One sold-out case per route: the cabin whose cheapest-seat fare has the fewest seats left (at most 8, so the
    // request stays within the 1..9 adults limit).
    let d = first_day.format("%Y-%m-%d").to_string();
    for (i, (o, de)) in ROUTES.iter().enumerate() {
        let path = format!("offers?origin={o}&destination={de}&date={d}&adults=1");
        let Some(Value::Array(offers)) = api::probe(client, &bases.url(Service::Java, &path)).await.body else {
            continue;
        };
        let pick = offers
            .iter()
            .filter_map(|x| Some((x["fare"]["seatsAvailable"].as_u64()?, x["fare"]["cabin"].as_str()?)))
            .filter(|(s, _)| (1..=8).contains(s))
            .min();
        if let Some((seats, cabin)) = pick {
            f.sold_out.push((i, cabin.to_string(), seats));
        }
    }
    f
}

/// One side of a comparison: a live response or a recorded golden.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip)]
    pub error: Option<String>,
}

impl From<Probe> for Response {
    fn from(p: Probe) -> Self {
        let text = if p.body.is_none() && p.error.is_none() {
            Some(p.text)
        } else {
            None
        };
        Response {
            status: p.status,
            content_type: p.content_type,
            body: p.body,
            text,
            error: p.error,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Golden {
    endpoint: String,
    label: String,
    path: String,
    #[serde(flatten)]
    response: Response,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    first_day: NaiveDate,
    java: String,
    recorded_at: String,
    cases: usize,
    /// Case slugs (the keys of `goldens.json.gz`), in catalogue order.
    slugs: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Identical,
    Different,
    RustDown,
    JavaDown,
}

impl Verdict {
    fn text(self) -> &'static str {
        match self {
            Verdict::Identical => "identical",
            Verdict::Different => "DIFF",
            Verdict::RustDown => "rust down",
            Verdict::JavaDown => "java down",
        }
    }
}

#[derive(Serialize)]
pub struct CaseResult {
    pub endpoint: String,
    pub label: String,
    pub path: String,
    pub verdict: Verdict,
    pub java_status: u16,
    pub rust_status: u16,
    pub java_content_type: String,
    pub rust_content_type: String,
    pub diffs: Vec<String>,
}

fn is_problem(r: &Response) -> bool {
    r.content_type.starts_with("application/problem+json")
}

fn show(v: Option<&Value>) -> String {
    v.map_or_else(|| "absent".to_string(), |v| v.to_string())
}

pub fn compare(case: &Case, j: &Response, r: &Response) -> CaseResult {
    let mut diffs = Vec::new();
    let verdict = if let Some(e) = &j.error {
        diffs.push(format!("java: {e}"));
        Verdict::JavaDown
    } else if let Some(e) = &r.error {
        diffs.push(format!("rust: {e}"));
        Verdict::RustDown
    } else {
        if j.status != r.status {
            diffs.push(format!("status: java {}, rust {}", j.status, r.status));
        }
        if j.content_type != r.content_type {
            diffs.push(format!("content-type: java {:?}, rust {:?}", j.content_type, r.content_type));
        }
        match (&j.body, &r.body) {
            (Some(a), Some(b)) if is_problem(j) || is_problem(r) => {
                let (mut a, mut b) = (a.clone(), b.clone());
                for f in PROBLEM_FIELDS {
                    let (x, y) = (
                        a.as_object_mut().and_then(|m| m.remove(f)),
                        b.as_object_mut().and_then(|m| m.remove(f)),
                    );
                    if x != y {
                        diffs.push(format!("problem.{f}: java {}, rust {}", show(x.as_ref()), show(y.as_ref())));
                    }
                }
                api::diff_paths(&a, &b, "$", &mut diffs, 12);
            }
            (Some(a), Some(b)) => api::diff_paths(a, b, "$", &mut diffs, 12),
            (None, None) if j.text == r.text => {}
            (None, None) => diffs.push("non-JSON bodies differ".into()),
            (Some(_), None) => diffs.push("body: JSON in java, not JSON in rust".into()),
            (None, Some(_)) => diffs.push("body: not JSON in java, JSON in rust".into()),
        }
        if diffs.is_empty() {
            Verdict::Identical
        } else {
            Verdict::Different
        }
    };
    CaseResult {
        endpoint: case.endpoint.clone(),
        label: case.label.clone(),
        path: case.path.clone(),
        verdict,
        java_status: j.status,
        rust_status: r.status,
        java_content_type: j.content_type.clone(),
        rust_content_type: r.content_type.clone(),
        diffs,
    }
}

pub struct Options<'a> {
    pub only: Option<&'a str>,
    pub out: &'a Path,
    pub golden_dir: &'a Path,
    pub record: bool,
    pub golden: bool,
}

type Goldens = (NaiveDate, Vec<(Case, Response)>);

fn load_goldens(dir: &Path) -> Result<Goldens, Box<dyn std::error::Error>> {
    let manifest_path = dir.join(MANIFEST);
    let manifest: Manifest = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).map_err(|e| format!("{}: {e} (record with --record)", manifest_path.display()))?,
    )?;
    let file = dir.join(GOLDENS);
    let gz = std::fs::File::open(&file).map_err(|e| format!("{}: {e} (record with --record)", file.display()))?;
    let mut goldens: BTreeMap<String, Golden> =
        serde_json::from_reader(std::io::BufReader::new(GzDecoder::new(gz))).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut out = Vec::with_capacity(manifest.slugs.len());
    for key in &manifest.slugs {
        let g = goldens
            .remove(key)
            .ok_or_else(|| format!("{}: no golden for {key:?}", file.display()))?;
        if slug(&g.label) != *key {
            return Err(format!("{}: label {:?} does not slugify to its key {key:?}", file.display(), g.label).into());
        }
        out.push((
            Case {
                endpoint: g.endpoint,
                label: g.label,
                path: g.path,
            },
            g.response,
        ));
    }
    if !goldens.is_empty() {
        return Err(format!("{}: {} goldens are not listed in {MANIFEST}", file.display(), goldens.len()).into());
    }
    Ok((manifest.first_day, out))
}

/// Writes `manifest.json` and one gzipped JSON object of goldens keyed by case slug.
fn write_goldens(dir: &Path, bases: &Bases, first_day: NaiveDate, recorded: &[(Case, Response)]) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            std::fs::remove_file(path)?;
        }
    }
    let mut slugs = Vec::with_capacity(recorded.len());
    let mut goldens = BTreeMap::new();
    for (case, response) in recorded {
        let key = slug(&case.label);
        slugs.push(key.clone());
        goldens.insert(
            key,
            Golden {
                endpoint: case.endpoint.clone(),
                label: case.label.clone(),
                path: case.path.clone(),
                response: response.clone(),
            },
        );
    }
    let mut gz = GzEncoder::new(
        std::io::BufWriter::new(std::fs::File::create(dir.join(GOLDENS))?),
        Compression::best(),
    );
    serde_json::to_writer_pretty(&mut gz, &goldens)?;
    gz.write_all(b"\n")?;
    gz.finish()?.flush()?;
    let manifest = Manifest {
        first_day,
        java: bases.java.clone(),
        recorded_at: Local::now().to_rfc3339(),
        cases: slugs.len(),
        slugs,
    };
    std::fs::write(dir.join(MANIFEST), serde_json::to_string_pretty(&manifest)? + "\n")?;
    Ok(())
}

fn md_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

pub async fn run(bases: &Bases, opts: &Options<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let client = api::client();
    let (mode, first_day, mut pairs): (&str, NaiveDate, Vec<(Case, Option<Response>)>) = if opts.golden {
        let (d0, goldens) = load_goldens(opts.golden_dir)?;
        let target = api::probe(&client, &bases.url(Service::Rust, "flights/1")).await;
        if let Some(dep) = target.body.as_ref().and_then(|b| b["departure"].as_str()) {
            if dep.get(..10) != Some(d0.to_string().as_str()) {
                return Err(format!(
                    "goldens in {} were recorded on a schedule starting {d0}, but rust serves one starting {} — \
                     restore the recorded database (scripts/parity.sh --golden) or re-record (scripts/parity.sh --record)",
                    opts.golden_dir.display(),
                    dep.get(..10).unwrap_or(dep)
                )
                .into());
            }
        }
        ("golden", d0, goldens.into_iter().map(|(c, r)| (c, Some(r))).collect())
    } else {
        let f = facts(&client, bases).await;
        let cases = catalogue(&f);
        check_unique(&cases)?;
        (
            if opts.record { "record" } else { "live" },
            f.first_day,
            cases.into_iter().map(|c| (c, None)).collect(),
        )
    };
    if let Some(o) = opts.only {
        pairs.retain(|(c, _)| c.path.contains(o) || c.label.contains(o));
    }
    let java_label = if opts.golden {
        format!("golden {}", opts.golden_dir.display())
    } else {
        bases.java.clone()
    };
    println!(
        "{}",
        bold(&format!(
            "parity  {} cases  java {java_label}  rust {}  first day {first_day}  mode {mode}",
            pairs.len(),
            bases.rust
        ))
    );

    let mut results = Vec::with_capacity(pairs.len());
    let mut recorded = Vec::new();
    for (case, golden) in pairs {
        let ru = bases.url(Service::Rust, &case.path);
        let (j, r): (Response, Response) = match golden {
            Some(g) => (g, api::probe(&client, &ru).await.into()),
            None => {
                let ju = bases.url(Service::Java, &case.path);
                let (j, r) = tokio::join!(api::probe(&client, &ju), api::probe(&client, &ru));
                (j.into(), r.into())
            }
        };
        let res = compare(&case, &j, &r);
        let mark = match res.verdict {
            Verdict::Identical => green("identical"),
            Verdict::Different => red("DIFF     "),
            Verdict::RustDown => yellow("rust down"),
            Verdict::JavaDown => red("java down"),
        };
        let st = |s: u16| if s == 0 { "---".to_string() } else { s.to_string() };
        println!(
            "  {mark}  {:>3} {:>3}  {:<52} {}",
            st(res.java_status),
            st(res.rust_status),
            res.label,
            dim(&res.path)
        );
        if res.verdict == Verdict::Different {
            for d in &res.diffs {
                println!("      {}", dim(d));
            }
        }
        if opts.record {
            recorded.push((case, j));
        }
        results.push(res);
    }

    if opts.record {
        if let Some(down) = recorded.iter().find(|(_, r)| r.error.is_some()) {
            return Err(format!("not recording goldens: java did not answer {:?}", down.0.path).into());
        }
        write_goldens(opts.golden_dir, bases, first_day, &recorded)?;
        println!(
            "{}",
            dim(&format!("recorded {} java goldens → {}", recorded.len(), opts.golden_dir.display()))
        );
    }

    let count = |v: Verdict| results.iter().filter(|r| r.verdict == v).count();
    let (identical, different, rust_down, java_down) = (
        count(Verdict::Identical),
        count(Verdict::Different),
        count(Verdict::RustDown),
        count(Verdict::JavaDown),
    );

    let mut groups: Vec<String> = ENDPOINTS.iter().map(|e| e.to_string()).collect();
    groups.push(UNKNOWN.into());
    for r in &results {
        if !groups.contains(&r.endpoint) {
            groups.push(r.endpoint.clone());
        }
    }
    println!();
    println!(
        "  {:<26} {:>6} {:>10} {:>6} {:>10} {:>10}",
        "endpoint", "cases", "identical", "diff", "rust down", "java down"
    );
    let mut table = String::from("| endpoint | cases | identical | diff | rust down | java down |\n|---|---:|---:|---:|---:|---:|\n");
    for g in &groups {
        let rs: Vec<&CaseResult> = results.iter().filter(|r| &r.endpoint == g).collect();
        if rs.is_empty() {
            continue;
        }
        let c = |v: Verdict| rs.iter().filter(|r| r.verdict == v).count();
        let line = format!(
            "  {:<26} {:>6} {:>10} {:>6} {:>10} {:>10}",
            g,
            rs.len(),
            c(Verdict::Identical),
            c(Verdict::Different),
            c(Verdict::RustDown),
            c(Verdict::JavaDown)
        );
        println!("{}", if c(Verdict::Identical) == rs.len() { line } else { red(&line) });
        table.push_str(&format!(
            "| `{g}` | {} | {} | {} | {} | {} |\n",
            rs.len(),
            c(Verdict::Identical),
            c(Verdict::Different),
            c(Verdict::RustDown),
            c(Verdict::JavaDown)
        ));
    }
    table.push_str(&format!(
        "| **all** | **{}** | **{identical}** | **{different}** | **{rust_down}** | **{java_down}** |\n",
        results.len()
    ));

    let all_ok = identical == results.len();
    let verdict = if all_ok {
        "IDENTICAL".to_string()
    } else if rust_down == results.len() {
        "RUST DOWN".to_string()
    } else {
        "DIFFERENT".to_string()
    };
    let mut md = format!(
        "# Java ↔ Rust parity report\n\n\
         - java: `{java_label}`\n- rust: `{}`\n- mode: {mode}\n- schedule first day: {first_day}\n- generated: {}\n\
         - cases: {}\n- verdict: **{verdict}**\n\n\
         Each case must return the same HTTP status, `Content-Type` and body (key order ignored; values, array order \
         and number text compared exactly; problem bodies compared field by field).\n\n{table}\n## Cases\n\n\
         | # | endpoint | case | path | java | rust | content type (java) | verdict | differences |\n\
         |---:|---|---|---|---:|---:|---|---|---|\n",
        bases.rust,
        Local::now().to_rfc3339(),
        results.len()
    );
    for (i, r) in results.iter().enumerate() {
        md.push_str(&format!(
            "| {} | `{}` | {} | `{}` | {} | {} | `{}` | {} | {} |\n",
            i + 1,
            r.endpoint,
            md_cell(&r.label),
            md_cell(&r.path),
            r.java_status,
            r.rust_status,
            r.java_content_type,
            if r.verdict == Verdict::Identical {
                r.verdict.text().to_string()
            } else {
                format!("**{}**", r.verdict.text())
            },
            md_cell(&r.diffs.join("; "))
        ));
    }
    std::fs::create_dir_all(opts.out)?;
    let report_path: PathBuf = opts.out.join("report.md");
    std::fs::write(&report_path, md)?;
    let json = serde_json::json!({
        "java": java_label,
        "rust": bases.rust,
        "mode": mode,
        "first_day": first_day.to_string(),
        "generated_at": Local::now().to_rfc3339(),
        "cases": results.len(),
        "identical": identical,
        "different": different,
        "rust_down": rust_down,
        "java_down": java_down,
        "verdict": verdict,
        "results": results,
    });
    std::fs::write(opts.out.join("results.json"), serde_json::to_string_pretty(&json)? + "\n")?;

    println!();
    if rust_down == results.len() && !results.is_empty() {
        println!(
            "{}",
            yellow(&format!("rust: down — {} cases not compared ({})", rust_down, bases.rust))
        );
    }
    let summary = format!(
        "parity: {identical}/{} identical, {different} diff, {rust_down} rust down, {java_down} java down  →  {}",
        results.len(),
        report_path.display()
    );
    println!("{}", if all_ok { green(&summary) } else { red(&summary) });
    if all_ok {
        Ok(())
    } else {
        Err(format!("{} of {} cases are not identical", results.len() - identical, results.len()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn facts() -> Facts {
        Facts {
            first_day: NaiveDate::from_ymd_opt(2026, 11, 5).unwrap(),
            last_flight: 3120,
            last_traveler: 50,
            locators: ["QX7K2M", "ABC123", "ZZT9KQ"].iter().map(|s| s.to_string()).collect(),
            sold_out: vec![(0, "BUSINESS".into(), 3), (2, "FIRST".into(), 1)],
        }
    }

    #[test]
    fn slug_is_lowercase_dash_separated() {
        assert_eq!(
            slug("offers NCE-LHR d+0 adults=2 cabin=BUSINESS"),
            "offers-nce-lhr-d-0-adults-2-cabin-business"
        );
        assert_eq!(slug("  airport padded nce "), "airport-padded-nce");
    }

    #[test]
    fn labels_slugify_uniquely_and_every_endpoint_is_covered() {
        let cases = catalogue(&facts());
        check_unique(&cases).unwrap();
        for e in ENDPOINTS {
            assert!(cases.iter().any(|c| c.endpoint == e), "no case for {e}");
        }
        assert!(cases.iter().all(|c| !c.label.contains("2026-11")), "labels must not contain dates");
    }

    #[test]
    fn catalogue_covers_every_case_class() {
        let cases = catalogue(&facts());
        let has = |p: &str| cases.iter().any(|c| c.path == p);
        for adults in [1, 2, 5, 9] {
            assert!(has(&format!("offers?origin=NCE&destination=CDG&date=2026-11-05&adults={adults}")));
        }
        for cabin in ["ECONOMY", "PREMIUM_ECONOMY", "BUSINESS", "FIRST"] {
            assert!(has(&format!(
                "offers?origin=LHR&destination=SIN&date=2026-11-12&adults=9&cabin={cabin}"
            )));
        }
        assert!(has("offers?origin=NCE&destination=LHR&date=2026-11-05&adults=0"));
        assert!(has("offers?origin=NCE&destination=LHR&date=2026-11-05&adults=10"));
        assert!(
            has("offers?origin=NCE&destination=LHR&date=2026-11-05&adults=4&cabin=BUSINESS"),
            "sold out"
        );
        assert!(
            has("offers?origin=NCE&destination=LHR&date=2026-12-05"),
            "outside the 30-day window"
        );
        assert!(has("flights?origin=NCE&destination=LHR&date=2026-12-04"), "last day of the window");
        assert!(has("airports/lhr") && has("airports/ZZZ") && has("flights/3121") && has("travelers/51"));
        assert!(has("bookings/QX7K2M") && has("bookings/ABC123") && has("bookings/ZZZZZZ"));
        assert_eq!(cases.iter().filter(|c| c.path == "bookings/QX7K2M").count(), 1);
        assert!(has("flights?origin=NCE&destination=LHR&date=bad") && has("offers?origin=NCE&destination=LHR"));
        assert!(has("offers?origin=QQQ&destination=LHR&date=2026-11-05"));
    }

    fn resp(status: u16, ct: &str, body: Value) -> Response {
        Response {
            status,
            content_type: ct.into(),
            body: Some(body),
            ..Response::default()
        }
    }

    fn case() -> Case {
        Case {
            endpoint: "airports/{iata}".into(),
            label: "airport missing ZZZ".into(),
            path: "airports/ZZZ".into(),
        }
    }

    #[test]
    fn compare_checks_status_content_type_and_every_problem_field() {
        let pd = "application/problem+json";
        let j = resp(
            404,
            pd,
            json!({"title": "Not Found", "status": 404, "detail": "Airport ZZZ not found", "instance": "/travel/api/airports/ZZZ"}),
        );
        assert_eq!(compare(&case(), &j, &j.clone()).verdict, Verdict::Identical);

        let r = resp(
            404,
            "application/json",
            json!({"type": "about:blank", "title": "Not Found", "status": 404, "detail": "Airport ZZZ not found"}),
        );
        let res = compare(&case(), &j, &r);
        assert_eq!(res.verdict, Verdict::Different);
        assert_eq!(
            res.diffs,
            vec![
                "content-type: java \"application/problem+json\", rust \"application/json\"",
                "problem.type: java absent, rust \"about:blank\"",
                "problem.instance: java \"/travel/api/airports/ZZZ\", rust absent",
            ]
        );
    }

    #[test]
    fn compare_reports_down_sides_and_number_scale() {
        let j = resp(200, "application/json", serde_json::from_str(r#"{"total": 384.00}"#).unwrap());
        let down = Response {
            error: Some("connection refused".into()),
            ..Response::default()
        };
        assert_eq!(compare(&case(), &j, &down).verdict, Verdict::RustDown);
        assert_eq!(compare(&case(), &down, &j).verdict, Verdict::JavaDown);
        let r = resp(200, "application/json", serde_json::from_str(r#"{"total": 384.0}"#).unwrap());
        assert_eq!(compare(&case(), &j, &r).diffs, vec!["$.total: 384.00 != 384.0"]);
    }
    #[test]
    fn goldens_round_trip_through_one_gzipped_file() {
        let dir = std::env::temp_dir().join(format!("j2r-goldens-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("stale-case.json"), "{}").unwrap();
        let bases = Bases {
            java: "http://java".into(),
            rust: "http://rust".into(),
        };
        let first_day = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let recorded = vec![
            (
                case(),
                resp(200, "application/json", serde_json::from_str(r#"{"total": 384.00}"#).unwrap()),
            ),
            (
                Case {
                    endpoint: "airports".into(),
                    label: "airports list".into(),
                    path: "airports".into(),
                },
                Response {
                    status: 500,
                    content_type: "text/plain".into(),
                    text: Some("boom".into()),
                    ..Response::default()
                },
            ),
        ];
        write_goldens(&dir, &bases, first_day, &recorded).unwrap();
        assert!(!dir.join("stale-case.json").exists());
        let (d0, loaded) = load_goldens(&dir).unwrap();
        assert_eq!(d0, first_day);
        assert_eq!(loaded.len(), 2);
        for ((c, r), (lc, lr)) in recorded.iter().zip(&loaded) {
            assert_eq!(c, lc);
            assert!(compare(c, r, lr).diffs.is_empty());
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
