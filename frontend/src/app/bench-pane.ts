import { Component, computed, effect, inject, signal } from '@angular/core';
import { BenchDataService, mean, sparkline } from './core/bench-data.service';
import { Sample, ServiceKind, Timeseries } from './core/models';

type Metric = { key: keyof Sample; label: string; unit: string; decimals: number; lowerIsBetter: boolean };

const METRICS: Metric[] = [
  { key: 'cpuCores', label: 'cpu', unit: 'cores', decimals: 2, lowerIsBetter: true },
  { key: 'rps', label: 'rps', unit: 'req/s', decimals: 0, lowerIsBetter: false },
  { key: 'p99Ms', label: 'p99', unit: 'ms', decimals: 1, lowerIsBetter: true },
  { key: 'rssMb', label: 'rss', unit: 'MB', decimals: 0, lowerIsBetter: true },
];

const WIDTH = 40;

@Component({
  selector: 'bench-pane',
  template: `
    <div class="title">bench <span class="dim">— bench/results/ as written by bench/run.sh</span>
      <span class="actions">
        <a (click)="replay()">replay</a>
        <a (click)="skip()">skip</a>
        <a (click)="reload()">reload</a>
      </span>
    </div>
    <pre class="term" #term>{{ screen() }}<span class="cursor" [class.blink]="typing()">█</span></pre>
  `,
  styles: `
    :host { display: flex; flex-direction: column; min-height: 0; gap: 6px; }
    .actions { float: right; }
    .actions a { margin-left: 12px; }
    .term { flex: 1; margin: 0; padding: 8px 10px; overflow: auto; border: 1px solid var(--line); background: #06080c; white-space: pre; font-size: 12px; line-height: 1.4; }
    .cursor { color: var(--green); opacity: 0.4; }
    .cursor.blink { opacity: 1; animation: blink 1s steps(1) infinite; }
    @keyframes blink { 50% { opacity: 0; } }
  `,
})
export class BenchPane {
  readonly data = inject(BenchDataService);
  readonly shown = signal(0);
  readonly typing = signal(false);
  private timer: ReturnType<typeof setInterval> | null = null;

  readonly lines = computed<string[]>(() => {
    if (!this.data.loaded()) return ['$ loading bench/results …'];
    const s = this.data.series();
    const log = this.data.log();
    const out: string[] = [];
    out.push(`$ bench/run.sh${s ? ` --rate ${s.ratePerSecond} --duration ${s.durationSeconds}s` : ''}`);
    if (!s) {
      out.push(`# ${this.data.error()}`);
      out.push('# nothing measured yet: start both services against the same postgres and run bench/run.sh');
      out.push('# (left pane still works live against whatever is listening on :9966 / :9977)');
      return out;
    }
    out.push(`# ${s.measuredAt}${s.host ? ' · ' + s.host : ''}${s.db ? ' · ' + s.db : ''}`);
    out.push(...(log.length ? log : ['# (no run.log)']));
    out.push('');
    out.push(...this.charts(s));
    out.push('');
    out.push(...this.summary(s));
    return out;
  });

  readonly screen = computed(() => this.lines().slice(0, this.shown()).join('\n') + (this.typing() ? '\n' : ''));

  constructor() {
    void this.data.load();
    effect(() => {
      this.lines();
      this.replay();
    });
  }

  replay(): void {
    this.stop();
    this.shown.set(0);
    this.typing.set(true);
    const total = () => this.lines().length;
    this.timer = setInterval(() => {
      if (this.shown() >= total()) return this.stop();
      this.shown.update((n) => n + 1);
      queueMicrotask(() => document.querySelector('.term')?.scrollTo({ top: 1e9 }));
    }, 45);
  }

  skip(): void {
    this.stop();
    this.shown.set(this.lines().length);
  }

  reload(): void {
    void this.data.load();
  }

  private stop(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
    this.typing.set(false);
  }

  private charts(s: Timeseries): string[] {
    const out: string[] = [];
    const kinds: ServiceKind[] = ['java', 'rust'];
    for (const m of METRICS) {
      const values = (k: ServiceKind) => (s.services[k]?.samples ?? []).map((x) => Number(x[m.key] ?? 0));
      const max = Math.max(...kinds.flatMap(values), 0);
      if (max === 0) continue;
      out.push(`${m.label.padEnd(4)} ${m.unit}`);
      for (const k of kinds) {
        const v = values(k);
        const avg = v.length ? mean(v).toFixed(m.decimals) : 'n/a';
        out.push(`  ${k.padEnd(5)}${sparkline(v, WIDTH, max)}  ${avg.padStart(7)}${v.length ? '' : '  (no samples)'}`);
      }
    }
    return out;
  }

  private summary(s: Timeseries): string[] {
    const j = s.summary.java;
    const r = s.summary.rust;
    const out = ['summary' + ' '.repeat(10) + 'java'.padStart(9) + 'rust'.padStart(9) + 'delta'.padStart(9)];
    const row = (label: string, a?: number, b?: number, decimals = 1, lowerIsBetter = true) => {
      if (a == null && b == null) return;
      const delta = a != null && b != null && a !== 0 ? `${(((b - a) / a) * 100).toFixed(0)}%` : 'n/a';
      const good = a != null && b != null && (lowerIsBetter ? b < a : b > a);
      out.push(`${label.padEnd(17)}${fmt(a, decimals).padStart(9)}${fmt(b, decimals).padStart(9)}${(delta.startsWith('-') || !good ? delta : '+' + delta).padStart(9)}`);
    };
    row(`cpu cores @${s.ratePerSecond}rps`, j?.cpuCores, r?.cpuCores, 2);
    row('p99 ms', j?.p99Ms, r?.p99Ms, 1);
    row('rss MB', j?.rssMb, r?.rssMb, 0);
    row('peak req/s', j?.peakRps, r?.peakRps, 0, false);
    if (j?.cpuCores != null && r?.cpuCores != null) {
      const freed = j.cpuCores - r.cpuCores;
      out.push('');
      out.push(`cpu freed at ${s.ratePerSecond} req/s: ${freed.toFixed(2)} cores (${((freed / j.cpuCores) * 100).toFixed(0)}% of the java footprint)`);
    } else {
      out.push('');
      out.push('# rust side not measured yet');
    }
    return out;
  }
}

function fmt(v: number | undefined, decimals: number): string {
  return v == null ? 'n/a' : v.toFixed(decimals);
}
