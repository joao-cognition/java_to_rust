import { Component, inject, signal } from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { ApiService, HealthStatus } from '../core/api.service';
import { ServiceKind } from '../core/models';

interface Counter { label: string; path: string; count: number | null; ms: number | null; error: string; }

interface ScopeRow { method: string; path: string; label: string; rust: 'migrated' | 'pending' | 'unknown'; }

const SCOPE: Array<Omit<ScopeRow, 'rust'>> = [
  { method: 'GET', path: '/vets', label: 'Vets + specialties' },
  { method: 'GET', path: '/specialties', label: 'Specialties' },
  { method: 'GET', path: '/pettypes', label: 'Pet types' },
  { method: 'GET', path: '/owners', label: 'Owner list / search' },
  { method: 'GET', path: '/owners/1', label: 'Owner lookup (nested pets, visits)' },
  { method: 'GET', path: '/pets/1', label: 'Pet lookup' },
  { method: 'GET', path: '/visits', label: 'Visits' },
  { method: 'GET', path: '/pets', label: 'Pets' },
];

@Component({
  selector: 'app-service-overview',
  imports: [DecimalPipe],
  template: `
    <section class="hero card">
      <div class="hero-text">
        <p class="eyebrow">Travel, Shopping &amp; Reservation · Reference &amp; lookup service</p>
        <h1>PetClinic REST — the Java service we are porting</h1>
        <p class="muted">
          Spring Boot 4 / Hibernate / Tomcat, {{ endpointsCount }} REST endpoints under <span class="mono">/petclinic/api</span>.
          The read hot path (reference lists, record lookup, search) is the first slice moving to Rust; both services run against the same PostgreSQL.
        </p>
      </div>
      <div class="health">
        @for (h of health(); track h.kind) {
          <div class="health-row">
            <span class="chip plain" [class.java]="h.kind === 'java'" [class.rust]="h.kind === 'rust'">{{ h.kind === 'java' ? 'Java · :9966' : 'Rust · :9977' }}</span>
            <span class="chip" [class.ok]="h.up" [class.bad]="!h.up">{{ h.up ? 'UP' : h.detail }}</span>
            <span class="muted">{{ h.ms | number: '1.0-0' }} ms</span>
          </div>
        }
        <button class="btn ghost sm" (click)="refresh()" [disabled]="loading()">Refresh</button>
      </div>
    </section>

    <section class="counters">
      @for (c of counters(); track c.path) {
        <div class="card counter">
          <span class="label">{{ c.label }}</span>
          @if (c.error) {
            <span class="value bad">—</span>
            <span class="muted">{{ c.error }}</span>
          } @else {
            <span class="value">{{ c.count === null ? '…' : (c.count | number) }}</span>
            <span class="muted">GET <span class="mono">{{ c.path }}</span>@if (c.ms !== null) { · {{ c.ms | number: '1.0-0' }} ms}</span>
          }
        </div>
      }
    </section>

    <section class="card scope">
      <header>
        <h2>Migration scope</h2>
        <span class="muted">{{ migratedCount() }} / {{ scope().length }} read endpoints served by Rust</span>
      </header>
      <table class="grid">
        <thead><tr><th>Endpoint</th><th>Purpose</th><th>Java</th><th>Rust</th></tr></thead>
        <tbody>
          @for (r of scope(); track r.path) {
            <tr>
              <td class="mono">{{ r.method }} {{ r.path }}</td>
              <td>{{ r.label }}</td>
              <td><span class="chip ok">Serving</span></td>
              <td>
                @switch (r.rust) {
                  @case ('migrated') { <span class="chip ok">Migrated</span> }
                  @case ('pending') { <span class="chip warn">Pending</span> }
                  @default { <span class="chip muted">Not reachable</span> }
                }
              </td>
            </tr>
          }
        </tbody>
      </table>
    </section>
  `,
  styles: `
    :host { display: flex; flex-direction: column; gap: 18px; }
    .hero { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 18px 24px; padding: 22px 24px; }
    .hero-text { flex: 1 1 420px; min-width: 0; }
    .eyebrow { margin: 0 0 6px; font-size: 11.5px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--am-blue); font-weight: 700; }
    h1 { font-size: 22px; margin-bottom: 8px; }
    .hero-text p.muted { margin: 0; max-width: 72ch; line-height: 1.5; }
    .health { display: flex; flex-direction: column; gap: 8px; align-items: flex-start; min-width: 260px; }
    .health-row { display: flex; gap: 8px; align-items: center; }
    .counters { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 12px; }
    .counter { padding: 14px 16px; display: flex; flex-direction: column; gap: 4px; }
    .counter .label { font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--am-ink-soft); font-weight: 600; }
    .counter .value { font-size: 26px; font-weight: 600; font-variant-numeric: tabular-nums; }
    .counter .value.bad { color: var(--am-red); }
    .scope header { display: flex; justify-content: space-between; align-items: baseline; padding: 14px 16px 6px; }
    .scope h2 { font-size: 15px; }
  `,
})
export class ServiceOverview {
  private readonly api = inject(ApiService);
  readonly endpointsCount = 36;

  readonly health = signal<HealthStatus[]>([]);
  readonly loading = signal(false);
  readonly counters = signal<Counter[]>([
    { label: 'Owners', path: '/owners', count: null, ms: null, error: '' },
    { label: 'Pets', path: '/pets', count: null, ms: null, error: '' },
    { label: 'Visits', path: '/visits', count: null, ms: null, error: '' },
    { label: 'Vets', path: '/vets', count: null, ms: null, error: '' },
    { label: 'Pet types', path: '/pettypes', count: null, ms: null, error: '' },
    { label: 'Specialties', path: '/specialties', count: null, ms: null, error: '' },
  ]);
  readonly scope = signal<ScopeRow[]>(SCOPE.map((s) => ({ ...s, rust: 'unknown' })));
  readonly migratedCount = signal(0);

  constructor() {
    void this.refresh();
  }

  async refresh(): Promise<void> {
    this.loading.set(true);
    const [java, rust] = await Promise.all([this.api.health('java'), this.api.health('rust')]);
    this.health.set([java, rust]);

    await Promise.all(
      this.counters().map(async (c) => {
        try {
          const r = await this.api.get<unknown[]>('java', c.path);
          this.update(c.path, { count: r.data.length, ms: r.ms, error: '' });
        } catch (err) {
          this.update(c.path, { count: null, ms: null, error: describeErr(err) });
        }
      }),
    );

    const rows = await Promise.all(
      SCOPE.map(async (s): Promise<ScopeRow> => {
        if (!rust.up) return { ...s, rust: 'unknown' };
        const p = await this.api.probe('rust', s.path);
        return { ...s, rust: p.status >= 200 && p.status < 300 ? 'migrated' : 'pending' };
      }),
    );
    this.scope.set(rows);
    this.migratedCount.set(rows.filter((r) => r.rust === 'migrated').length);
    this.loading.set(false);
  }

  private update(path: string, patch: Partial<Counter>): void {
    this.counters.update((list) => list.map((c) => (c.path === path ? { ...c, ...patch } : c)));
  }
}

function describeErr(err: unknown): string {
  const status = (err as { status?: number })?.status;
  return status ? `HTTP ${status}` : 'Java service unreachable';
}

export type { ServiceKind };
