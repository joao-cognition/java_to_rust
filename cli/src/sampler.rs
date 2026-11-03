//! Where the CPU and RSS numbers come from: the real process behind each port, read from /proc or the
//! container's cgroup. Never from the load generator's point of view.

use crate::api::Bases;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum Source {
    Proc { pid: i32, comm: String },
    Cgroup { name: String, path: PathBuf },
    Unknown { reason: String },
}

impl Source {
    pub fn describe(&self) -> String {
        match self {
            Source::Proc { pid, comm } => format!("pid {pid} ({comm})"),
            Source::Cgroup { name, .. } => format!("container {name}"),
            Source::Unknown { reason } => format!("cpu/rss unavailable: {reason}"),
        }
    }

    pub fn known(&self) -> bool {
        !matches!(self, Source::Unknown { .. })
    }

    /// (cumulative cpu seconds, rss in MB)
    pub fn snapshot(&self) -> Option<(f64, f64)> {
        match self {
            Source::Proc { pid, .. } => {
                let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
                let after = &stat[stat.rfind(')')? + 2..];
                let f: Vec<&str> = after.split_whitespace().collect();
                let utime: f64 = f.get(11)?.parse().ok()?;
                let stime: f64 = f.get(12)?.parse().ok()?;
                let rss_pages: f64 = f.get(21)?.parse().ok()?;
                Some(((utime + stime) / clk_tck(), rss_pages * page_size() / 1048576.0))
            }
            Source::Cgroup { path, .. } => {
                let cpu = fs::read_to_string(path.join("cpu.stat")).ok()?;
                let usec: f64 = cpu.lines().find_map(|l| l.strip_prefix("usage_usec "))?.trim().parse().ok()?;
                let mem: f64 = fs::read_to_string(path.join("memory.current")).ok()?.trim().parse().ok()?;
                Some((usec / 1e6, mem / 1048576.0))
            }
            Source::Unknown { .. } => None,
        }
    }
}

fn clk_tck() -> f64 {
    Command::new("getconf")
        .arg("CLK_TCK")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(100.0)
}

fn page_size() -> f64 {
    Command::new("getconf")
        .arg("PAGESIZE")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(4096.0)
}

pub struct Sources {
    pub java: Source,
    pub rust: Source,
}

impl Sources {
    pub fn resolve(
        bases: &Bases,
        java_pid: Option<i32>,
        rust_pid: Option<i32>,
        java_container: Option<&str>,
        rust_container: Option<&str>,
    ) -> Self {
        Sources {
            java: resolve_one(&bases.java, java_pid, java_container),
            rust: resolve_one(&bases.rust, rust_pid, rust_container),
        }
    }

    pub fn get(&self, s: crate::Service) -> &Source {
        match s {
            crate::Service::Java => &self.java,
            crate::Service::Rust => &self.rust,
        }
    }
}

fn resolve_one(base: &str, pid: Option<i32>, container: Option<&str>) -> Source {
    if let Some(pid) = pid {
        return proc_source(pid).unwrap_or(Source::Unknown {
            reason: format!("/proc/{pid} not readable"),
        });
    }
    if let Some(c) = container {
        return cgroup_source(c).unwrap_or(Source::Unknown {
            reason: format!("container {c} not found"),
        });
    }
    let Some((host, port)) = host_port(base) else {
        return Source::Unknown {
            reason: "could not parse host:port from base URL".into(),
        };
    };
    if !["localhost", "127.0.0.1", "::1", "0.0.0.0"].contains(&host.as_str()) {
        return Source::Unknown {
            reason: format!("{host} is remote — pass --<svc>-pid or --<svc>-container on that host"),
        };
    }
    if let Some(pid) = pid_listening_on(port) {
        if let Some(src) = proc_source(pid) {
            if !matches!(&src, Source::Proc { comm, .. } if comm.contains("docker-proxy")) {
                return src;
            }
        }
    }
    if let Some(id) = container_publishing(port) {
        if let Some(src) = cgroup_source(&id) {
            return src;
        }
    }
    Source::Unknown {
        reason: format!("no readable process found listening on :{port} (pass --java-pid/--rust-pid or --*-container)"),
    }
}

fn host_port(base: &str) -> Option<(String, u16)> {
    let rest = base.split("://").nth(1)?;
    let authority = rest.split('/').next()?;
    let (host, port) = authority.rsplit_once(':')?;
    Some((host.trim_matches(|c| c == '[' || c == ']').to_string(), port.parse().ok()?))
}

fn proc_source(pid: i32) -> Option<Source> {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).ok()?.trim().to_string();
    fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    Some(Source::Proc { pid, comm })
}

fn cgroup_source(name_or_id: &str) -> Option<Source> {
    let out = Command::new("docker")
        .args(["inspect", "-f", "{{.Id}} {{.Name}}", name_or_id])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let (id, name) = s.trim().split_once(' ')?;
    let candidates = [
        PathBuf::from(format!("/sys/fs/cgroup/system.slice/docker-{id}.scope")),
        PathBuf::from(format!("/sys/fs/cgroup/docker/{id}")),
        PathBuf::from(format!("/sys/fs/cgroup/cpu/docker/{id}")),
    ];
    let path = candidates.into_iter().find(|p| p.join("cpu.stat").exists())?;
    Some(Source::Cgroup {
        name: name.trim_start_matches('/').to_string(),
        path,
    })
}

/// Listening socket inode for the port (tcp + tcp6), then the pid owning it (only works for our own processes).
fn pid_listening_on(port: u16) -> Option<i32> {
    let mut inodes = Vec::new();
    for f in ["/proc/net/tcp", "/proc/net/tcp6"] {
        let Ok(t) = fs::read_to_string(f) else { continue };
        for line in t.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 10 || cols[3] != "0A" {
                continue;
            }
            let Some((_, p)) = cols[1].rsplit_once(':') else { continue };
            if u16::from_str_radix(p, 16).ok() == Some(port) {
                inodes.push(cols[9].to_string());
            }
        }
    }
    if inodes.is_empty() {
        return None;
    }
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() else {
            continue;
        };
        let Ok(fds) = fs::read_dir(entry.path().join("fd")) else { continue };
        for fd in fds.flatten() {
            if let Ok(target) = fs::read_link(fd.path()) {
                let t = target.to_string_lossy();
                if inodes.iter().any(|i| t == format!("socket:[{i}]")) {
                    return Some(pid);
                }
            }
        }
    }
    None
}

fn container_publishing(port: u16) -> Option<String> {
    let out = Command::new("docker")
        .args(["ps", "-q", "--filter", &format!("publish={port}")])
        .output()
        .ok()?;
    let id = String::from_utf8_lossy(&out.stdout).lines().next()?.trim().to_string();
    (!id.is_empty()).then_some(id)
}
