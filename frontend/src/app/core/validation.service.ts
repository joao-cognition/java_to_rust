import { Injectable, computed, inject, signal } from '@angular/core';
import { ApiService } from './api.service';
import { SettingsService } from './settings.service';

export type ParityState = 'identical' | 'different' | 'rust-missing' | 'java-missing' | 'pending';

export interface EndpointCheck {
  path: string;
  label: string;
  parity: ParityState;
  javaStatus: number;
  rustStatus: number;
  java: LatencyStats | null;
  rust: LatencyStats | null;
}

export interface LatencyStats { p50: number; p99: number; mean: number; samples: number; }

export interface Verdict {
  state: 'idle' | 'running' | 'validated' | 'partial' | 'failed';
  headline: string;
  detail: string;
  p99Speedup?: number;
  parityPct?: number;
}

/** The endpoints a migrated service must serve identically before we call the migration validated. */
export const VALIDATION_ENDPOINTS: Array<{ path: string; label: string }> = [
  { path: '/vets', label: 'Vets (reference list)' },
  { path: '/specialties', label: 'Specialties (reference list)' },
  { path: '/pettypes', label: 'Pet types (reference list)' },
  { path: '/owners', label: 'Owners (list)' },
  { path: '/owners?lastName=Davis', label: 'Owner search by last name' },
  { path: '/owners/1', label: 'Owner lookup (nested pets + visits)' },
  { path: '/pets/1', label: 'Pet lookup' },
];

@Injectable({ providedIn: 'root' })
export class ValidationService {
  private readonly api = inject(ApiService);
  private readonly settings = inject(SettingsService);

  readonly checks = signal<EndpointCheck[]>([]);
  readonly running = signal(false);
  readonly progress = signal(0);
  readonly lastRunAt = signal<Date | null>(null);

  readonly verdict = computed<Verdict>(() => {
    const checks = this.checks();
    if (this.running()) {
      return { state: 'running', headline: 'Validating…', detail: `${Math.round(this.progress() * 100)}% of probes sent` };
    }
    if (!checks.length) {
      return {
        state: 'idle',
        headline: 'Not validated yet',
        detail: 'Runs the same requests against the Java and Rust services, checks the JSON is identical and compares latency.',
      };
    }
    const rustUp = checks.some((c) => c.rustStatus > 0);
    if (!rustUp) {
      return {
        state: 'failed',
        headline: 'Rust service not reachable',
        detail: 'Start the Rust port (port 9977) or fix the Rust base URL, then validate again.',
      };
    }
    const identical = checks.filter((c) => c.parity === 'identical').length;
    const parityPct = (identical / checks.length) * 100;
    const javaP99 = percentileAcross(checks.map((c) => c.java?.p99));
    const rustP99 = percentileAcross(checks.map((c) => c.rust?.p99));
    const p99Speedup = javaP99 && rustP99 ? javaP99 / rustP99 : undefined;
    const migrated = checks.filter((c) => c.rustStatus >= 200 && c.rustStatus < 300).length;

    if (identical === checks.length && (p99Speedup ?? 0) >= 1) {
      return {
        state: 'validated',
        headline: 'Migration validated',
        detail: `All ${checks.length} endpoints return identical JSON; Rust p99 is ${p99Speedup!.toFixed(1)}x lower in this probe.`,
        p99Speedup,
        parityPct,
      };
    }
    if (identical === checks.length) {
      return {
        state: 'partial',
        headline: 'Parity OK, latency not better in this probe',
        detail: 'Browser-side timing through the dev proxy is noisy. Use bench/run.sh for the authoritative numbers.',
        p99Speedup,
        parityPct,
      };
    }
    return {
      state: 'partial',
      headline: `${migrated}/${checks.length} endpoints migrated, ${identical} identical`,
      detail: 'Endpoints marked "different" or "not ported" block sign-off.',
      p99Speedup,
      parityPct,
    };
  });

  async run(): Promise<void> {
    if (this.running()) return;
    this.running.set(true);
    this.progress.set(0);
    const n = Math.max(3, Math.min(200, this.settings.settings().probeRequests));
    const total = VALIDATION_ENDPOINTS.length * 2 * n;
    let done = 0;
    const results: EndpointCheck[] = [];

    for (const ep of VALIDATION_ENDPOINTS) {
      const first = { java: await this.api.probe('java', ep.path), rust: await this.api.probe('rust', ep.path) };
      done += 2;
      const javaMs: number[] = [first.java.ms];
      const rustMs: number[] = [first.rust.ms];
      let javaStatus = first.java.status;
      let rustStatus = first.rust.status;
      for (let i = 1; i < n; i++) {
        if (ok(javaStatus)) {
          const r = await this.api.probe('java', ep.path);
          javaMs.push(r.ms);
          if (!ok(r.status)) javaStatus = r.status;
        }
        if (ok(rustStatus)) {
          const r = await this.api.probe('rust', ep.path);
          rustMs.push(r.ms);
          if (!ok(r.status)) rustStatus = r.status;
        }
        done += 2;
        this.progress.set(done / total);
      }
      results.push({
        path: ep.path,
        label: ep.label,
        parity: parity(javaStatus, rustStatus, first.java.body, first.rust.body),
        javaStatus,
        rustStatus,
        java: ok(javaStatus) ? stats(javaMs) : null,
        rust: ok(rustStatus) ? stats(rustMs) : null,
      });
      this.checks.set([...results]);
    }
    this.lastRunAt.set(new Date());
    this.running.set(false);
    this.progress.set(1);
  }

  reset(): void {
    this.checks.set([]);
    this.progress.set(0);
  }
}

function ok(status: number): boolean {
  return status >= 200 && status < 300;
}

function parity(javaStatus: number, rustStatus: number, javaBody: unknown, rustBody: unknown): ParityState {
  if (!ok(javaStatus)) return 'java-missing';
  if (!ok(rustStatus)) return 'rust-missing';
  return canonical(javaBody) === canonical(rustBody) ? 'identical' : 'different';
}

/** Stable JSON: sorted keys, so field order differences between serializers don't count as a diff. */
export function canonical(value: unknown): string {
  return JSON.stringify(sortKeys(value));
}

function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === 'object') {
    return Object.keys(value as Record<string, unknown>)
      .sort()
      .reduce<Record<string, unknown>>((acc, k) => {
        acc[k] = sortKeys((value as Record<string, unknown>)[k]);
        return acc;
      }, {});
  }
  return value;
}

export function stats(samples: number[]): LatencyStats {
  const sorted = [...samples].sort((a, b) => a - b);
  const pick = (p: number) => sorted[Math.min(sorted.length - 1, Math.floor(p * sorted.length))];
  const mean = sorted.reduce((a, b) => a + b, 0) / sorted.length;
  return { p50: pick(0.5), p99: pick(0.99), mean, samples: sorted.length };
}

function percentileAcross(values: Array<number | undefined>): number | undefined {
  const xs = values.filter((v): v is number => typeof v === 'number');
  return xs.length ? Math.max(...xs) : undefined;
}
