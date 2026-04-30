import { useMemo } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  AlertTriangle,
  Activity,
  Brain,
  CheckCircle2,
  Clock3,
  Database,
  RefreshCw,
  ServerCog,
  XCircle,
} from 'lucide-react';

import { getIngestionStats, getRerankerMetrics, type IngestionSourceEntry } from '../api/analytics';
import { mlRequest, request } from '../api/client';
import { getMlReady, type MlReadyResponse } from '../api/ml-health';
import { getSourceHealth } from '../api/source-health';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { SectionCard } from '../components/ui/SectionCard';
import { cn } from '../lib/cn';
import { formatFallbackLabel } from '../lib/format';
import { readProfileId } from '../lib/profileSession';

type MlHealthResponse = {
  status: string;
  service: string;
  engine_api_base_url: string;
  llm_provider: string;
};

type EngineReadyResponse = {
  status: string;
  service?: string;
  components?: {
    database?: {
      status?: string;
      latency_ms?: number;
    };
    ingestion?: {
      status?: string;
      last_run_at?: string | null;
    };
  };
};

type DebugIssue = {
  key: string;
  title: string;
  detail: string;
  tone: 'warning' | 'danger';
};

async function getMlHealth(): Promise<MlHealthResponse> {
  return mlRequest<MlHealthResponse>('/health');
}

async function getEngineReady(): Promise<EngineReadyResponse> {
  return request<EngineReadyResponse>('/ready');
}

function parseApiDate(value: string | null | undefined): Date | null {
  if (!value) return null;
  const date = new Date(value.endsWith('Z') ? value : `${value}Z`);
  return Number.isNaN(date.getTime()) ? null : date;
}

function formatRelativeTime(value: string | null | undefined): string {
  const date = parseApiDate(value);
  if (!date) return 'Never';

  const diffMin = Math.floor((Date.now() - date.getTime()) / 60_000);
  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin} min ago`;

  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  return `${Math.floor(diffHours / 24)}d ago`;
}

function formatNumber(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value)) return '—';
  return new Intl.NumberFormat('en-US').format(value);
}

function statusVariant(status: string | undefined): 'success' | 'warning' | 'danger' | 'muted' {
  if (status === 'ok' || status === 'ready' || status === 'completed') return 'success';
  if (status === 'degraded' || status === 'partial' || status === 'stale') return 'warning';
  if (status === 'failed' || status === 'error' || status === 'not_ready') return 'danger';
  return 'muted';
}

function StatusBadge({ status }: { status: string | undefined }) {
  const label = status ? formatFallbackLabel(status) : 'Unknown';
  return <Badge variant={statusVariant(status)}>{label}</Badge>;
}

function MetricTile({
  label,
  value,
  detail,
}: {
  label: string;
  value: string | number;
  detail?: string;
}) {
  return (
    <div className="rounded-lg border border-border/70 bg-surface-muted px-4 py-3">
      <p className="m-0 text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
        {label}
      </p>
      <p className="m-0 mt-2 text-xl font-semibold text-card-foreground">{value}</p>
      {detail ? <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">{detail}</p> : null}
    </div>
  );
}

function SourceRow({ source }: { source: IngestionSourceEntry }) {
  const failed = source.status === 'failed' || source.errors > 0;

  return (
    <div className="grid gap-3 rounded-lg border border-border/70 bg-surface-muted px-4 py-3 text-sm md:grid-cols-[minmax(0,1fr)_110px_120px_120px_120px] md:items-center">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <p className="m-0 truncate font-semibold text-card-foreground">{source.displayName}</p>
          <StatusBadge status={source.status} />
        </div>
        <p className="m-0 mt-1 text-xs text-muted-foreground">
          Last success/failure marker: {formatRelativeTime(source.lastRunAt)}
        </p>
      </div>
      <MetricInline label="Indexed" value={formatNumber(source.count)} />
      <MetricInline label="Fetched" value={formatNumber(source.jobsFetched)} />
      <MetricInline label="Upserted" value={formatNumber(source.jobsUpserted)} />
      <MetricInline
        label="Errors"
        value={formatNumber(source.errors)}
        className={failed ? 'text-destructive' : undefined}
      />
    </div>
  );
}

function MetricInline({
  label,
  value,
  className,
}: {
  label: string;
  value: string;
  className?: string;
}) {
  return (
    <div>
      <p className="m-0 text-[11px] uppercase tracking-[0.12em] text-muted-foreground">{label}</p>
      <p className={cn('m-0 mt-1 font-semibold text-card-foreground', className)}>{value}</p>
    </div>
  );
}

export default function Debug() {
  const queryClient = useQueryClient();
  const profileId = readProfileId();

  const mlHealth = useQuery({
    queryKey: ['debug', 'ml-health'],
    queryFn: getMlHealth,
    retry: 0,
  });

  const mlReady = useQuery<MlReadyResponse>({
    queryKey: ['debug', 'ml-ready'],
    queryFn: getMlReady,
    retry: 0,
  });

  const engineReady = useQuery({
    queryKey: ['debug', 'engine-ready'],
    queryFn: getEngineReady,
    retry: 0,
  });

  const ingestionStats = useQuery({
    queryKey: ['debug', 'ingestion-stats'],
    queryFn: getIngestionStats,
    retry: 0,
  });

  const sourceHealth = useQuery({
    queryKey: ['debug', 'source-health'],
    queryFn: getSourceHealth,
    retry: 0,
  });

  const rerankerMetrics = useQuery({
    queryKey: ['debug', 'reranker-metrics', profileId ?? 'none'],
    queryFn: () => getRerankerMetrics(profileId!),
    enabled: !!profileId,
    retry: 0,
  });

  const issues = useMemo<DebugIssue[]>(() => {
    const nextIssues: DebugIssue[] = [];

    if (mlHealth.error) {
      nextIssues.push({
        key: 'ml-health',
        title: 'ML health unavailable',
        detail: mlHealth.error.message,
        tone: 'danger',
      });
    }

    if (mlReady.error) {
      nextIssues.push({
        key: 'ml-ready',
        title: 'ML readiness unavailable',
        detail: mlReady.error.message,
        tone: 'danger',
      });
    } else if (mlReady.data && mlReady.data.status !== 'ready') {
      nextIssues.push({
        key: 'ml-ready-status',
        title: 'ML readiness is degraded',
        detail: `Status: ${formatFallbackLabel(mlReady.data.status)}`,
        tone: statusVariant(mlReady.data.status) === 'danger' ? 'danger' : 'warning',
      });
    }

    if (engineReady.error) {
      nextIssues.push({
        key: 'engine-ready',
        title: 'Engine readiness unavailable',
        detail: engineReady.error.message,
        tone: 'danger',
      });
    }

    for (const source of ingestionStats.data?.sources ?? []) {
      if (source.status !== 'ok' || source.errors > 0) {
        nextIssues.push({
          key: `source-${source.source}`,
          title: `${source.displayName} ingestion needs attention`,
          detail: `${formatFallbackLabel(source.status)} with ${source.errors} recent errors.`,
          tone: source.status === 'failed' ? 'danger' : 'warning',
        });
      }
    }

    const metrics = rerankerMetrics.data;
    if (metrics?.summary.failedRunCount) {
      nextIssues.push({
        key: 'reranker-failures',
        title: 'Reranker training failures recorded',
        detail: `${metrics.summary.failedRunCount} failed run(s) in training history.`,
        tone: 'danger',
      });
    }
    if (metrics?.summary.lastWarningReason) {
      nextIssues.push({
        key: 'reranker-warning',
        title: 'Latest reranker warning',
        detail: metrics.summary.lastWarningReason,
        tone: 'warning',
      });
    }

    return nextIssues;
  }, [
    engineReady.error,
    ingestionStats.data?.sources,
    mlHealth.error,
    mlReady.data,
    mlReady.error,
    rerankerMetrics.data,
  ]);

  const sourceRows = ingestionStats.data?.sources ?? [];
  const latestRun = rerankerMetrics.data?.runs[0];

  return (
    <Page>
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="m-0 text-xs font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Operator diagnostics
          </p>
          <h1 className="m-0 mt-2 text-2xl font-semibold text-card-foreground">Debug Dashboard</h1>
          <p className="m-0 mt-2 max-w-3xl text-sm leading-6 text-muted-foreground">
            Runtime health for ML, reranker training, ingestion runs, and recent observable errors.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          onClick={() => queryClient.invalidateQueries({ queryKey: ['debug'] })}
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </Button>
      </div>

      <SectionCard
        title="ML Service Health"
        description="Provider selection and readiness from the ML sidecar."
        icon={ServerCog}
        eyebrow="ML"
      >
        <div className="mb-4 flex flex-wrap gap-2">
          <StatusBadge status={mlHealth.data?.status ?? mlReady.data?.status} />
          <Badge variant="muted">Provider {mlHealth.data?.llm_provider ?? 'unknown'}</Badge>
        </div>
        <div className="grid gap-3 md:grid-cols-3">
          <MetricTile
            label="Sidecar"
            value={formatFallbackLabel(mlReady.data?.components?.ml_sidecar?.status ?? 'unknown')}
            detail={mlHealth.error ? mlHealth.error.message : (mlHealth.data?.service ?? 'ml')}
          />
          <MetricTile
            label="Engine DB Probe"
            value={`${mlReady.data?.components?.database?.latency_ms ?? 0} ms`}
            detail={formatFallbackLabel(mlReady.data?.components?.database?.status ?? 'unknown')}
          />
          <MetricTile
            label="Last Enrichment Latency"
            value="Not reported"
            detail="Current ML API exposes readiness latency, not per-enrichment timing."
          />
        </div>
      </SectionCard>

      <SectionCard
        title="Reranker Status"
        description="Training state and latest persisted reranker metrics for the active profile."
        icon={Brain}
        eyebrow="Ranking"
      >
        {!profileId ? (
          <EmptyState message="No active profile selected." className="px-4 py-4 text-left" />
        ) : (
          <>
            <div className="mb-4 flex flex-wrap gap-2">
              <StatusBadge status={rerankerMetrics.data?.state.lastTrainingStatus ?? 'idle'} />
              <Badge variant="muted">
                Mode{' '}
                {latestRun?.modelType ? formatFallbackLabel(latestRun.modelType) : 'not trained'}
              </Badge>
            </div>
            <div className="grid gap-3 md:grid-cols-4">
              <MetricTile
                label="Model Version"
                value={rerankerMetrics.data?.state.lastArtifactVersion ?? 'none'}
              />
              <MetricTile
                label="Training Samples"
                value={formatNumber(rerankerMetrics.data?.state.examplesSinceRetrain)}
                detail="Examples since last retrain"
              />
              <MetricTile
                label="Last Retrained"
                value={formatRelativeTime(rerankerMetrics.data?.state.lastRetrainedAt)}
              />
              <MetricTile
                label="Failed Runs"
                value={formatNumber(rerankerMetrics.data?.summary.failedRunCount)}
                detail={`${formatNumber(rerankerMetrics.data?.summary.warningRunCount)} warning(s)`}
              />
            </div>
          </>
        )}
      </SectionCard>

      <SectionCard
        title="Ingestion Run History"
        description="Latest per-source run state and ingestion counters."
        icon={Database}
        eyebrow="Sources"
      >
        <div className="mb-4 grid gap-3 md:grid-cols-4">
          <MetricTile label="Total Jobs" value={formatNumber(ingestionStats.data?.totalJobs)} />
          <MetricTile label="Active Jobs" value={formatNumber(ingestionStats.data?.activeJobs)} />
          <MetricTile
            label="Inactive Jobs"
            value={formatNumber(ingestionStats.data?.inactiveJobs)}
          />
          <MetricTile
            label="Last Ingest"
            value={formatRelativeTime(ingestionStats.data?.lastIngestedAt)}
          />
        </div>
        <div className="space-y-3">
          {sourceRows.length > 0 ? (
            sourceRows.map((source) => <SourceRow key={source.source} source={source} />)
          ) : (
            <EmptyState message="No ingestion runs have reported yet." className="px-4 py-4" />
          )}
        </div>
      </SectionCard>

      <SectionCard
        title="Recent Error Summary"
        description="Safe summary from readiness checks, ingestion run counters, and reranker metrics."
        icon={Activity}
        eyebrow="Errors"
      >
        {issues.length === 0 ? (
          <div className="flex items-center gap-3 rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
            <CheckCircle2 className="h-4 w-4 shrink-0" />
            No recent observable errors from available debug sources.
          </div>
        ) : (
          <div className="space-y-3">
            {issues.map((issue) => {
              const Icon = issue.tone === 'danger' ? XCircle : AlertTriangle;
              return (
                <div
                  key={issue.key}
                  className={cn(
                    'flex items-start gap-3 rounded-lg border px-4 py-3 text-sm',
                    issue.tone === 'danger'
                      ? 'border-destructive/30 bg-destructive/10 text-destructive'
                      : 'border-warning/30 bg-warning/10 text-warning',
                  )}
                >
                  <Icon className="mt-0.5 h-4 w-4 shrink-0" />
                  <div className="min-w-0">
                    <p className="m-0 font-semibold">{issue.title}</p>
                    <p className="m-0 mt-1 break-words text-xs leading-5 opacity-90">
                      {issue.detail}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        )}
        <div className="mt-4 grid gap-3 md:grid-cols-3">
          <MetricTile
            label="Engine Readiness"
            value={formatFallbackLabel(engineReady.data?.status ?? 'unknown')}
            detail={`DB ${engineReady.data?.components?.database?.latency_ms ?? 0} ms`}
          />
          <MetricTile
            label="Source Health"
            value={`${sourceHealth.data?.filter((source) => source.degraded).length ?? 0} degraded`}
            detail={`${sourceHealth.data?.length ?? 0} source(s) checked`}
          />
          <MetricTile
            label="Last Ingestion Signal"
            value={formatRelativeTime(engineReady.data?.components?.ingestion?.last_run_at)}
            detail={formatFallbackLabel(
              engineReady.data?.components?.ingestion?.status ?? 'unknown',
            )}
          />
        </div>
        <div className="mt-4 flex items-center gap-2 text-xs text-muted-foreground">
          <Clock3 className="h-3.5 w-3.5" />
          <span>Debug route is enabled with VITE_ENABLE_DEBUG_PAGE=true.</span>
        </div>
      </SectionCard>
    </Page>
  );
}
