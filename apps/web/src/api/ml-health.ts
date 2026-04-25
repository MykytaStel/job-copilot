import { mlRequest } from './client';

export interface MlReadyCheck {
  name: string;
  status: 'ok' | 'degraded';
  detail?: string;
}

export interface MlReadyResponse {
  status: string;
  service: string;
  checks: MlReadyCheck[];
}

export function getMlReady(): Promise<MlReadyResponse> {
  return mlRequest<MlReadyResponse>('/ready');
}

export function isMlDegraded(ready: MlReadyResponse): boolean {
  return (
    ready.status === 'degraded' ||
    ready.checks.some((c) => c.name === 'enrichment_provider' && c.status === 'degraded')
  );
}
