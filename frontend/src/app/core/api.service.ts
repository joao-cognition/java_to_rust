import { Injectable } from '@angular/core';
import { Probe, SERVICES, ServiceKind } from './models';

@Injectable({ providedIn: 'root' })
export class ApiService {
  url(kind: ServiceKind, path: string): string {
    const p = path.startsWith('/') ? path : `/${path}`;
    return `${SERVICES[kind].base}/api${p}`;
  }

  /** One request, never throws: status 0 means the proxy target is down. */
  async probe(kind: ServiceKind, path: string): Promise<Probe> {
    const t0 = performance.now();
    try {
      const res = await fetch(this.url(kind, path), { headers: { accept: 'application/json' } });
      const text = await res.text();
      const ms = performance.now() - t0;
      let body: unknown = null;
      try {
        body = text ? JSON.parse(text) : null;
      } catch {
        body = text;
      }
      if ([502, 503, 504].includes(res.status) && (body === null || typeof body !== 'object')) {
        return { status: 0, ms, bytes: 0, body: null, text: '', error: `proxy ${res.status}: nothing listening on :${SERVICES[kind].port}` };
      }
      return { status: res.status, ms, bytes: text.length, body, text };
    } catch (err) {
      return { status: 0, ms: performance.now() - t0, bytes: 0, body: null, text: '', error: (err as Error).message };
    }
  }
}

export function ok(status: number): boolean {
  return status >= 200 && status < 300;
}

/** Stable JSON (sorted keys) so serializer field order does not count as a difference. */
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

/** JSON-path list of leaf differences between two values (capped). */
export function diffPaths(a: unknown, b: unknown, path = '$', out: string[] = [], cap = 12): string[] {
  if (out.length >= cap) return out;
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) out.push(`${path}.length ${a.length} != ${b.length}`);
    for (let i = 0; i < Math.min(a.length, b.length); i++) diffPaths(a[i], b[i], `${path}[${i}]`, out, cap);
    return out;
  }
  if (a && b && typeof a === 'object' && typeof b === 'object') {
    const ao = a as Record<string, unknown>;
    const bo = b as Record<string, unknown>;
    for (const k of new Set([...Object.keys(ao), ...Object.keys(bo)])) {
      if (!(k in ao)) out.push(`${path}.${k} missing in java`);
      else if (!(k in bo)) out.push(`${path}.${k} missing in rust`);
      else diffPaths(ao[k], bo[k], `${path}.${k}`, out, cap);
      if (out.length >= cap) break;
    }
    return out;
  }
  if (a !== b) out.push(`${path}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`);
  return out;
}

export function percentile(samples: number[], p: number): number {
  if (!samples.length) return 0;
  const s = [...samples].sort((x, y) => x - y);
  return s[Math.min(s.length - 1, Math.floor(p * s.length))];
}
