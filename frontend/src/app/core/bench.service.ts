import { HttpClient } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';
import { firstValueFrom } from 'rxjs';
import { BenchSummary, ServiceMetrics } from './models';
import { SettingsService } from './settings.service';

export interface MetricRow {
  key: keyof ServiceMetrics;
  label: string;
  unit: string;
  java: number | undefined;
  rust: number | undefined;
  /** Factor expressed so that > 1 always means "Rust better". */
  factor: number | undefined;
  betterWhen: 'lower' | 'higher';
  decimals: number;
}

export interface Impact {
  ratio: number;
  migratedVcpus: number;
  freedVcpus: number;
  annualSavingUsd: number;
  monthlySavingUsd: number;
  perPercentAnnualUsd: number;
  requestsPerYearPerVcpu: number;
}

const ROWS: Array<Pick<MetricRow, 'key' | 'label' | 'unit' | 'betterWhen' | 'decimals'>> = [
  { key: 'cpuCoresAtFixedRate', label: 'CPU for the same 500 req/s', unit: 'cores', betterWhen: 'lower', decimals: 2 },
  { key: 'costPerMillionRequestsUsd', label: 'vCPU cost per million requests', unit: '$', betterWhen: 'lower', decimals: 4 },
  { key: 'peakRequestsPerSecond', label: 'Peak throughput on 2 vCPUs', unit: 'req/s', betterWhen: 'higher', decimals: 0 },
  { key: 'p99Ms', label: 'p99 latency at peak', unit: 'ms', betterWhen: 'lower', decimals: 0 },
  { key: 'memoryMb', label: 'Memory', unit: 'MB', betterWhen: 'lower', decimals: 0 },
  { key: 'startupSeconds', label: 'Start-up time', unit: 's', betterWhen: 'lower', decimals: 1 },
  { key: 'dbQueriesPerRequest', label: 'DB queries per request', unit: '', betterWhen: 'lower', decimals: 0 },
];

@Injectable({ providedIn: 'root' })
export class BenchService {
  private readonly http = inject(HttpClient);
  private readonly settings = inject(SettingsService);

  readonly summary = signal<BenchSummary | null>(null);
  readonly sourceLabel = signal<string>('');
  readonly loadError = signal<string>('');

  readonly rows = computed<MetricRow[]>(() => {
    const s = this.summary();
    if (!s) return [];
    return ROWS.map((r) => {
      const java = s.services.java[r.key] as number | undefined;
      const rust = s.services.rust[r.key] as number | undefined;
      let factor: number | undefined;
      if (typeof java === 'number' && typeof rust === 'number' && java > 0 && rust > 0) {
        factor = r.betterWhen === 'lower' ? java / rust : rust / java;
      }
      return { ...r, java, rust, factor };
    }).filter((r) => r.java !== undefined || r.rust !== undefined);
  });

  /** CPU ratio Java/Rust for identical traffic — the single number the cost model hinges on. */
  readonly cpuRatio = computed(() => {
    const s = this.summary();
    if (!s) return 1;
    const { java, rust } = s.services;
    return rust.cpuCoresAtFixedRate > 0 ? java.cpuCoresAtFixedRate / rust.cpuCoresAtFixedRate : 1;
  });

  readonly impact = computed<Impact>(() => {
    const st = this.settings.settings();
    const ratio = this.cpuRatio();
    const migratedVcpus = st.fleetVcpus * (st.migratedSharePct / 100);
    const freedVcpus = ratio > 0 ? migratedVcpus * (1 - 1 / ratio) : 0;
    const hoursPerYear = 8760;
    const annualSavingUsd = freedVcpus * hoursPerYear * st.vcpuHourUsd;
    const perPercentAnnualUsd =
      st.fleetVcpus * 0.01 * (ratio > 0 ? 1 - 1 / ratio : 0) * hoursPerYear * st.vcpuHourUsd;
    const s = this.summary();
    const rps = s?.workload?.ratePerSecond ?? 500;
    const javaCores = s?.services.java.cpuCoresAtFixedRate ?? 1;
    const requestsPerYearPerVcpu = javaCores > 0 ? (rps / javaCores) * 3600 * hoursPerYear : 0;
    return {
      ratio,
      migratedVcpus,
      freedVcpus,
      annualSavingUsd,
      monthlySavingUsd: annualSavingUsd / 12,
      perPercentAnnualUsd,
      requestsPerYearPerVcpu,
    };
  });

  async loadReference(): Promise<void> {
    try {
      const s = await firstValueFrom(this.http.get<BenchSummary>('bench/reference-summary.json'));
      this.apply(s, s.source ?? 'Reference run');
    } catch (err) {
      this.loadError.set('Could not load the reference benchmark.');
    }
  }

  async loadFromFile(file: File): Promise<void> {
    try {
      const parsed = JSON.parse(await file.text()) as BenchSummary;
      if (!parsed?.services?.java || !parsed?.services?.rust) {
        throw new Error('missing services.java / services.rust');
      }
      this.apply(parsed, `${file.name} — measured ${formatDate(parsed.measuredAt)}`);
    } catch (err) {
      this.loadError.set(`Not a bench summary: ${(err as Error).message}`);
    }
  }

  private apply(s: BenchSummary, label: string): void {
    this.loadError.set('');
    this.summary.set(s);
    this.sourceLabel.set(label);
    if (s.vcpuHourUsd && this.settings.settings().vcpuHourUsd === 0.0509) {
      this.settings.patch({ vcpuHourUsd: s.vcpuHourUsd });
    }
  }
}

export function formatDate(iso: string | undefined): string {
  if (!iso) return 'unknown date';
  const d = new Date(iso);
  return isNaN(d.getTime()) ? iso : d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' });
}
