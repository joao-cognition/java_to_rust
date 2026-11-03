import { Component, inject, signal } from '@angular/core';
import { ApiPane } from './api-pane';
import { ApiService, ok } from './core/api.service';
import { SERVICES, ServiceKind } from './core/models';

@Component({
  selector: 'app-root',
  imports: [ApiPane],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  private readonly api = inject(ApiService);
  readonly services = SERVICES;
  readonly kinds: ServiceKind[] = ['java', 'rust'];
  readonly up = signal<Record<ServiceKind, string>>({ java: '…', rust: '…' });

  constructor() {
    void this.health();
    setInterval(() => void this.health(), 10_000);
  }

  private async health(): Promise<void> {
    const res = await Promise.all(this.kinds.map((k) => this.api.probe(k, '/pettypes')));
    this.up.set({
      java: ok(res[0].status) ? `up ${res[0].ms.toFixed(0)}ms` : 'down',
      rust: ok(res[1].status) ? `up ${res[1].ms.toFixed(0)}ms` : 'down',
    });
  }
}
