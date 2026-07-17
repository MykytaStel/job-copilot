import { Laptop2 } from 'lucide-react';

import type { MarketRemoteAdoption } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { titleCase } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

const workModeMeta = {
  remote: { label: 'Remote', className: 'bg-primary' },
  hybrid: { label: 'Hybrid', className: 'bg-success' },
  onsite: { label: 'On-site', className: 'bg-warning' },
  unknown: { label: 'Unknown', className: 'bg-muted-foreground/40' },
} as const;

type SourceWeek = {
  weekStart: string;
  sourceTotal: number;
  entries: MarketRemoteAdoption[];
};

function groupBySource(entries: MarketRemoteAdoption[]) {
  const grouped = new Map<string, Map<string, SourceWeek>>();

  for (const entry of entries) {
    const weeks = grouped.get(entry.source) ?? new Map<string, SourceWeek>();
    const week = weeks.get(entry.weekStart) ?? {
      weekStart: entry.weekStart,
      sourceTotal: entry.sourceTotal,
      entries: [],
    };
    week.entries.push(entry);
    week.sourceTotal = Math.max(week.sourceTotal, entry.sourceTotal);
    weeks.set(entry.weekStart, week);
    grouped.set(entry.source, weeks);
  }

  return Array.from(grouped, ([source, weeks]) => ({
    source,
    weeks: Array.from(weeks.values()).sort((left, right) =>
      left.weekStart.localeCompare(right.weekStart),
    ),
  })).sort((left, right) => left.source.localeCompare(right.source));
}

function percentageFor(week: SourceWeek, mode: MarketRemoteAdoption['workMode']) {
  return week.entries.find((entry) => entry.workMode === mode)?.percentage ?? 0;
}

function formatWeek(date: string) {
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', timeZone: 'UTC' }).format(
    new Date(`${date}T00:00:00Z`),
  );
}

function formatSource(source: string) {
  const knownSources: Record<string, string> = {
    djinni: 'Djinni',
    dou_ua: 'DOU',
    work_ua: 'Work.ua',
    robota_ua: 'Robota.ua',
  };

  return knownSources[source] ?? titleCase(source.replaceAll('_', ' '));
}

function SourceCard({ source, weeks }: { source: string; weeks: SourceWeek[] }) {
  const latest = weeks.at(-1);
  const recentWeeks = weeks.slice(-4);

  if (!latest) {
    return null;
  }

  return (
    <article className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h3 className="m-0 text-sm font-semibold text-card-foreground">{formatSource(source)}</h3>
          <p className="m-0 mt-1 text-xs text-muted-foreground">
            Week of {formatWeek(latest.weekStart)} · {latest.sourceTotal} new active jobs
          </p>
        </div>
        <span className="rounded-full border border-border bg-background px-2.5 py-1 text-xs font-medium text-muted-foreground">
          {percentageFor(latest, 'remote').toFixed(1)}% remote
        </span>
      </div>

      <div
        className="mt-4 flex h-3 overflow-hidden rounded-full bg-background"
        aria-label={`${formatSource(source)} work-mode distribution`}
      >
        {latest.entries.map((entry) => (
          <div
            key={entry.workMode}
            className={workModeMeta[entry.workMode].className}
            style={{ width: `${entry.percentage}%` }}
            title={`${workModeMeta[entry.workMode].label}: ${entry.percentage.toFixed(1)}%`}
          />
        ))}
      </div>

      <div className="mt-3 grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4">
        {Object.entries(workModeMeta).map(([mode, meta]) => (
          <div key={mode} className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className={`h-2 w-2 shrink-0 rounded-full ${meta.className}`} />
            <span>{meta.label}</span>
            <span className="ml-auto font-semibold text-card-foreground">
              {percentageFor(latest, mode as MarketRemoteAdoption['workMode']).toFixed(1)}%
            </span>
          </div>
        ))}
      </div>

      <div className="mt-4 border-t border-border/70 pt-3">
        <p className="m-0 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Remote share by week
        </p>
        <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4">
          {recentWeeks.map((week) => (
            <div
              key={week.weekStart}
              className="rounded-xl border border-border/70 bg-background p-2.5"
            >
              <p className="m-0 text-[11px] text-muted-foreground">{formatWeek(week.weekStart)}</p>
              <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                {percentageFor(week, 'remote').toFixed(1)}%
              </p>
            </div>
          ))}
        </div>
      </div>
    </article>
  );
}

export function MarketRemoteAdoptionSection({ state }: { state: MarketPageState }) {
  const sources = groupBySource(state.remoteAdoptionQuery.data ?? []);

  return (
    <MarketSection
      title="Remote Work Adoption"
      description="Weekly work-mode mix for newly seen active jobs, split by source. Unknown values stay visible instead of being counted as office jobs."
      icon={Laptop2}
    >
      {state.remoteAdoptionQuery.isPending ? (
        <ListSkeleton rows={3} />
      ) : state.remoteAdoptionQuery.isError ? (
        <EmptyState
          message="Unable to load remote-work trends."
          description="The remote-adoption endpoint did not return a usable response."
        />
      ) : sources.length > 0 ? (
        <div className="grid gap-4 xl:grid-cols-2">
          {sources.map((source) => (
            <SourceCard key={source.source} source={source.source} weeks={source.weeks} />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No remote-work trend yet."
          description="This view needs active jobs with source variants seen during the last eight weeks."
        />
      )}
    </MarketSection>
  );
}
