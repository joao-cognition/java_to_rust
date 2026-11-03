//! `j2r bench` / `j2r watch` / `j2r show`: fixed-rate load with per-second samples of the *server* process.

use crate::api::{self, Bases};
use crate::render::{self, bold, dim, green, red, strip_ansi, svc, yellow};
use crate::sampler::{Source, Sources};
use crate::Service;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

pub struct Plan {
    pub rate: u32,
    pub duration: u64,
    pub peak: u64,
    pub only: Option<Service>,
}

/// Request mix of the read hot path: mostly record lookups and searches, some reference lists.
const MIX: &[&str] = &[
    "owners/1",
    "owners/1",
    "owners/1",
    "owners/1",
    "owners?lastName=Davis",
    "owners?lastName=Davis",
    "vets",
    "specialties",
    "pettypes",
    "pets/1",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub t: u64,
    pub cpu_cores: Option<f64>,
    pub rps: f64,
    pub p50_ms: f64,
    pub p99_ms: f64,
    pub rss_mb: Option<f64>,
    pub errors: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSeries {
    pub label: String,
    pub source: String,
    pub samples: Vec<Sample>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub cpu_cores: Option<f64>,
    #[serde(default)]
    pub rps: f64,
    pub p50_ms: f64,
    pub p99_ms: f64,
    pub rss_mb: Option<f64>,
    pub peak_rps: Option<f64>,
    pub errors: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Timeseries {
    pub measured_at: String,
    pub host: String,
    pub rate_per_second: u32,
    pub duration_seconds: u64,
    pub peak_seconds: u64,
    pub mix: Vec<String>,
    pub services: BTreeMap<String, ServiceSeries>,
    pub summary: BTreeMap<String, Summary>,
}

pub async fn run(bases: &Bases, sources: &Sources, plan: Plan, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = api::client();
    let mut ts = Timeseries {
        measured_at: chrono::Local::now().to_rfc3339(),
        host: host_label(),
        rate_per_second: plan.rate,
        duration_seconds: plan.duration,
        peak_seconds: plan.peak,
        mix: MIX.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    };
    let mut log: Vec<String> = Vec::new();
    let mut say = |line: String| {
        println!("{line}");
        log.push(strip_ansi(&line));
    };

    say(format!(
        "$ j2r bench --rate {} --duration {} --peak {}",
        plan.rate, plan.duration, plan.peak
    ));
    say(dim(&format!("# {} · {}", ts.measured_at, ts.host)));
    say(dim(&format!("# mix: {}", MIX.join(" "))));

    for service in [Service::Java, Service::Rust] {
        if plan.only.is_some_and(|o| o != service) {
            continue;
        }
        let name = service.name();
        let base = bases.url(service, "");
        let source = sources.get(service);
        let health = api::probe(&client, &bases.url(service, "pettypes")).await;
        if !health.ok() {
            say(format!("[{}] {} {} — skipped", svc(name), red("down"), dim(&base)));
            continue;
        }
        say(format!(
            "[{}] {} {} · {}",
            svc(name),
            green("up"),
            dim(&base),
            dim(&source.describe())
        ));
        if !source.known() {
            say(yellow("# cpu/rss will be n/a for this side"));
        }

        // warm-up: 2 s at the target rate, not recorded (JIT, connection pools)
        let _ = fixed_rate_second(&client, &base, plan.rate, source, 0).await;
        let _ = fixed_rate_second(&client, &base, plan.rate, source, 0).await;

        let mut series = ServiceSeries {
            label: name.to_string(),
            source: source.describe(),
            samples: Vec::new(),
        };
        for t in 1..=plan.duration {
            let s = fixed_rate_second(&client, &base, plan.rate, source, t).await;
            say(format!(
                "[{}] t={:>3}s cpu={} rps={:>5.0} p50={:>5.1}ms p99={:>6.1}ms rss={}{}",
                svc(name),
                s.t,
                s.cpu_cores.map(|c| format!("{c:.2} cores")).unwrap_or_else(|| "  n/a    ".into()),
                s.rps,
                s.p50_ms,
                s.p99_ms,
                s.rss_mb.map(|m| format!("{m:.0}MB")).unwrap_or_else(|| "n/a".into()),
                if s.errors > 0 {
                    red(&format!(" errors={}", s.errors))
                } else {
                    String::new()
                }
            ));
            series.samples.push(s);
        }

        let mut summary = summarize(&series.samples);
        if plan.peak > 0 {
            let (rps, p99) = peak_phase(&client, &base, plan.peak).await;
            say(format!(
                "[{}] peak: {} req/s over {}s, 64 in flight, p99={:.1}ms",
                svc(name),
                bold(&format!("{rps:.0}")),
                plan.peak,
                p99
            ));
            summary.peak_rps = Some(rps);
        }
        ts.services.insert(name.to_string(), series);
        ts.summary.insert(name.to_string(), summary);
    }

    println!();
    let rendered = render_results(&ts);
    println!("{rendered}");
    log.push(String::new());
    log.push(strip_ansi(&rendered));

    std::fs::create_dir_all(out)?;
    std::fs::write(out.join("timeseries.json"), serde_json::to_string_pretty(&ts)?)?;
    std::fs::write(out.join("summary.json"), serde_json::to_string_pretty(&ts.summary)?)?;
    std::fs::write(out.join("run.log"), log.join("\n") + "\n")?;
    println!(
        "{}",
        dim(&format!("# wrote {}/{{timeseries.json,summary.json,run.log}}", out.display()))
    );
    Ok(())
}

/// One second of open-loop load at `rate` req/s; CPU and RSS deltas of the server process over the same wall time.
async fn fixed_rate_second(client: &reqwest::Client, base: &str, rate: u32, source: &Source, t: u64) -> Sample {
    let snap0 = source.snapshot();
    let t0 = Instant::now();
    let mut set = JoinSet::new();
    let mut tick = tokio::time::interval(Duration::from_secs_f64(1.0 / rate as f64));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
    for i in 0..rate as usize {
        tick.tick().await;
        let url = format!("{base}{}", MIX[(t as usize * rate as usize + i) % MIX.len()]);
        let c = client.clone();
        set.spawn(async move {
            let p = api::probe(&c, &url).await;
            (p.ok(), p.ms)
        });
    }
    let mut lat = Vec::with_capacity(rate as usize);
    let mut errors = 0u64;
    while let Some(r) = set.join_next().await {
        match r {
            Ok((true, ms)) => lat.push(ms),
            _ => errors += 1,
        }
    }
    let wall = t0.elapsed().as_secs_f64();
    let snap1 = source.snapshot();
    let (cpu_cores, rss_mb) = match (snap0, snap1) {
        (Some((c0, _)), Some((c1, m1))) => (Some(((c1 - c0) / wall).max(0.0)), Some(m1)),
        _ => (None, None),
    };
    Sample {
        t,
        cpu_cores,
        rps: lat.len() as f64 / wall,
        p50_ms: api::percentile(&lat, 0.5),
        p99_ms: api::percentile(&lat, 0.99),
        rss_mb,
        errors,
    }
}

/// Closed loop, 64 workers, as fast as the server answers: peak throughput and its p99.
async fn peak_phase(client: &reqwest::Client, base: &str, seconds: u64) -> (f64, f64) {
    let done = Arc::new(AtomicU64::new(0));
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut set = JoinSet::new();
    for w in 0..64usize {
        let c = client.clone();
        let base = base.to_string();
        let done = done.clone();
        set.spawn(async move {
            let mut lat = Vec::new();
            let mut i = w;
            while Instant::now() < deadline {
                let p = api::probe(&c, &format!("{base}{}", MIX[i % MIX.len()])).await;
                if p.ok() && Instant::now() <= deadline {
                    lat.push(p.ms);
                    done.fetch_add(1, Ordering::Relaxed);
                }
                i += 64;
            }
            lat
        });
    }
    let mut all = Vec::new();
    while let Some(Ok(l)) = set.join_next().await {
        all.extend(l);
    }
    (
        done.load(Ordering::Relaxed) as f64 / seconds.max(1) as f64,
        api::percentile(&all, 0.99),
    )
}

fn summarize(samples: &[Sample]) -> Summary {
    let n = samples.len().max(1) as f64;
    let mean = |f: &dyn Fn(&Sample) -> Option<f64>| {
        let v: Vec<f64> = samples.iter().filter_map(f).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    };
    Summary {
        cpu_cores: mean(&|s| s.cpu_cores),
        rps: samples.iter().map(|s| s.rps).sum::<f64>() / n,
        p50_ms: samples.iter().map(|s| s.p50_ms).sum::<f64>() / n,
        p99_ms: samples.iter().map(|s| s.p99_ms).sum::<f64>() / n,
        rss_mb: mean(&|s| s.rss_mb),
        peak_rps: None,
        errors: samples.iter().map(|s| s.errors).sum(),
    }
}

/// (label, unit, extractor, decimals)
type Metric = (&'static str, &'static str, fn(&Sample) -> Option<f64>, usize);

pub fn render_results(ts: &Timeseries) -> String {
    let width = render::term_width().clamp(60, 140);
    let spark_w = width.saturating_sub(24).clamp(20, 60);
    let mut out = String::new();
    let metrics: [Metric; 4] = [
        ("cpu", "cores", |s| s.cpu_cores, 2),
        ("rps", "req/s", |s| Some(s.rps), 0),
        ("p99", "ms", |s| Some(s.p99_ms), 1),
        ("rss", "MB", |s| s.rss_mb, 0),
    ];
    for (label, unit, f, decimals) in metrics {
        let values = |k: &str| -> Vec<f64> {
            ts.services
                .get(k)
                .map(|s| s.samples.iter().filter_map(f).collect())
                .unwrap_or_default()
        };
        let (jv, rv) = (values("java"), values("rust"));
        let max = jv.iter().chain(rv.iter()).cloned().fold(0.0, f64::max);
        if max == 0.0 {
            continue;
        }
        out.push_str(&format!("{} {}\n", bold(label), dim(unit)));
        for (k, v) in [("java", &jv), ("rust", &rv)] {
            let avg = if v.is_empty() {
                "n/a".to_string()
            } else {
                render::fmt_num(v.iter().sum::<f64>() / v.len() as f64, decimals)
            };
            out.push_str(&format!(
                "  {}  {}  {:>8}{}\n",
                svc(k),
                render::sparkline(v, spark_w, max),
                avg,
                if v.is_empty() { dim("  (not measured)") } else { String::new() }
            ));
        }
    }
    let (j, r) = (ts.summary.get("java"), ts.summary.get("rust"));
    out.push('\n');
    out.push_str(&format!("{:<22}{:>9}{:>9}{:>9}\n", bold("summary"), "java", "rust", "delta"));
    let mut row = |label: String, a: Option<f64>, b: Option<f64>, decimals: usize, lower_is_better: bool| {
        if a.is_none() && b.is_none() {
            return;
        }
        let f = |v: Option<f64>| v.map(|x| render::fmt_num(x, decimals)).unwrap_or_else(|| "n/a".into());
        let delta = match (a, b) {
            (Some(a), Some(b)) => {
                let d = render::pct_delta(a, b);
                if (lower_is_better && b < a) || (!lower_is_better && b > a) {
                    green(&format!("{d:>9}"))
                } else {
                    yellow(&format!("{d:>9}"))
                }
            }
            _ => format!("{:>9}", "n/a"),
        };
        out.push_str(&format!("{label:<22}{:>9}{:>9}{delta}\n", f(a), f(b)));
    };
    let (dj, dr) = (delivered(ts, "java"), delivered(ts, "rust"));
    let rate = ts.rate_per_second;
    row("delivered req/s".into(), dj, dr, 0, false);
    row(
        format!("cpu cores @{rate} req/s"),
        j.and_then(|s| s.cpu_cores),
        r.and_then(|s| s.cpu_cores),
        2,
        true,
    );
    row("p50 ms".into(), j.map(|s| s.p50_ms), r.map(|s| s.p50_ms), 1, true);
    row("p99 ms".into(), j.map(|s| s.p99_ms), r.map(|s| s.p99_ms), 1, true);
    row("rss MB".into(), j.and_then(|s| s.rss_mb), r.and_then(|s| s.rss_mb), 0, true);
    row(
        "peak req/s".into(),
        j.and_then(|s| s.peak_rps),
        r.and_then(|s| s.peak_rps),
        0,
        false,
    );
    out.push('\n');
    let short: Vec<String> = [("java", j, dj), ("rust", r, dr)]
        .into_iter()
        .filter_map(|(k, sm, d)| {
            let d = d.filter(|d| *d < 0.95 * rate as f64)?;
            let errors = sm.map(|s| s.errors).unwrap_or(0);
            Some(if errors > 0 {
                format!("{k} {d:.0} ({errors} errors)")
            } else {
                format!("{k} {d:.0}")
            })
        })
        .collect();
    match (j.and_then(|s| s.cpu_cores), r.and_then(|s| s.cpu_cores), dj, dr) {
        (Some(a), Some(b), Some(da), Some(db)) if short.is_empty() => {
            let freed = a - b;
            out.push_str(&bold(&format!(
                "cpu freed at {rate} req/s: {:.2} cores ({:.0}% of the java footprint) — same traffic on {:.1}x fewer cores",
                freed,
                freed / a * 100.0,
                a / b.max(1e-9)
            )));
            let _ = (da, db);
        }
        (Some(a), Some(b), Some(da), Some(db)) => {
            // Delivered rates differ: compare CPU per request instead of claiming a saving at the requested rate.
            let (ja, rb) = (a * 1000.0 / da.max(1e-9), b * 1000.0 / db.max(1e-9));
            out.push_str(&yellow(&format!(
                "# requested {rate} req/s but delivered only {} — failed or saturated, so no like-for-like saving is claimed",
                short.join(", ")
            )));
            out.push('\n');
            out.push_str(&bold(&format!(
                "cpu per request: java {ja:.2} core-ms, rust {rb:.2} core-ms ({:.1}x less)",
                ja / rb.max(1e-9)
            )));
        }
        _ => {
            out.push_str(&dim(
                "# one side not measured — run both services against the same database and rerun",
            ));
            if !short.is_empty() {
                out.push('\n');
                out.push_str(&yellow(&format!(
                    "# requested {rate} req/s but delivered only {}",
                    short.join(", ")
                )));
            }
        }
    }
    out.push('\n');
    out
}

/// Mean delivered req/s of one side: from the summary, or recomputed from the samples for older files.
fn delivered(ts: &Timeseries, side: &str) -> Option<f64> {
    let summary = ts.summary.get(side)?;
    if summary.rps > 0.0 {
        return Some(summary.rps);
    }
    let samples = &ts.services.get(side)?.samples;
    (!samples.is_empty()).then(|| samples.iter().map(|s| s.rps).sum::<f64>() / samples.len() as f64)
}

pub fn show(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ts: Timeseries = serde_json::from_str(&std::fs::read_to_string(dir.join("timeseries.json"))?)?;
    println!(
        "{}",
        dim(&format!(
            "# {} · {} · rate {} req/s · {}s",
            ts.measured_at, ts.host, ts.rate_per_second, ts.duration_seconds
        ))
    );
    println!("{}", render_results(&ts));
    Ok(())
}

pub async fn watch(bases: &Bases, sources: &Sources, interval: u64) -> Result<(), Box<dyn std::error::Error>> {
    let client = api::client();
    for s in [Service::Java, Service::Rust] {
        println!("{} {} · {}", svc(s.name()), dim(&bases.url(s, "")), dim(&sources.get(s).describe()));
    }
    println!(
        "{}",
        dim("# cpu = cores used by the service process over the interval; p = latency of one GET /pettypes. ctrl-c to stop")
    );
    let mut prev: [Option<(f64, f64)>; 2] = [sources.java.snapshot(), sources.rust.snapshot()];
    let mut prev_at: [Instant; 2] = [Instant::now(), Instant::now()];
    let started = Instant::now();
    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        let mut line = format!("t={:>4.0}s ", started.elapsed().as_secs_f64());
        for (i, s) in [Service::Java, Service::Rust].into_iter().enumerate() {
            let now = sources.get(s).snapshot();
            let elapsed = prev_at[i].elapsed().as_secs_f64().max(1e-3);
            prev_at[i] = Instant::now();
            let cpu = match (prev[i], now) {
                (Some((c0, _)), Some((c1, _))) => format!("{:.2}", ((c1 - c0) / elapsed).max(0.0)),
                _ => "n/a ".into(),
            };
            let rss = now.map(|(_, m)| format!("{m:.0}MB")).unwrap_or_else(|| "n/a".into());
            prev[i] = now;
            let p = api::probe(&client, &bases.url(s, "pettypes")).await;
            let lat = if p.ok() { format!("{:.1}ms", p.ms) } else { red("down") };
            line.push_str(&format!(" {} cpu={cpu} rss={rss:>6} p={lat:<8}", svc(s.name())));
        }
        println!("{line}");
    }
}

fn host_label() -> String {
    let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                l.strip_prefix("model name")
                    .map(|m| m.trim_start_matches([' ', ':', '\t']).to_string())
            })
        })
        .unwrap_or_else(|| std::env::consts::ARCH.to_string());
    format!("{cpus} vcpu · {model}")
}
