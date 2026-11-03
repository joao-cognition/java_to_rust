//! `j2r parity`: replay a fixed request catalogue covering every openapi.yml operation against the reference
//! (Java) and the target (Rust), then diff status, `Content-Type`, `Location` and the JSON body per case.
//!
//! Writes run in catalogue order after a DB reset before each pass, so both sides see the same seed data and
//! generated ids. `--record` stores the Java pass as golden fixtures; `--golden` checks Rust against them.

use crate::api::{canonical, diff_paths, Bases};
use crate::render::{bold, dim, green, red, svc, yellow};
use reqwest::Method;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct Case {
    pub op: &'static str,
    pub name: &'static str,
    pub method: Method,
    pub path: &'static str,
    pub body: Option<Body>,
}

pub enum Body {
    Json(Value),
    Raw(&'static str),
}

/// Pseudo operation for requests that match no openapi.yml route (unknown path or method).
pub const UNROUTED: &str = "(unrouted)";

fn case(op: &'static str, name: &'static str, method: Method, path: &'static str, body: Option<Body>) -> Case {
    Case {
        op,
        name,
        method,
        path,
        body,
    }
}
fn get(op: &'static str, name: &'static str, path: &'static str) -> Case {
    case(op, name, Method::GET, path, None)
}
fn post(op: &'static str, name: &'static str, path: &'static str, body: Value) -> Case {
    case(op, name, Method::POST, path, Some(Body::Json(body)))
}
fn put(op: &'static str, name: &'static str, path: &'static str, body: Value) -> Case {
    case(op, name, Method::PUT, path, Some(Body::Json(body)))
}
fn delete(op: &'static str, name: &'static str, path: &'static str) -> Case {
    case(op, name, Method::DELETE, path, None)
}
fn unreadable(op: &'static str, method: Method, path: &'static str) -> Case {
    case(op, "unreadable-json-500", method, path, Some(Body::Raw("{\"name\": ")))
}

/// Seed data (data.sql): owners 1-10, pets 1-13, visits 1-4, vets 1-6, specialties 1-3, pet types 1-6.
/// Reads come first, then writes (created ids follow the seed: owner 11, pet 14, visit 5, ...), deletes last.
pub fn catalogue() -> Vec<Case> {
    let owner =
        json!({"firstName": "Ada", "lastName": "Lovelace", "address": "12 Analytical Row", "city": "London", "telephone": "4420794600"});
    let pet = json!({"name": "Babbage", "birthDate": "2020-01-01", "type": {"id": 2, "name": "dog"}});
    vec![
        // reads
        get("failingRequest", "oops", "oops"),
        get("listOwners", "all", "owners"),
        get("listOwners", "by-last-name", "owners?lastName=Davis"),
        get("listOwners", "by-last-name-prefix", "owners?lastName=Da"),
        get("listOwners", "no-match-404", "owners?lastName=Zzz"),
        get("listOwnersPage", "default", "v2/owners"),
        get("listOwnersPage", "page-1-size-3", "v2/owners?page=1&size=3"),
        get("listOwnersPage", "by-last-name", "v2/owners?lastName=Davis"),
        get("listOwnersPage", "past-last-page", "v2/owners?page=99&size=5"),
        get("listOwnersPage", "size-0-500", "v2/owners?size=0"),
        get("listOwnersPage", "non-numeric-page-500", "v2/owners?page=x"),
        get("getOwner", "seed", "owners/1"),
        get("getOwner", "with-pets-and-visits", "owners/6"),
        get("getOwner", "unknown-404", "owners/999"),
        get("getOwner", "non-numeric-500", "owners/abc"),
        get("getOwnersPet", "seed", "owners/6/pets/7"),
        get("getOwnersPet", "other-owners-pet-404", "owners/1/pets/2"),
        get("getOwnersPet", "unknown-owner-404", "owners/999/pets/1"),
        get("getOwnersPet", "non-numeric-500", "owners/1/pets/abc"),
        get("listPetTypes", "all", "pettypes"),
        get("getPetType", "seed", "pettypes/1"),
        get("getPetType", "unknown-404", "pettypes/999"),
        get("getPetType", "non-numeric-500", "pettypes/abc"),
        get("listPets", "all", "pets"),
        get("listPetsPage", "default", "v2/pets"),
        get("listPetsPage", "page-2-size-5", "v2/pets?page=2&size=5"),
        get("listPetsPage", "size-0-500", "v2/pets?size=0"),
        get("getPet", "with-visits", "pets/7"),
        get("getPet", "unknown-404", "pets/999"),
        get("getPet", "non-numeric-500", "pets/abc"),
        get("listVisits", "all", "visits"),
        get("getVisit", "seed", "visits/1"),
        get("getVisit", "unknown-404", "visits/999"),
        get("getVisit", "non-numeric-500", "visits/abc"),
        get("listSpecialties", "all", "specialties"),
        get("getSpecialty", "seed", "specialties/1"),
        get("getSpecialty", "unknown-404", "specialties/999"),
        get("getSpecialty", "non-numeric-500", "specialties/abc"),
        get("listVets", "all", "vets"),
        get("getVet", "no-specialties", "vets/1"),
        get("getVet", "two-specialties", "vets/3"),
        get("getVet", "unknown-404", "vets/999"),
        get("getVet", "non-numeric-500", "vets/abc"),
        get(UNROUTED, "unknown-path-500", "nope"),
        get(UNROUTED, "unknown-subpath-500", "owners/1/nope"),
        delete(UNROUTED, "unsupported-method-500", "owners"),
        // owners
        post("addOwner", "create", "owners", owner.clone()),
        post(
            "addOwner",
            "invalid-fields-400",
            "owners",
            json!({"firstName": "", "lastName": "x1", "address": "a", "city": "c", "telephone": "abc"}),
        ),
        post("addOwner", "empty-body-400", "owners", json!({})),
        post(
            "addOwner",
            "entity-telephone-500",
            "owners",
            json!({"firstName": "Ann", "lastName": "Lee", "address": "1 St", "city": "X", "telephone": "123"}),
        ),
        unreadable("addOwner", Method::POST, "owners"),
        get("getOwner", "created", "owners/11"),
        put(
            "updateOwner",
            "update",
            "owners/11",
            json!({"firstName": "Ada", "lastName": "King", "address": "1 Ockham Park", "city": "Surrey", "telephone": "4414830000"}),
        ),
        put("updateOwner", "unknown-404", "owners/999", owner.clone()),
        put(
            "updateOwner",
            "invalid-fields-400",
            "owners/11",
            json!({"firstName": "Ada", "lastName": "", "address": "a", "city": "c", "telephone": "1"}),
        ),
        put("updateOwner", "non-numeric-500", "owners/abc", owner.clone()),
        unreadable("updateOwner", Method::PUT, "owners/11"),
        // pets of an owner
        post("addPetToOwner", "create", "owners/11/pets", pet.clone()),
        post(
            "addPetToOwner",
            "future-birth-date-400",
            "owners/11/pets",
            json!({"name": "Rex", "birthDate": "2999-01-01", "type": {"id": 2, "name": "dog"}}),
        ),
        post(
            "addPetToOwner",
            "birth-date-over-50y-400",
            "owners/11/pets",
            json!({"name": "Rex", "birthDate": "1900-01-01", "type": {"id": 2, "name": "dog"}}),
        ),
        post(
            "addPetToOwner",
            "missing-fields-400",
            "owners/11/pets",
            json!({"type": {"id": 2, "name": "dog"}}),
        ),
        post("addPetToOwner", "unknown-owner-404", "owners/999/pets", pet.clone()),
        unreadable("addPetToOwner", Method::POST, "owners/11/pets"),
        get("getOwnersPet", "created", "owners/11/pets/14"),
        put(
            "updateOwnersPet",
            "update",
            "owners/11/pets/14",
            json!({"name": "Charles", "birthDate": "2019-06-15", "type": {"id": 1, "name": "cat"}}),
        ),
        put("updateOwnersPet", "unknown-owner-404", "owners/999/pets/14", pet.clone()),
        put("updateOwnersPet", "unknown-pet-404", "owners/11/pets/999", pet.clone()),
        put(
            "updateOwnersPet",
            "invalid-fields-400",
            "owners/11/pets/14",
            json!({"name": "", "birthDate": "2999-01-01", "type": {"id": 1, "name": "cat"}}),
        ),
        post(
            "addVisitToOwner",
            "create",
            "owners/11/pets/14/visits",
            json!({"date": "2024-05-01", "description": "annual checkup"}),
        ),
        post(
            "addVisitToOwner",
            "missing-description-400",
            "owners/11/pets/14/visits",
            json!({"date": "2024-05-01"}),
        ),
        post(
            "addVisitToOwner",
            "unknown-pet-404",
            "owners/11/pets/999/visits",
            json!({"date": "2024-05-01", "description": "x"}),
        ),
        unreadable("addVisitToOwner", Method::POST, "owners/11/pets/14/visits"),
        // pet types
        post("addPetType", "create", "pettypes", json!({"name": "hamster"})),
        post("addPetType", "empty-name-400", "pettypes", json!({"name": ""})),
        unreadable("addPetType", Method::POST, "pettypes"),
        get("getPetType", "created", "pettypes/7"),
        put("updatePetType", "update", "pettypes/7", json!({"id": 7, "name": "gerbil"})),
        put("updatePetType", "unknown-404", "pettypes/999", json!({"id": 999, "name": "gerbil"})),
        put("updatePetType", "empty-name-400", "pettypes/7", json!({"id": 7, "name": ""})),
        // specialties
        post("addSpecialty", "create", "specialties", json!({"name": "cardiology"})),
        post("addSpecialty", "empty-name-400", "specialties", json!({"name": ""})),
        unreadable("addSpecialty", Method::POST, "specialties"),
        get("getSpecialty", "created", "specialties/4"),
        put("updateSpecialty", "update", "specialties/4", json!({"id": 4, "name": "oncology"})),
        put(
            "updateSpecialty",
            "unknown-404",
            "specialties/999",
            json!({"id": 999, "name": "oncology"}),
        ),
        put("updateSpecialty", "empty-name-400", "specialties/4", json!({"id": 4, "name": ""})),
        // vets
        post(
            "addVet",
            "create",
            "vets",
            json!({"firstName": "Grace", "lastName": "Hopper", "specialties": [{"id": 4, "name": "oncology"}]}),
        ),
        post(
            "addVet",
            "invalid-name-400",
            "vets",
            json!({"firstName": "Grace1", "lastName": "", "specialties": []}),
        ),
        unreadable("addVet", Method::POST, "vets"),
        get("getVet", "created", "vets/7"),
        put(
            "updateVet",
            "update",
            "vets/7",
            json!({"firstName": "Grace", "lastName": "Brewster", "specialties": [{"id": 1, "name": "radiology"}, {"id": 2, "name": "surgery"}]}),
        ),
        put(
            "updateVet",
            "unknown-404",
            "vets/999",
            json!({"firstName": "Grace", "lastName": "Hopper", "specialties": []}),
        ),
        put(
            "updateVet",
            "invalid-name-400",
            "vets/7",
            json!({"firstName": "", "lastName": "Hopper", "specialties": []}),
        ),
        // visits
        post(
            "addVisit",
            "create",
            "visits",
            json!({"date": "2024-06-01", "description": "vaccination", "petId": 14}),
        ),
        post(
            "addVisit",
            "missing-date-400",
            "visits",
            json!({"description": "vaccination", "petId": 14}),
        ),
        unreadable("addVisit", Method::POST, "visits"),
        // visit 6 was burned by the failed unknown-pet insert above (identity value consumed, row rolled back)
        get("getVisit", "created", "visits/7"),
        put(
            "updateVisit",
            "update",
            "visits/7",
            json!({"date": "2024-06-02", "description": "booster"}),
        ),
        put(
            "updateVisit",
            "unknown-404",
            "visits/999",
            json!({"date": "2024-06-02", "description": "booster"}),
        ),
        put(
            "updateVisit",
            "empty-description-400",
            "visits/7",
            json!({"date": "2024-06-02", "description": ""}),
        ),
        // pets
        put(
            "updatePet",
            "update",
            "pets/14",
            json!({"id": 14, "name": "Ada", "birthDate": "2021-02-03", "type": {"id": 3, "name": "lizard"}, "visits": []}),
        ),
        put(
            "updatePet",
            "unknown-404",
            "pets/999",
            json!({"id": 999, "name": "Ada", "birthDate": "2021-02-03", "type": {"id": 3, "name": "lizard"}, "visits": []}),
        ),
        put(
            "updatePet",
            "future-birth-date-400",
            "pets/14",
            json!({"id": 14, "name": "Ada", "birthDate": "2999-01-01", "type": {"id": 3, "name": "lizard"}, "visits": []}),
        ),
        get("getPet", "updated", "pets/14"),
        // users
        post(
            "addUser",
            "create",
            "users",
            json!({"username": "parity", "password": "secret", "enabled": true, "roles": [{"name": "OWNER_ADMIN"}]}),
        ),
        post(
            "addUser",
            "missing-username-400",
            "users",
            json!({"password": "secret", "enabled": true}),
        ),
        unreadable("addUser", Method::POST, "users"),
        // state after the writes
        get("listOwnersPage", "after-writes", "v2/owners?lastName=King"),
        get("listVisits", "after-writes", "visits"),
        // deletes
        delete("deleteVisit", "delete", "visits/7"),
        delete("deleteVisit", "already-deleted-404", "visits/7"),
        delete("deleteVisit", "non-numeric-500", "visits/abc"),
        delete("deletePet", "delete-with-visits", "pets/14"),
        delete("deletePet", "unknown-404", "pets/999"),
        delete("deleteOwner", "delete", "owners/11"),
        delete("deleteOwner", "unknown-404", "owners/999"),
        delete("deleteOwner", "seed-with-pets-404", "owners/1"),
        delete("deleteVet", "delete", "vets/7"),
        delete("deleteVet", "unknown-404", "vets/999"),
        delete("deleteSpecialty", "delete", "specialties/4"),
        delete("deleteSpecialty", "unknown-404", "specialties/999"),
        delete("deletePetType", "delete", "pettypes/7"),
        delete("deletePetType", "unknown-404", "pettypes/999"),
        delete("deletePetType", "in-use", "pettypes/2"),
        get("listPets", "after-deletes", "pets"),
    ]
}

/// What a case is compared on. JSON form doubles as the golden fixture's `response`.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub status: u16,
    pub content_type: Option<String>,
    pub location: Option<String>,
    /// Parsed and normalized JSON body; `None` for an empty body.
    pub body: Option<Value>,
    /// Raw body when it is not JSON.
    pub text: Option<String>,
}

impl Outcome {
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("status".into(), json!(self.status));
        m.insert(
            "headers".into(),
            json!({"content-type": self.content_type, "location": self.location}),
        );
        if let Some(b) = &self.body {
            m.insert("body".into(), b.clone());
        }
        if let Some(t) = &self.text {
            m.insert("text".into(), json!(t));
        }
        Value::Object(m)
    }

    pub fn from_json(v: &Value) -> Option<Outcome> {
        let h = v.get("headers")?;
        let s = |k: &str| h.get(k).and_then(Value::as_str).map(String::from);
        Some(Outcome {
            status: v.get("status")?.as_u64()? as u16,
            content_type: s("content-type"),
            location: s("location"),
            body: v.get("body").cloned(),
            text: v.get("text").and_then(Value::as_str).map(String::from),
        })
    }
}

/// Make two runs comparable without hiding real differences:
/// * ProblemDetail `timestamp`: replaced by `<timestamp>` only when it is a valid RFC 3339 instant;
/// * ProblemDetail `type`: the request URL, so the service's own origin is replaced by `{origin}`;
/// * ProblemDetail `schemaValidationErrors`: sorted, since Hibernate Validator reports violations in hash order.
pub fn normalize(mut body: Value, origin: &str) -> Value {
    if let Value::Object(m) = &mut body {
        if m.contains_key("schemaValidationErrors") {
            if let Some(Value::String(ts)) = m.get("timestamp") {
                if chrono::DateTime::parse_from_rfc3339(ts).is_ok() {
                    m.insert("timestamp".into(), json!("<timestamp>"));
                }
            }
            if let Some(Value::String(t)) = m.get("type") {
                if let Some(rest) = t.strip_prefix(origin) {
                    let t = format!("{{origin}}{rest}");
                    m.insert("type".into(), json!(t));
                }
            }
            if let Some(Value::Array(errs)) = m.get_mut("schemaValidationErrors") {
                errs.sort_by_key(canonical);
            }
        }
    }
    body
}

/// `scheme://host[:port]` of a base URL.
pub fn origin(base: &str) -> String {
    match base.find("://") {
        Some(i) => {
            let rest = &base[i + 3..];
            let end = rest.find('/').map(|j| i + 3 + j).unwrap_or(base.len());
            base[..end].to_string()
        }
        None => base.to_string(),
    }
}

async fn send(client: &reqwest::Client, base: &str, c: &Case) -> Result<Outcome, String> {
    let url = format!("{}/{}", base, c.path);
    let mut req = client.request(c.method.clone(), &url).header("accept", "application/json");
    match &c.body {
        Some(Body::Json(v)) => req = req.header("content-type", "application/json").body(v.to_string()),
        Some(Body::Raw(s)) => req = req.header("content-type", "application/json").body(*s),
        None => {}
    }
    let res = req.send().await.map_err(|e| {
        if e.is_connect() {
            format!("{url}: connection refused — nothing listening")
        } else {
            format!("{url}: {e}")
        }
    })?;
    let header = |name: &str| res.headers().get(name).and_then(|v| v.to_str().ok()).map(String::from);
    let (content_type, location) = (header("content-type"), header("location"));
    let status = res.status().as_u16();
    let text = res.text().await.map_err(|e| format!("{url}: body read failed: {e}"))?;
    let (body, text) = if text.is_empty() {
        (None, None)
    } else {
        match serde_json::from_str::<Value>(&text) {
            Ok(v) => (Some(normalize(v, &origin(base))), None),
            Err(_) => (None, Some(text)),
        }
    };
    Ok(Outcome {
        status,
        content_type,
        location,
        body,
        text,
    })
}

fn reset(cmd: Option<&str>) -> Result<(), String> {
    let Some(cmd) = cmd else { return Ok(()) };
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("reset `{cmd}`: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "reset `{cmd}` failed ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

/// One pass of the whole catalogue against one base, after a DB reset.
async fn pass(base: &str, cases: &[Case], reset_cmd: Option<&str>) -> Result<Vec<Outcome>, String> {
    reset(reset_cmd)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(cases.len());
    for c in cases {
        out.push(send(&client, base, c).await?);
    }
    Ok(out)
}

/// `(operationId, METHOD, path)` for every operation in openapi.yml, in file order.
pub fn spec_operations(yaml: &str) -> Vec<(String, String, String)> {
    let (mut path, mut method) = (String::new(), String::new());
    let mut ops = Vec::new();
    for line in yaml.lines() {
        if let Some(p) = line.strip_prefix("  /") {
            path = format!("/{}", p.trim_end().trim_end_matches(':'));
        } else if let Some(m) = ["get", "post", "put", "delete", "patch"]
            .iter()
            .find(|m| line.trim_end() == format!("    {m}:"))
        {
            method = m.to_uppercase();
        } else if let Some(id) = line.trim().strip_prefix("operationId:") {
            ops.push((id.trim().to_string(), method.clone(), path.clone()));
        }
    }
    ops
}

fn fixture_name(seq: usize, c: &Case) -> String {
    let op = if c.op == UNROUTED { "unrouted" } else { c.op };
    format!("{:03}-{op}-{}.json", seq + 1, c.name)
}

/// `NNN-<op>-<case>.json`, as written by `fixture_name`.
fn is_fixture_name(n: &str) -> bool {
    let b = n.as_bytes();
    n.ends_with(".json") && b.len() > 9 && b[..3].iter().all(u8::is_ascii_digit) && b[3] == b'-'
}

/// Whether a case path (no leading `/`, optional query) is an instance of an openapi path template.
pub fn matches_route(template: &str, path: &str) -> bool {
    let path = path.split('?').next().unwrap_or_default();
    let t: Vec<&str> = template.trim_start_matches('/').split('/').collect();
    let p: Vec<&str> = path.split('/').collect();
    t.len() == p.len()
        && t.iter()
            .zip(&p)
            .all(|(t, p)| t == p || (t.starts_with('{') && t.ends_with('}') && !p.is_empty()))
}

fn request_json(c: &Case) -> Value {
    json!({
        "method": c.method.as_str(),
        "path": c.path,
        "body": match &c.body {
            Some(Body::Json(v)) => v.clone(),
            Some(Body::Raw(s)) => json!(s),
            None => Value::Null,
        },
    })
}

fn record(dir: &Path, cases: &[Case], outcomes: &[Outcome]) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.file_name().and_then(|n| n.to_str()).is_some_and(is_fixture_name) {
            std::fs::remove_file(p)?;
        }
    }
    for (i, (c, o)) in cases.iter().zip(outcomes).enumerate() {
        let v = json!({
            "operation": c.op,
            "case": c.name,
            "request": request_json(c),
            "response": o.to_json(),
        });
        std::fs::write(dir.join(fixture_name(i, c)), serde_json::to_string_pretty(&v)? + "\n")?;
    }
    Ok(())
}

fn load_golden(dir: &Path, cases: &[Case]) -> Result<Vec<Option<Outcome>>, Box<dyn std::error::Error>> {
    cases
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let p = dir.join(fixture_name(i, c));
            if !p.exists() {
                return Ok(None);
            }
            let v: Value = serde_json::from_str(&std::fs::read_to_string(&p)?)?;
            if v.get("request") != Some(&request_json(c)) {
                return Err(format!("{}: request differs from the catalogue, re-record the fixtures", p.display()).into());
            }
            Ok(v.get("response").and_then(Outcome::from_json))
        })
        .collect()
}

pub struct Opts {
    pub record: Option<PathBuf>,
    pub golden: Option<PathBuf>,
    pub reset: Option<String>,
    pub spec: PathBuf,
    pub verbose: bool,
}

pub async fn run(bases: &Bases, opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let cases = catalogue();
    let reset_cmd = opts.reset.as_deref();
    println!(
        "{}",
        dim(&format!(
            "{} cases · reset before each pass: {}",
            cases.len(),
            reset_cmd.unwrap_or("off")
        ))
    );

    if let Some(dir) = &opts.record {
        println!("{} {}", bold(&svc("java")), dim(&bases.java));
        let outcomes = pass(&bases.java, &cases, reset_cmd).await?;
        record(dir, &cases, &outcomes)?;
        println!(
            "{}",
            green(&format!("recorded {} golden responses in {}", outcomes.len(), dir.display()))
        );
        coverage(&opts.spec, &cases, None);
        return Ok(());
    }

    let (left_name, left): (String, Vec<Option<Outcome>>) = match &opts.golden {
        Some(dir) => {
            println!("{} {}", bold("golden"), dim(&dir.display().to_string()));
            (format!("golden {}", dir.display()), load_golden(dir, &cases)?)
        }
        None => {
            println!("{} {}", bold(&svc("java")), dim(&bases.java));
            let o = pass(&bases.java, &cases, reset_cmd).await?;
            (bases.java.clone(), o.into_iter().map(Some).collect())
        }
    };
    println!("{} {}", bold(&svc("rust")), dim(&bases.rust));
    let right = pass(&bases.rust, &cases, reset_cmd).await.map_err(|e| format!("rust: {e}"))?;
    println!();

    let mut same = vec![false; cases.len()];
    for (i, c) in cases.iter().enumerate() {
        let r = &right[i];
        let label = format!("{:<6} /{}", c.method.as_str(), c.path);
        let id = format!("{}  {}", c.op, dim(c.name));
        match &left[i] {
            None => println!("{} {label}  {id}  {}", red("✗"), red("no golden fixture")),
            Some(l) => {
                let (a, b) = (l.to_json(), r.to_json());
                if canonical(&a) == canonical(&b) {
                    same[i] = true;
                    if opts.verbose {
                        println!("{} {label}  {id}  {}", green("✓"), dim(&r.status.to_string()));
                    }
                } else {
                    let mut out = Vec::new();
                    diff_paths(&a, &b, "$", &mut out, 8);
                    println!("{} {label}  {id}", red("✗"));
                    for d in out {
                        println!("    {d}");
                    }
                }
            }
        }
    }

    coverage(&opts.spec, &cases, Some(&same));
    let ok = same.iter().filter(|s| **s).count();
    let pct = 100.0 * ok as f64 / cases.len() as f64;
    let line = format!("{ok}/{} identical ({pct:.1}%) — {left_name} vs {}", cases.len(), bases.rust);
    println!();
    if ok == cases.len() {
        println!("{}", green(&line));
        Ok(())
    } else {
        println!("{}", yellow(&line));
        Err(format!("{} case(s) differ", cases.len() - ok).into())
    }
}

/// Case count (and identical count) per openapi.yml operation; operations without cases are listed as gaps.
fn coverage(spec: &Path, cases: &[Case], same: Option<&[bool]>) {
    let ops = match std::fs::read_to_string(spec) {
        Ok(y) => spec_operations(&y),
        Err(e) => {
            println!("{}", yellow(&format!("coverage: cannot read {}: {e}", spec.display())));
            Vec::new()
        }
    };
    let mut per: BTreeMap<&str, (usize, usize, Vec<u16>)> = BTreeMap::new();
    for (i, c) in cases.iter().enumerate() {
        if let Some((_, m, t)) = ops.iter().find(|(id, _, _)| id == c.op) {
            if m != c.method.as_str() || !matches_route(t, c.path) {
                println!(
                    "{}",
                    yellow(&format!("{} {}: {} {} is not {m} {t}, not counted", c.op, c.name, c.method, c.path))
                );
                continue;
            }
        }
        let e = per.entry(c.op).or_default();
        e.0 += 1;
        if same.is_some_and(|s| s[i]) {
            e.1 += 1;
        }
    }
    let mut rows: Vec<(String, String, String)> = ops.clone();
    for c in cases {
        if !rows.iter().any(|(id, _, _)| id == c.op) {
            rows.push((c.op.to_string(), String::new(), String::new()));
        }
    }
    println!();
    println!(
        "{}",
        bold(&format!(
            "{:<18} {:<7} {:<36} {:>5}{}",
            "operation",
            "method",
            "path",
            "cases",
            if same.is_some() { "  identical" } else { "" }
        ))
    );
    let mut gaps = 0;
    for (id, method, path) in &rows {
        let (n, ok, _) = per.get(id.as_str()).cloned().unwrap_or_default();
        let counts = match same {
            Some(_) => format!("{n:>5}  {:>9}", format!("{ok}/{n}")),
            None => format!("{n:>5}"),
        };
        let line = format!("{id:<18} {method:<7} {path:<36} {counts}");
        if n == 0 {
            gaps += 1;
            println!("{}  {}", red(&line), red("no cases"));
        } else if same.is_some() && ok < n {
            println!("{}", yellow(&line));
        } else {
            println!("{line}");
        }
    }
    let unknown: Vec<&str> = per
        .keys()
        .filter(|k| **k != UNROUTED && !ops.iter().any(|(id, _, _)| id == *k))
        .copied()
        .collect();
    let summary = format!(
        "{} operations in {}, {} without cases{}",
        ops.len(),
        spec.display(),
        gaps,
        if unknown.is_empty() {
            String::new()
        } else {
            format!(", not in spec: {}", unknown.join(", "))
        }
    );
    println!(
        "{}",
        if gaps == 0 && unknown.is_empty() {
            dim(&summary)
        } else {
            red(&summary)
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_masks_volatile_problem_detail_fields_only() {
        let v = json!({
            "type": "http://localhost:9966/petclinic/api/owners",
            "timestamp": "2026-11-03T21:58:58.569746688Z",
            "schemaValidationErrors": [{"message": "b"}, {"message": "a"}],
        });
        let n = normalize(v, "http://localhost:9966");
        assert_eq!(
            n,
            json!({
                "type": "{origin}/petclinic/api/owners",
                "timestamp": "<timestamp>",
                "schemaValidationErrors": [{"message": "a"}, {"message": "b"}],
            })
        );
        let bad = normalize(json!({"timestamp": "yesterday", "schemaValidationErrors": []}), "x");
        assert_eq!(bad["timestamp"], "yesterday");
        let dto = json!({"timestamp": "2026-11-03T21:58:58Z", "type": "http://localhost:9966/x"});
        assert_eq!(normalize(dto.clone(), "http://localhost:9966"), dto);
    }

    #[test]
    fn routes_and_fixture_names() {
        assert!(matches_route("/owners/{ownerId}/pets/{petId}", "owners/1/pets/abc"));
        assert!(matches_route("/v2/owners", "v2/owners?page=1"));
        assert!(!matches_route("/owners/{ownerId}", "owners"));
        assert!(!matches_route("/owners/{ownerId}", "pets/1"));
        assert!(is_fixture_name("001-getOwner-seed.json"));
        assert!(!is_fixture_name("notes.json"));
    }

    #[test]
    fn origin_strips_path() {
        assert_eq!(origin("http://localhost:9977/petclinic/api"), "http://localhost:9977");
        assert_eq!(origin("http://h:1"), "http://h:1");
    }

    #[test]
    fn spec_operations_parses_paths_methods_ids() {
        let y = "paths:\n  /owners/{ownerId}:\n    get:\n      operationId: getOwner\n    put:\n      operationId: updateOwner\n";
        assert_eq!(
            spec_operations(y),
            vec![
                ("getOwner".into(), "GET".into(), "/owners/{ownerId}".into()),
                ("updateOwner".into(), "PUT".into(), "/owners/{ownerId}".into()),
            ]
        );
    }

    #[test]
    fn catalogue_covers_every_spec_operation_with_unique_names() {
        let ops = spec_operations(include_str!("../../src/main/resources/openapi.yml"));
        assert_eq!(ops.len(), 37);
        let cases = catalogue();
        for (id, _, _) in &ops {
            assert!(cases.iter().any(|c| c.op == id), "no case for {id}");
        }
        for c in &cases {
            if let Some((_, m, t)) = ops.iter().find(|(id, _, _)| id == c.op) {
                assert!(
                    m == c.method.as_str() && matches_route(t, c.path),
                    "{} {}: not {m} {t}",
                    c.op,
                    c.name
                );
            }
        }
        let mut names: Vec<String> = cases.iter().enumerate().map(|(i, c)| fixture_name(i, c)).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), cases.len());
        let mut keys: Vec<(&str, &str)> = cases.iter().map(|c| (c.op, c.name)).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), cases.len(), "duplicate (operation, case)");
    }
}
