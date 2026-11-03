import { HttpClient } from '@angular/common/http';
import { Injectable, inject, signal } from '@angular/core';
import { firstValueFrom } from 'rxjs';
import { Timeseries } from './models';

const RESULTS = 'bench/results';

@Injectable({ providedIn: 'root' })
export class BenchDataService {
  private readonly http = inject(HttpClient);

  readonly series = signal<Timeseries | null>(null);
  readonly log = signal<string[]>([]);
  readonly error = signal('');
  readonly loaded = signal(false);

  async load(): Promise<void> {
    this.error.set('');
    const [series, log] = await Promise.all([
      firstValueFrom(this.http.get<Timeseries>(`${RESULTS}/timeseries.json`)).catch(() => null),
      firstValueFrom(this.http.get(`${RESULTS}/run.log`, { responseType: 'text' })).catch(() => ''),
    ]);
    this.series.set(series);
    this.log.set(log ? log.replace(/\s+$/, '').split('\n') : []);
    if (!series) this.error.set(`${RESULTS}/timeseries.json not found — run bench/run.sh`);
    this.loaded.set(true);
  }
}

const BARS = '▁▂▃▄▅▆▇█';

/** Sparkline of `values` resampled to `width` chars, scaled to `max` (shared across the pair). */
export function sparkline(values: number[], width: number, max: number): string {
  if (!values.length) return ' '.repeat(width);
  const out: string[] = [];
  for (let i = 0; i < width; i++) {
    const from = Math.floor((i * values.length) / width);
    const to = Math.max(from + 1, Math.floor(((i + 1) * values.length) / width));
    const bucket = values.slice(from, to);
    const v = bucket.reduce((a, b) => a + b, 0) / bucket.length;
    const idx = max > 0 ? Math.min(BARS.length - 1, Math.round((v / max) * (BARS.length - 1))) : 0;
    out.push(BARS[idx]);
  }
  return out.join('');
}

export function mean(values: number[]): number {
  return values.length ? values.reduce((a, b) => a + b, 0) / values.length : 0;
}
