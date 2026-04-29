import { mlRequest } from './client';

export interface MlReadyCheck {
  name: string;
  status: 'ok' | 'degraded';
  detail?: string;
}

export interface MlReadyComponents {
  database?: {
    status?: string;
    latency_ms?: number;
  };
  ml_sidecar?: {
    status?: string;
  };
  ingestion?: {
    status?: string;
    last_run_at?: string | null;
  };
}

export interface MlReadyResponse {
  status: string;
  service?: string;
  checks?: MlReadyCheck[];
  components?: MlReadyComponents;
}

export function getMlReady(): Promise<MlReadyResponse> {
  return mlRequest<MlReadyResponse>('/ready');
}

export function isMlDegraded(ready: MlReadyResponse): boolean {
  if (ready.status !== 'ready') {
    return true;
  }

  const checks = Array.isArray(ready.checks) ? ready.checks : [];
  return (
    checks.some((c) => c.name === 'enrichment_provider' && c.status === 'degraded') ||
    Object.values(ready.components ?? {}).some((component) => component?.status === 'degraded')
  );
}
