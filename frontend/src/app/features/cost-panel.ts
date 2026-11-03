import { Component, computed, inject, signal } from '@angular/core';
import { CurrencyPipe, DecimalPipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { BenchService, formatDate } from '../core/bench.service';
import { SettingsService } from '../core/settings.service';
import { ValidationService } from '../core/validation.service';

@Component({
  selector: 'app-cost-panel',
  imports: [DecimalPipe, CurrencyPipe, FormsModule],
  template: `
    <aside class="panel">
      <header class="panel-head">
        <div>
          <p class="eyebrow">CPU cost impact</p>
          <h2>Java → Rust</h2>
        </div>
        <span class="chip" [class.ok]="verdict().state === 'validated'" [class.warn]="verdict().state === 'partial' || verdict().state === 'running'" [class.muted]="verdict().state === 'idle'" [class.bad]="verdict().state === 'failed'">
          {{ verdict().state === 'validated' ? 'Validated' : verdict().state === 'idle' ? 'Pending' : verdict().state === 'running' ? 'Running' : verdict().state === 'failed' ? 'Blocked' : 'Partial' }}
        </span>
      </header>

      <!-- 1. Benchmark -->
      <section class="block">
        <div class="block-head">
          <h3>1 · Same traffic, same database, 2 vCPUs</h3>
          <label class="btn ghost sm file">
            Load results
            <input type="file" accept="application/json,.json" (change)="onFile($event)" hidden />
          </label>
        </div>
        <p class="muted small">{{ bench.sourceLabel() || 'Loading reference benchmark…' }}</p>
        @if (bench.loadError()) { <p class="small bad">{{ bench.loadError() }}</p> }
        <table class="grid compact">
          <thead><tr><th></th><th class="num java">Java</th><th class="num rust">Rust</th><th class="num">Gain</th></tr></thead>
          <tbody>
            @for (r of bench.rows(); track r.key) {
              <tr>
                <td>{{ r.label }}<span class="unit"> {{ r.unit }}</span></td>
                <td class="num">{{ fmt(r.java, r.decimals) }}</td>
                <td class="num">{{ fmt(r.rust, r.decimals) }}</td>
                <td class="num gain">@if (r.factor) { {{ r.factor | number: '1.1-1' }}x }</td>
              </tr>
            }
          </tbody>
        </table>
        @if (bench.summary(); as s) {
          <p class="muted small">{{ s.workload?.mix }} · {{ s.runs ?? 1 }} run(s) · {{ s.vcpuHourUsd | currency: 'USD': 'symbol': '1.4-4' }}/vCPU-h</p>
        }
      </section>

      <!-- 2. Amadeus scale -->
      <section class="block">
        <h3>2 · What it means at Amadeus scale</h3>
        <p class="muted small">Assumptions are editable. Only CPU is counted; memory (32x less) is left out.</p>
        <div class="inputs">
          <label class="field">vCPUs in the CPU-intensive fleet
            <input type="number" min="1" step="100" [ngModel]="s().fleetVcpus" (ngModelChange)="set('fleetVcpus', $event)" />
          </label>
          <label class="field">Price per vCPU-hour (USD)
            <input type="number" min="0" step="0.0001" [ngModel]="s().vcpuHourUsd" (ngModelChange)="set('vcpuHourUsd', $event)" />
          </label>
        </div>
        <label class="field slider">
          <span>Share of the fleet ported to Rust: <strong>{{ s().migratedSharePct | number: '1.0-1' }}%</strong></span>
          <input type="range" min="0.5" max="100" step="0.5" [ngModel]="s().migratedSharePct" (ngModelChange)="set('migratedSharePct', $event)" />
          <span class="ticks"><span>0.5%</span><span>25%</span><span>50%</span><span>75%</span><span>100%</span></span>
        </label>

        <div class="kpis">
          <div class="kpi">
            <span class="kpi-label">vCPUs freed</span>
            <span class="kpi-value">{{ impact().freedVcpus | number: '1.0-0' }}</span>
            <span class="muted small">of {{ impact().migratedVcpus | number: '1.0-0' }} ported · CPU ratio {{ impact().ratio | number: '1.1-1' }}x</span>
          </div>
          <div class="kpi accent">
            <span class="kpi-label">Saving per year</span>
            <span class="kpi-value">{{ impact().annualSavingUsd | currency: 'USD': 'symbol': '1.0-0' }}</span>
            <span class="muted small">{{ impact().monthlySavingUsd | currency: 'USD': 'symbol': '1.0-0' }} / month</span>
          </div>
        </div>
        <p class="small takeaway">
          Every <strong>1%</strong> of this fleet ported saves about <strong>{{ impact().perPercentAnnualUsd | currency: 'USD': 'symbol': '1.0-0' }}</strong> a year,
          or the same traffic on <strong>{{ (1 - 1 / impact().ratio) * 100 | number: '1.0-0' }}% fewer cores</strong>.
        </p>
      </section>

      <!-- 3. Validation -->
      <section class="block">
        <h3>3 · Validate the migration</h3>
        <div class="verdict" [class]="'verdict ' + verdict().state">
          <strong>{{ verdict().headline }}</strong>
          <span class="small">{{ verdict().detail }}</span>
          @if (validation.running()) {
            <progress [value]="validation.progress()" max="1"></progress>
          }
        </div>
        <div class="inputs">
          <label class="field">Java base URL
            <input type="text" [ngModel]="s().javaBaseUrl" (ngModelChange)="set('javaBaseUrl', $event)" />
          </label>
          <label class="field">Rust base URL
            <input type="text" [ngModel]="s().rustBaseUrl" (ngModelChange)="set('rustBaseUrl', $event)" />
          </label>
        </div>
        <div class="actions">
          <label class="field inline">Requests per endpoint
            <input type="number" min="3" max="200" [ngModel]="s().probeRequests" (ngModelChange)="set('probeRequests', $event)" />
          </label>
          <button class="btn" (click)="validation.run()" [disabled]="validation.running()">
            {{ validation.running() ? 'Validating…' : validation.checks().length ? 'Validate again' : 'Run validation' }}
          </button>
        </div>

        @if (validation.checks().length) {
          <table class="grid compact">
            <thead><tr><th>Endpoint</th><th>Parity</th><th class="num">Java p50/p99</th><th class="num">Rust p50/p99</th></tr></thead>
            <tbody>
              @for (c of validation.checks(); track c.path) {
                <tr>
                  <td><span class="mono">{{ c.path }}</span><br /><span class="muted small">{{ c.label }}</span></td>
                  <td>
                    @switch (c.parity) {
                      @case ('identical') { <span class="chip ok">Identical</span> }
                      @case ('different') { <span class="chip bad">Different</span> }
                      @case ('rust-missing') { <span class="chip warn">{{ c.rustStatus ? 'Not ported (' + c.rustStatus + ')' : 'Rust down' }}</span> }
                      @case ('java-missing') { <span class="chip bad">Java {{ c.javaStatus || 'down' }}</span> }
                      @default { <span class="chip muted">…</span> }
                    }
                  </td>
                  <td class="num">@if (c.java) { {{ c.java.p50 | number: '1.0-0' }} / {{ c.java.p99 | number: '1.0-0' }} ms } @else { — }</td>
                  <td class="num">@if (c.rust) { {{ c.rust.p50 | number: '1.0-0' }} / {{ c.rust.p99 | number: '1.0-0' }} ms } @else { — }</td>
                </tr>
              }
            </tbody>
          </table>
          @if (validation.lastRunAt(); as t) {
            <p class="muted small">Probed from this browser at {{ t.toLocaleTimeString() }} · parity on first response, latency over {{ s().probeRequests }} requests. Authoritative CPU/cost figures come from <span class="mono">bench/run.sh</span>.</p>
          }
        }
      </section>

      <footer class="panel-foot">
        <button class="btn ghost sm" (click)="resetAll()">Reset assumptions</button>
        <span class="muted small">Measured {{ measuredOn() }}</span>
      </footer>
    </aside>
  `,
  styles: `
    :host { display: block; height: 100%; }
    .panel { height: 100%; overflow: auto; background: var(--am-surface); border-left: 1px solid var(--am-line); display: flex; flex-direction: column; }
    .panel-head { display: flex; justify-content: space-between; align-items: flex-start; padding: 18px 18px 10px; }
    .eyebrow { margin: 0 0 2px; font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--am-blue); font-weight: 700; }
    .panel-head h2 { font-size: 18px; }
    .block { padding: 12px 18px 16px; border-top: 1px solid var(--am-line); display: flex; flex-direction: column; gap: 8px; }
    .block h3 { font-size: 13px; color: var(--am-navy); }
    .block-head { display: flex; justify-content: space-between; align-items: center; }
    .small { font-size: 12px; }
    .bad { color: var(--am-red); }
    .unit { color: var(--am-ink-soft); font-size: 11.5px; }
    table.compact th, table.compact td { padding: 6px 6px; font-size: 12.5px; }
    th.java { color: var(--am-java); } th.rust { color: var(--am-rust); }
    td.gain { color: var(--am-green); font-weight: 600; }
    .inputs { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .inputs input, .field input[type="text"], .field input[type="number"] { width: 100%; min-width: 0; }
    .field.slider { gap: 6px; }
    .field.slider input[type="range"] { width: 100%; accent-color: var(--am-blue); }
    .ticks { display: flex; justify-content: space-between; font-size: 10.5px; color: var(--am-ink-soft); }
    .kpis { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .kpi { border: 1px solid var(--am-line); border-radius: 9px; padding: 10px 12px; display: flex; flex-direction: column; gap: 2px; }
    .kpi.accent { background: var(--am-navy); color: #fff; border-color: var(--am-navy); }
    .kpi.accent .muted { color: #b8c7da; }
    .kpi-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--am-ink-soft); font-weight: 600; }
    .kpi.accent .kpi-label { color: #b8c7da; }
    .kpi-value { font-size: 22px; font-weight: 600; font-variant-numeric: tabular-nums; }
    .takeaway { margin: 0; padding: 10px 12px; border-radius: 8px; background: var(--am-sky-soft); color: var(--am-navy); line-height: 1.45; }
    .verdict { display: flex; flex-direction: column; gap: 4px; padding: 10px 12px; border-radius: 8px; border: 1px solid var(--am-line); background: #f8fafc; }
    .verdict.validated { background: var(--am-green-soft); border-color: #b7e4c7; color: #0b5c33; }
    .verdict.partial, .verdict.running { background: var(--am-amber-soft); border-color: #f5d7a1; color: #7a4a00; }
    .verdict.failed { background: var(--am-red-soft); border-color: #f3bcbc; color: #8a1c1c; }
    progress { width: 100%; height: 6px; accent-color: var(--am-blue); }
    .actions { display: flex; justify-content: space-between; align-items: flex-end; gap: 10px; }
    .field.inline input { width: 90px; }
    .file { cursor: pointer; }
    .panel-foot { margin-top: auto; display: flex; justify-content: space-between; align-items: center; padding: 12px 18px; border-top: 1px solid var(--am-line); }
  `,
})
export class CostPanel {
  readonly bench = inject(BenchService);
  readonly validation = inject(ValidationService);
  private readonly settings = inject(SettingsService);

  readonly s = this.settings.settings;
  readonly impact = this.bench.impact;
  readonly verdict = this.validation.verdict;
  readonly measuredOn = computed(() => formatDate(this.bench.summary()?.measuredAt));

  constructor() {
    void this.bench.loadReference();
  }

  set<K extends keyof ReturnType<typeof this.s>>(key: K, value: ReturnType<typeof this.s>[K]): void {
    const v = typeof this.s()[key] === 'number' ? Number(value) : value;
    if (typeof v === 'number' && !isFinite(v)) return;
    this.settings.patch({ [key]: v } as Partial<ReturnType<typeof this.s>>);
  }

  onFile(ev: Event): void {
    const file = (ev.target as HTMLInputElement).files?.[0];
    if (file) void this.bench.loadFromFile(file);
    (ev.target as HTMLInputElement).value = '';
  }

  resetAll(): void {
    this.settings.reset();
    this.validation.reset();
    void this.bench.loadReference();
  }

  fmt(v: number | undefined, decimals: number): string {
    if (v === undefined) return '—';
    return v.toLocaleString('en-US', { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
  }
}
