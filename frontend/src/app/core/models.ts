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
