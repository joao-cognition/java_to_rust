export interface NamedRef { id: number; name: string; }

export interface Visit { id: number; date: string; description: string; petId?: number; }

export interface Pet {
  id: number;
  name: string;
  birthDate: string;
  type: NamedRef;
  ownerId?: number;
  visits?: Visit[];
}

export interface Owner {
  id: number;
  firstName: string;
  lastName: string;
  address: string;
  city: string;
  telephone: string;
  pets: Pet[];
}

export interface Vet { id: number; firstName: string; lastName: string; specialties: NamedRef[]; }

export type ServiceKind = 'java' | 'rust';

export interface ServiceMetrics {
  label: string;
  cpuCoresAtFixedRate: number;
  cpuSecondsPerMillionRequests?: number;
  costPerMillionRequestsUsd: number;
  peakRequestsPerSecond: number;
  p50Ms?: number;
  p99Ms: number;
  memoryMb: number;
  startupSeconds: number;
  dbQueriesPerRequest?: number;
  failedRequests?: number;
}

export interface BenchSummary {
  measuredAt: string;
  source?: string;
  host?: string;
  runs?: number;
  vcpuHourUsd: number;
  workload?: { ratePerSecond: number; durationSeconds: number; peakConcurrency?: number; mix?: string };
  services: { java: ServiceMetrics; rust: ServiceMetrics };
}
