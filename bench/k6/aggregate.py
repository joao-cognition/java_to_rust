#!/usr/bin/env python3
"""Per-second aggregation of a k6 `--out json` stream plus /proc sampling of the service.

Reads k6 JSON lines on stdin, buckets `http_req_duration` points per wall-clock second and,
once per second, samples CPU time (utime+stime, all threads) and resident memory of PID from
/proc. Writes the samples as JSON to --out and prints one plain-text line per second:

    [java] t=12s cpu=0.47 cores rps=501 p50=4.1ms p99=21.4ms rss=552MB
"""
import argparse
import json
import os
import sys
import threading
import time
from datetime import datetime, timezone

CLK_TCK = os.sysconf("SC_CLK_TCK")


def cpu_seconds(pid):
    with open(f"/proc/{pid}/stat") as f:
        fields = f.read().rsplit(")", 1)[1].split()
    return (int(fields[11]) + int(fields[12])) / CLK_TCK


def rss_mb(pid):
    with open(f"/proc/{pid}/status") as f:
        for line in f:
            if line.startswith("VmRSS:"):
                return int(line.split()[1]) / 1024
    return 0.0


def percentile(sorted_values, p):
    if not sorted_values:
        return 0.0
    k = (len(sorted_values) - 1) * p
    lo, hi = int(k), min(int(k) + 1, len(sorted_values) - 1)
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * (k - lo)


def parse_ts(ts):
    """k6 emits RFC 3339 with nanoseconds (2026-11-03T20:39:30.123456789+01:00)."""
    head, _, rest = ts.partition(".")
    frac, tz = rest, ""
    for i, c in enumerate(rest):
        if c in "+-Z":
            frac, tz = rest[:i], rest[i:]
            break
    frac = frac[:6].ljust(6, "0")
    if tz in ("Z", ""):
        tz = "+00:00"
    return datetime.fromisoformat(f"{head}.{frac}{tz}").timestamp()


class Aggregator:
    """Buckets k6 points by their own timestamps (k6 runs on this host, so they share the clock
    with /proc samples) and samples /proc every 100 ms; the CPU of a bucket is the delta between
    the samples nearest to its start and end."""

    def __init__(self, service, pid, lag, duration):
        self.service = service
        self.pid = pid
        self.lag = lag
        self.duration = duration
        self.t0 = None
        self.buckets = {}  # second index -> {"durations": [...], "failed": n}
        self.proc = []  # (wall, cpu_seconds, rss_mb)
        self.lock = threading.Lock()
        self.samples = []
        self.done = False
        self.closed = 0

    def add(self, point):
        metric = point["metric"]
        if metric not in ("http_req_duration", "http_req_failed"):
            return
        data = point["data"]
        ts = parse_ts(data["time"])
        with self.lock:
            if self.t0 is None:
                self.t0 = ts
            idx = int(ts - self.t0)
            if self.duration and idx >= self.duration:
                return  # requests completing after the window
            bucket = self.buckets.setdefault(idx, {"durations": [], "failed": 0})
            if metric == "http_req_duration":
                bucket["durations"].append(data["value"])
            elif data["value"] >= 1:
                bucket["failed"] += 1

    def proc_at(self, wall):
        """/proc sample closest to `wall` (the list is short and sorted)."""
        best = min(self.proc, key=lambda s: abs(s[0] - wall))
        return best

    def close_bucket(self, idx):
        bucket = self.buckets.pop(idx, {"durations": [], "failed": 0})
        durations = sorted(bucket["durations"])
        start, end = self.proc_at(self.t0 + idx), self.proc_at(self.t0 + idx + 1)
        wall = end[0] - start[0]
        cpu = (end[1] - start[1]) / wall if wall > 0 else 0.0
        sample = {
            "t": idx + 1,
            "cpuCores": round(cpu, 3),
            "rps": len(durations),
            "p50Ms": round(percentile(durations, 0.50), 2),
            "p99Ms": round(percentile(durations, 0.99), 2),
            "rssMb": round(end[2], 1),
            "failed": bucket["failed"],
        }
        self.samples.append(sample)
        print(
            f"[{self.service}] t={sample['t']}s cpu={sample['cpuCores']:.2f} cores "
            f"rps={sample['rps']} p50={sample['p50Ms']:.1f}ms p99={sample['p99Ms']:.1f}ms "
            f"rss={sample['rssMb']:.0f}MB",
            file=sys.stderr,
            flush=True,
        )

    def sample_proc(self):
        try:
            self.proc.append((time.time(), cpu_seconds(self.pid), rss_mb(self.pid)))
        except FileNotFoundError:
            pass

    def sampler(self):
        while not self.done:
            self.sample_proc()
            with self.lock:
                # a bucket is closed `lag` seconds after it ends so that late-flushed points land in it
                while self.t0 is not None and time.time() >= self.t0 + self.closed + 1 + self.lag:
                    if self.duration and self.closed >= self.duration:
                        break
                    self.close_bucket(self.closed)
                    self.closed += 1
            time.sleep(0.1)
        self.sample_proc()
        with self.lock:
            for idx in sorted(self.buckets):
                if idx >= self.closed:
                    self.close_bucket(idx)
                    self.closed = idx + 1


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--service", required=True)
    ap.add_argument("--pid", type=int, required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--lag", type=float, default=1.5)
    ap.add_argument("--duration", type=int, default=0, help="window length in seconds; later points are ignored")
    args = ap.parse_args()

    agg = Aggregator(args.service, args.pid, args.lag, args.duration)
    thread = threading.Thread(target=agg.sampler, daemon=True)
    thread.start()
    for line in sys.stdin:
        if '"type":"Point"' not in line:
            continue
        if '"http_req_duration"' not in line and '"http_req_failed"' not in line:
            continue
        try:
            agg.add(json.loads(line))
        except (ValueError, KeyError):
            continue
    agg.done = True
    thread.join()
    with open(args.out, "w") as f:
        json.dump(
            {
                "service": args.service,
                "measuredAt": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                "samples": agg.samples,
            },
            f,
            indent=1,
        )


if __name__ == "__main__":
    main()
