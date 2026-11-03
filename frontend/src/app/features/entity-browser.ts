import { Component, computed, inject, signal } from '@angular/core';
import { DecimalPipe } from '@angular/common';
import { ApiService } from '../core/api.service';
import { NamedRef, Owner, Pet, ServiceKind, Vet, Visit } from '../core/models';

type Entity = 'owners' | 'pets' | 'vets' | 'visits' | 'pettypes' | 'specialties';

const TABS: Array<{ id: Entity; label: string }> = [
  { id: 'owners', label: 'Owners' },
  { id: 'pets', label: 'Pets' },
  { id: 'vets', label: 'Vets' },
  { id: 'visits', label: 'Visits' },
  { id: 'pettypes', label: 'Pet types' },
  { id: 'specialties', label: 'Specialties' },
];

@Component({
  selector: 'app-entity-browser',
  imports: [DecimalPipe],
  template: `
    <section class="card">
      <header>
        <nav class="tabs">
          @for (t of tabs; track t.id) {
            <button [class.active]="entity() === t.id" (click)="select(t.id)">{{ t.label }}</button>
          }
        </nav>
        <div class="source">
          <span class="muted">Served by</span>
          <div class="seg">
            <button [class.active]="source() === 'java'" (click)="setSource('java')">Java</button>
            <button [class.active]="source() === 'rust'" (click)="setSource('rust')">Rust</button>
          </div>
          @if (ms() !== null) { <span class="chip plain muted">{{ ms() | number: '1.0-0' }} ms · {{ rowCount() }} rows</span> }
        </div>
      </header>

      @if (error()) {
        <p class="error">{{ error() }}</p>
      } @else {
        <div class="scroll">
          @switch (entity()) {
            @case ('owners') {
              <table class="grid">
                <thead><tr><th>#</th><th>Name</th><th>City</th><th>Telephone</th><th>Pets</th></tr></thead>
                <tbody>
                  @for (o of owners(); track o.id) {
                    <tr>
                      <td class="num">{{ o.id }}</td>
                      <td>{{ o.firstName }} {{ o.lastName }}</td>
                      <td>{{ o.city }}</td>
                      <td class="mono">{{ o.telephone }}</td>
                      <td>
                        @for (p of o.pets; track p.id) {
                          <span class="chip plain muted">{{ p.name }} · {{ p.type.name }}@if (p.visits?.length) { · {{ p.visits!.length }} visits}</span>
                        }
                      </td>
                    </tr>
                  }
                </tbody>
              </table>
            }
            @case ('pets') {
              <table class="grid">
                <thead><tr><th>#</th><th>Name</th><th>Type</th><th>Born</th><th>Owner</th><th>Visits</th></tr></thead>
                <tbody>
                  @for (p of pets(); track p.id) {
                    <tr>
                      <td class="num">{{ p.id }}</td><td>{{ p.name }}</td><td>{{ p.type.name }}</td>
                      <td class="mono">{{ p.birthDate }}</td><td class="num">{{ p.ownerId }}</td><td class="num">{{ p.visits?.length ?? 0 }}</td>
                    </tr>
                  }
                </tbody>
              </table>
            }
            @case ('vets') {
              <table class="grid">
                <thead><tr><th>#</th><th>Name</th><th>Specialties</th></tr></thead>
                <tbody>
                  @for (v of vets(); track v.id) {
                    <tr>
                      <td class="num">{{ v.id }}</td><td>{{ v.firstName }} {{ v.lastName }}</td>
                      <td>@for (s of v.specialties; track s.id) { <span class="chip plain muted">{{ s.name }}</span> } @empty { <span class="muted">none</span> }</td>
                    </tr>
                  }
                </tbody>
              </table>
            }
            @case ('visits') {
              <table class="grid">
                <thead><tr><th>#</th><th>Date</th><th>Description</th><th>Pet</th></tr></thead>
                <tbody>
                  @for (v of visits(); track v.id) {
                    <tr><td class="num">{{ v.id }}</td><td class="mono">{{ v.date }}</td><td>{{ v.description }}</td><td class="num">{{ v.petId }}</td></tr>
                  }
                </tbody>
              </table>
            }
            @default {
              <table class="grid">
                <thead><tr><th>#</th><th>Name</th></tr></thead>
                <tbody>
                  @for (r of refs(); track r.id) {
                    <tr><td class="num">{{ r.id }}</td><td>{{ r.name }}</td></tr>
                  }
                </tbody>
              </table>
            }
          }
        </div>
      }
    </section>
  `,
  styles: `
    header { display: flex; justify-content: space-between; align-items: center; gap: 16px; padding: 8px 12px; border-bottom: 1px solid var(--am-line); flex-wrap: wrap; }
    .tabs { display: flex; gap: 2px; }
    .tabs button { border: none; background: transparent; padding: 8px 12px; border-radius: 7px; color: var(--am-ink-soft); font-weight: 600; cursor: pointer; }
    .tabs button.active { background: var(--am-sky-soft); color: var(--am-blue); }
    .source { display: flex; align-items: center; gap: 10px; font-size: 12.5px; }
    .seg { display: inline-flex; border: 1px solid var(--am-line); border-radius: 7px; overflow: hidden; }
    .seg button { border: none; background: #fff; padding: 5px 12px; cursor: pointer; font-weight: 600; color: var(--am-ink-soft); }
    .seg button.active { background: var(--am-navy); color: #fff; }
    .scroll { max-height: 420px; overflow: auto; }
    .error { padding: 18px; color: var(--am-red); }
    .chip + .chip { margin-left: 4px; }
  `,
})
export class EntityBrowser {
  private readonly api = inject(ApiService);
  readonly tabs = TABS;

  readonly entity = signal<Entity>('owners');
  readonly source = signal<ServiceKind>('java');
  readonly ms = signal<number | null>(null);
  readonly error = signal('');
  readonly rows = signal<unknown[]>([]);
  readonly rowCount = computed(() => this.rows().length);

  readonly owners = computed(() => this.rows() as Owner[]);
  readonly pets = computed(() => this.rows() as Pet[]);
  readonly vets = computed(() => this.rows() as Vet[]);
  readonly visits = computed(() => this.rows() as Visit[]);
  readonly refs = computed(() => this.rows() as NamedRef[]);

  constructor() {
    void this.load();
  }

  select(e: Entity): void {
    this.entity.set(e);
    void this.load();
  }

  setSource(s: ServiceKind): void {
    this.source.set(s);
    void this.load();
  }

  private requestId = 0;

  private async load(): Promise<void> {
    const id = ++this.requestId;
    this.error.set('');
    this.ms.set(null);
    this.rows.set([]);
    try {
      const r = await this.api.get<unknown[]>(this.source(), `/${this.entity()}`);
      if (id !== this.requestId) return;
      this.rows.set(r.data);
      this.ms.set(r.ms);
    } catch (err) {
      if (id !== this.requestId) return;
      this.rows.set([]);
      const status = (err as { status?: number })?.status;
      this.error.set(
        this.source() === 'rust'
          ? status
            ? `Rust service answered HTTP ${status} for /${this.entity()} — this endpoint is not ported yet.`
            : 'Rust service unreachable (expected on port 9977).'
          : status
            ? `Java service answered HTTP ${status}.`
            : 'Java service unreachable (expected on port 9966).',
      );
    }
  }
}
