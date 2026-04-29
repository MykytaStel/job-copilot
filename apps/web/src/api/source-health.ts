import { request } from './client';

export interface SourceHealthRun {
  runAt: string;
  jobsFetched: number;
  jobsUpserted: number;
  errors: number;
  durationMs: number;
  status: string;
}

export interface SourceHealthItem {
  source: string;
  displayName: string;
  status: string;
  degraded: boolean;
  lastRun: SourceHealthRun | null;
}

interface EngineSourceHealthRun {
  run_at: string;
  jobs_fetched: number;
  jobs_upserted: number;
  errors: number;
  duration_ms: number;
  status: string;
}

interface EngineSourceHealthItem {
  source: string;
  display_name: string;
  status: string;
  degraded: boolean;
  last_run: EngineSourceHealthRun | null;
}

interface EngineSourceHealthResponse {
  sources: EngineSourceHealthItem[];
}

export async function getSourceHealth(): Promise<SourceHealthItem[]> {
  const response = await request<EngineSourceHealthResponse>('/api/v1/sources/health');
  return response.sources.map((source) => ({
    source: source.source,
    displayName: source.display_name,
    status: source.status,
    degraded: source.degraded,
    lastRun: source.last_run
      ? {
          runAt: source.last_run.run_at,
          jobsFetched: source.last_run.jobs_fetched,
          jobsUpserted: source.last_run.jobs_upserted,
          errors: source.last_run.errors,
          durationMs: source.last_run.duration_ms,
          status: source.last_run.status,
        }
      : null,
  }));
}
