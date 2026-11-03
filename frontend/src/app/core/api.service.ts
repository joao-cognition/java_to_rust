import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { firstValueFrom } from 'rxjs';
import { NamedRef, Owner, Pet, ServiceKind, Visit, Vet } from './models';
import { SettingsService } from './settings.service';

export interface TimedResult<T> { data: T; ms: number; }

export interface HealthStatus { kind: ServiceKind; up: boolean; ms: number; detail: string; }

@Injectable({ providedIn: 'root' })
export class ApiService {
  private readonly http = inject(HttpClient);
  private readonly settings = inject(SettingsService);

  baseUrl(kind: ServiceKind): string {
    const s = this.settings.settings();
    return (kind === 'java' ? s.javaBaseUrl : s.rustBaseUrl).replace(/\/+$/, '');
  }

  apiUrl(kind: ServiceKind, path: string): string {
    return `${this.baseUrl(kind)}/api${path.startsWith('/') ? path : '/' + path}`;
  }

  async health(kind: ServiceKind): Promise<HealthStatus> {
    const started = performance.now();
    try {
      const body = await firstValueFrom(
        this.http.get<{ status?: string }>(`${this.baseUrl(kind)}/actuator/health`),
      );
      const ms = performance.now() - started;
      const up = (body?.status ?? '').toUpperCase() === 'UP';
      return { kind, up, ms, detail: up ? 'UP' : (body?.status ?? 'unknown') };
    } catch (err: unknown) {
      return { kind, up: false, ms: performance.now() - started, detail: describe(err) };
    }
  }

  async get<T>(kind: ServiceKind, path: string): Promise<TimedResult<T>> {
    const started = performance.now();
    const data = await firstValueFrom(this.http.get<T>(this.apiUrl(kind, path)));
    return { data, ms: performance.now() - started };
  }

  /** Raw fetch used by the validation probe: returns status + parsed body without throwing. */
  async probe(kind: ServiceKind, path: string): Promise<{ status: number; ms: number; body: unknown }> {
    const started = performance.now();
    try {
      const res = await fetch(this.apiUrl(kind, path), { headers: { Accept: 'application/json' } });
      const text = await res.text();
      let body: unknown = text;
      try { body = JSON.parse(text); } catch { /* non-JSON body */ }
      return { status: res.status, ms: performance.now() - started, body };
    } catch {
      return { status: 0, ms: performance.now() - started, body: null };
    }
  }

  owners(kind: ServiceKind) { return this.get<Owner[]>(kind, '/owners'); }
  pets(kind: ServiceKind) { return this.get<Pet[]>(kind, '/pets'); }
  vets(kind: ServiceKind) { return this.get<Vet[]>(kind, '/vets'); }
  visits(kind: ServiceKind) { return this.get<Visit[]>(kind, '/visits'); }
  petTypes(kind: ServiceKind) { return this.get<NamedRef[]>(kind, '/pettypes'); }
  specialties(kind: ServiceKind) { return this.get<NamedRef[]>(kind, '/specialties'); }
}

export function describe(err: unknown): string {
  if (err && typeof err === 'object' && 'status' in err) {
    const status = (err as { status: number }).status;
    return status === 0 || status === 502 || status === 503 || status === 504 ? 'unreachable' : `HTTP ${status}`;
  }
  return 'unreachable';
}
