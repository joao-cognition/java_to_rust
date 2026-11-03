import { DecimalPipe } from '@angular/common';
import { Component, computed, inject, signal } from '@angular/core';
import { ApiService, canonical, diffPaths, ok, percentile } from './core/api.service';
import { Probe, SERVICES, ServiceKind } from './core/models';

const QUICK = ['vets', 'specialties', 'pettypes', 'owners', 'owners/1', 'owners?lastName=Davis', 'pets/1', 'visits'];

interface Call {
  path: string;
  java: Probe;
  rust: Probe;
  diff: string[];
}

interface Burst {
  n: number;
  java: { p50: number; p99: number; errors: number };
  rust: { p50: number; p99: number; errors: number };
}

@Component({
  selector: 'api-pane',
  imports: [DecimalPipe],
  template: `
    <div class="title">api <span class="dim">— same request to both services, raw response</span></div>

    <form class="row" (submit)="send(); $event.preventDefault()">
      <span class="dim">GET /petclinic/api/</span>
      <input class="path" [value]="path()" (input)="path.set($any($event.target).value)" spellcheck="false" autocomplete="off" />
      <button type="submit" [disabled]="busy()">send</button>
      <button type="button" (click)="burst()" [disabled]="busy()">x{{ burstN }}</button>
    </form>
    <div class="row quick">
      @for (q of quick; track q) {
        <a (click)="path.set(q); send()" [class.on]="q === path()">{{ q }}</a>
      }
    </div>

    @if (call(); as c) {
      <div class="cols">
        @for (k of kinds; track k) {
          <div class="col">
            <div class="status">
              <span class="svc {{ k }}">{{ k }}</span><span class="dim">:{{ services[k].port }}</span>
              <span class="st" [class.bad]="!isOk(c[k].status)">{{ c[k].status || 'down' }}</span>
              @if (c[k].status) {
                <span>{{ c[k].ms | number: '1.0-0' }}ms</span>
                <span class="dim">{{ c[k].bytes }}B</span>
              }
            </div>
            <pre class="body">{{ bodyText(c[k]) }}</pre>
          </div>
        }
      </div>
      <div class="diff" [class.good]="c.diff.length === 0 && isOk(c.rust.status)" [class.bad]="c.diff.length > 0">
        @if (!isOk(c.rust.status)) {
          diff: n/a — rust {{ c.rust.status ? 'HTTP ' + c.rust.status : 'not reachable on :' + services.rust.port }}
        } @else if (c.diff.length === 0) {
          diff: identical ({{ c.java.bytes }} B, key order ignored)
        } @else {
          diff: {{ c.diff.length }} difference{{ c.diff.length > 1 ? 's' : '' }}
          @for (d of c.diff; track d) {<div class="dpath">  {{ d }}</div>}
        }
      </div>
    }

    @if (lastBurst(); as b) {
      <pre class="burst">{{ burstText(b) }}</pre>
    }

    <div class="hist">
      @for (h of history(); track $index) {<div>{{ h }}</div>}
    </div>
  `,
  styles: `
    :host { display: flex; flex-direction: column; min-height: 0; gap: 6px; }
    .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
    .path { flex: 1; min-width: 160px; }
    .quick a { margin-right: 10px; }
    .cols { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; min-height: 0; }
    .col { display: flex; flex-direction: column; min-height: 0; border: 1px solid var(--line); }
    .status { display: flex; gap: 10px; padding: 3px 8px; white-space: nowrap; overflow: hidden; border-bottom: 1px solid var(--line); background: var(--bg2); }
    .st { color: var(--green); }
    .st.bad { color: var(--red); }
    .body { margin: 0; padding: 6px 8px; overflow: auto; max-height: 38vh; min-height: 120px; font-size: 12px; line-height: 1.35; }
    .diff { padding: 3px 0; }
    .diff.good { color: var(--green); }
    .diff.bad { color: var(--amber); }
    .dpath { color: var(--fg); }
    .burst { margin: 0; padding: 6px 8px; border: 1px solid var(--line); background: var(--bg2); }
    .hist { color: var(--dim); overflow: auto; max-height: 14vh; font-size: 12px; }
  `,
})
export class ApiPane {
  private readonly api = inject(ApiService);
  readonly services = SERVICES;
  readonly kinds: ServiceKind[] = ['java', 'rust'];
  readonly quick = QUICK;
  readonly burstN = 50;

  readonly path = signal('vets');
  readonly busy = signal(false);
  readonly call = signal<Call | null>(null);
  readonly lastBurst = signal<Burst | null>(null);
  readonly history = signal<string[]>([]);
  readonly isOk = ok;

  constructor() {
    void this.send();
  }

  async send(): Promise<void> {
    if (this.busy()) return;
    this.busy.set(true);
    const path = this.path().trim();
    const [java, rust] = await Promise.all([this.api.probe('java', path), this.api.probe('rust', path)]);
    const diff = ok(java.status) && ok(rust.status) && canonical(java.body) !== canonical(rust.body) ? diffPaths(java.body, rust.body) : [];
    this.call.set({ path, java, rust, diff });
    const verdict = !ok(rust.status) ? 'rust n/a' : diff.length ? `${diff.length} diff` : 'identical';
    this.push(`> GET /${path}  java ${java.status || 'down'} ${java.ms.toFixed(0)}ms  rust ${rust.status || 'down'} ${rust.ms.toFixed(0)}ms  ${verdict}`);
    this.busy.set(false);
  }

  async burst(): Promise<void> {
    if (this.busy()) return;
    this.busy.set(true);
    const path = this.path().trim();
    const run = async (k: ServiceKind) => {
      const ms: number[] = [];
      let errors = 0;
      for (let i = 0; i < this.burstN; i++) {
        const p = await this.api.probe(k, path);
        if (ok(p.status)) ms.push(p.ms);
        else errors++;
      }
      return { p50: percentile(ms, 0.5), p99: percentile(ms, 0.99), errors };
    };
    const [java, rust] = await Promise.all([run('java'), run('rust')]);
    this.lastBurst.set({ n: this.burstN, java, rust });
    this.push(`> ${this.burstN}x GET /${path}  java p50=${java.p50.toFixed(1)} p99=${java.p99.toFixed(1)}  rust ${rust.errors === this.burstN ? 'n/a' : `p50=${rust.p50.toFixed(1)} p99=${rust.p99.toFixed(1)}`}`);
    this.busy.set(false);
  }

  bodyText(p: Probe): string {
    if (p.status === 0) return `# ${p.error ?? 'connection failed'}`;
    if (typeof p.body === 'string') return p.text;
    return JSON.stringify(p.body, null, 2);
  }

  burstText(b: Burst): string {
    const line = (k: ServiceKind, s: Burst['java']) =>
      s.errors === b.n
        ? `${k.padEnd(5)} n/a (${s.errors} errors)`
        : `${k.padEnd(5)} p50 ${s.p50.toFixed(1).padStart(6)} ms   p99 ${s.p99.toFixed(1).padStart(6)} ms   errors ${s.errors}`;
    return [`${b.n} sequential requests from this browser, GET /${this.path()}`, line('java', b.java), line('rust', b.rust)].join('\n');
  }

  private push(line: string): void {
    this.history.update((h) => [line, ...h].slice(0, 40));
  }
}
