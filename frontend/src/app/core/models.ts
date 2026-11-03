export type ServiceKind = 'java' | 'rust';

export const SERVICES: Record<ServiceKind, { base: string; port: number; label: string }> = {
  java: { base: '/petclinic', port: 9966, label: 'spring-boot' },
  rust: { base: '/rust/petclinic', port: 9977, label: 'axum' },
};

export interface Probe {
  status: number;
  ms: number;
  bytes: number;
  body: unknown;
  text: string;
  error?: string;
}

export interface Sample {
  t: number;
  cpuCores: number;
  rps: number;
  p50Ms?: number;
  p99Ms: number;
  rssMb: number;
}

export interface ServiceSeries {
  label: string;
  samples: Sample[];
}

export interface ServiceSummary {
  cpuCores: number;
  p99Ms: number;
  rssMb: number;
  peakRps?: number;
}

export interface Timeseries {
  measuredAt: string;
  host?: string;
  db?: string;
  ratePerSecond: number;
  durationSeconds: number;
  services: Partial<Record<ServiceKind, ServiceSeries>>;
  summary: Partial<Record<ServiceKind, ServiceSummary>>;
}
