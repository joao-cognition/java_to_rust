import { Injectable, effect, signal } from '@angular/core';

export interface ConsoleSettings {
  javaBaseUrl: string;
  rustBaseUrl: string;
  fleetVcpus: number;
  vcpuHourUsd: number;
  migratedSharePct: number;
  probeRequests: number;
}

const STORAGE_KEY = 'amadeus-console-settings';

export const DEFAULT_SETTINGS: ConsoleSettings = {
  javaBaseUrl: '/petclinic',
  rustBaseUrl: '/rust/petclinic',
  fleetVcpus: 10000,
  vcpuHourUsd: 0.0509,
  migratedSharePct: 1,
  probeRequests: 25,
};

@Injectable({ providedIn: 'root' })
export class SettingsService {
  readonly settings = signal<ConsoleSettings>(load());

  constructor() {
    effect(() => {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings()));
      } catch {
        /* storage unavailable: keep in memory only */
      }
    });
  }

  patch(partial: Partial<ConsoleSettings>): void {
    this.settings.update((s) => ({ ...s, ...partial }));
  }

  reset(): void {
    this.settings.set({ ...DEFAULT_SETTINGS });
  }
}

function load(): ConsoleSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
  } catch {
    /* fall through */
  }
  return { ...DEFAULT_SETTINGS };
}
