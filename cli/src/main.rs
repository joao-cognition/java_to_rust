//! `j2r` — the java → rust migration console, in the terminal.
//!
//! * `j2r api <path>`   same GET against both services, raw JSON side by side, diff line, optional burst p50/p99
//! * `j2r bench`        fixed-rate load against each service while sampling CPU and RSS of the real process
//! * `j2r watch`        live per-second CPU / RSS / probe latency of both services
//! * `j2r show`         re-render a saved bench (sparklines + before/after summary)

mod api;
mod bench;
mod render;
mod sampler;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "j2r",
    version,
    about = "java → rust migration console: raw API comparison and CPU/latency benchmark"
)]
pub struct Cli {
    /// Base URL of the Java service (Spring Boot)
    #[arg(long, global = true, env = "J2R_JAVA", default_value = "http://localhost:9966/petclinic/api")]
    pub java: String,
    /// Base URL of the Rust service (Axum)
    #[arg(long, global = true, env = "J2R_RUST", default_value = "http://localhost:9977/petclinic/api")]
    pub rust: String,
    /// Disable colours
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
pub enum Service {
    Java,
    Rust,
}

impl Service {
    pub fn name(self) -> &'static str {
        match self {
            Service::Java => "java",
            Service::Rust => "rust",
        }
    }
}

#[derive(Subcommand)]
pub enum Cmd {
    /// GET one path on both services and compare the raw responses
    Api {
        /// Path under the API base, e.g. `vets`, `owners/1`, `owners?lastName=Davis`
        path: String,
        /// Also send N sequential requests per side and print p50/p99
        #[arg(short, long, default_value_t = 0)]
        n: usize,
        /// Print only status/latency/diff, not the bodies
        #[arg(long)]
        no_body: bool,
    },
    /// Fixed-rate load against each service while sampling CPU seconds and RSS of its process
    Bench {
        /// Requests per second for the fixed-rate phase
        #[arg(long, default_value_t = 300)]
        rate: u32,
        /// Seconds of fixed-rate load per service
        #[arg(long, default_value_t = 30)]
        duration: u64,
        /// Seconds of uncapped (closed-loop, 64 in flight) load per service to find peak req/s; 0 to skip
        #[arg(long, default_value_t = 10)]
        peak: u64,
        /// Directory for timeseries.json, summary.json and run.log
        #[arg(long, default_value = "bench/results")]
        out: PathBuf,
        /// Only one side
        #[arg(long)]
        only: Option<Service>,
        /// PID of the Java process (auto-detected from the listening port when omitted)
        #[arg(long)]
        java_pid: Option<i32>,
        /// PID of the Rust process
        #[arg(long)]
        rust_pid: Option<i32>,
        /// Docker container name/id serving the Java port (auto-detected when omitted)
        #[arg(long)]
        java_container: Option<String>,
        /// Docker container name/id serving the Rust port
        #[arg(long)]
        rust_container: Option<String>,
    },
    /// Live CPU / RSS / probe latency of both services, one line per interval
    Watch {
        #[arg(long, default_value_t = 1)]
        interval: u64,
        #[arg(long)]
        java_pid: Option<i32>,
        #[arg(long)]
        rust_pid: Option<i32>,
        #[arg(long)]
        java_container: Option<String>,
        #[arg(long)]
        rust_container: Option<String>,
    },
    /// Re-render a saved bench from its results directory
    Show {
        #[arg(default_value = "bench/results")]
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    render::set_color(!cli.no_color);
    let code = match run(cli).await {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", render::red(&format!("error: {e}")));
            1
        }
    };
    std::process::exit(code);
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let bases = api::Bases {
        java: cli.java.trim_end_matches('/').to_string(),
        rust: cli.rust.trim_end_matches('/').to_string(),
    };
    match cli.cmd {
        Cmd::Api { path, n, no_body } => api::compare(&bases, &path, n, !no_body).await,
        Cmd::Bench {
            rate,
            duration,
            peak,
            out,
            only,
            java_pid,
            rust_pid,
            java_container,
            rust_container,
        } => {
            let sources = sampler::Sources::resolve(&bases, java_pid, rust_pid, java_container.as_deref(), rust_container.as_deref());
            bench::run(
                &bases,
                &sources,
                bench::Plan {
                    rate,
                    duration,
                    peak,
                    only,
                },
                &out,
            )
            .await
        }
        Cmd::Watch {
            interval,
            java_pid,
            rust_pid,
            java_container,
            rust_container,
        } => {
            let sources = sampler::Sources::resolve(&bases, java_pid, rust_pid, java_container.as_deref(), rust_container.as_deref());
            bench::watch(&bases, &sources, interval).await
        }
        Cmd::Show { dir } => bench::show(&dir),
    }
}
