import { useState } from 'react';
import { ChevronDown, TrendingUp } from 'lucide-react';
import { Link } from 'react-router-dom';

import type { MarketCompany } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';
import { MarketTrendBadge } from './MarketTrendBadge';

function CompanyRow({ company }: { company: MarketCompany }) {
  const [expanded, setExpanded] = useState(false);
  const reviewHref =
    company.latestJobIds.length > 0
      ? `/?job_ids=${encodeURIComponent(company.latestJobIds.join(','))}&company=${encodeURIComponent(company.companyName)}`
      : `/?company=${encodeURIComponent(company.companyName)}`;

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-4">
      <button
        type="button"
        className="grid w-full gap-3 text-left lg:grid-cols-[minmax(0,1.3fr)_120px_180px_32px] lg:items-center"
        onClick={() => setExpanded((current) => !current)}
      >
        <div className="min-w-0">
          <p className="m-0 truncate text-sm font-semibold text-card-foreground">
            {company.companyName}
          </p>
          <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
            {formatCount(company.thisWeek)} new this week
            {' • '}
            {formatCount(company.prevWeek)} previous week
          </p>
        </div>
        <div className="flex items-center justify-between gap-3 lg:block lg:text-right">
          <span className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            Active jobs
          </span>
          <p className="m-0 mt-1 text-lg font-semibold text-card-foreground">
            {formatCount(company.activeJobs)}
          </p>
        </div>
        <div className="flex items-center justify-between gap-3 lg:justify-end">
          <span className="text-xs text-muted-foreground">Velocity</span>
          <MarketTrendBadge trend={company.velocity} />
        </div>
        <ChevronDown
          className={`h-4 w-4 text-muted-foreground transition-transform ${expanded ? 'rotate-180' : ''}`}
        />
      </button>

      {expanded ? (
        <div className="mt-4 grid gap-4 border-t border-border/70 pt-4 md:grid-cols-3">
          <CompanyDetail label="Sources" values={company.sources} />
          <CompanyDetail label="Top roles" values={company.topRoleGroups} />
          <CompanyDetail label="Latest jobs" values={company.latestJobIds} />
          <div className="md:col-span-3">
            <Link
              to={reviewHref}
              className="inline-flex h-9 items-center justify-center rounded-[var(--radius-lg)] border border-border px-3.5 text-xs font-semibold text-foreground no-underline hover:bg-white-a05"
            >
              Review company jobs
            </Link>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function CompanyDetail({ label, values }: { label: string; values: string[] }) {
  return (
    <div>
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <div className="mt-2 flex flex-wrap gap-2">
        {values.length > 0 ? (
          values.map((value) => (
            <span key={value} className="rounded-full border border-border bg-background/60 px-2.5 py-1 text-xs text-card-foreground">
              {value}
            </span>
          ))
        ) : (
          <span className="text-xs text-muted-foreground">No grouped data</span>
        )}
      </div>
    </div>
  );
}

export function MarketCompaniesSection({ state }: { state: MarketPageState }) {
  return (
    <MarketSection
      title="Top Hiring Companies"
      description="Companies with the largest active footprint in the current live feed, plus week-over-week hiring velocity."
      icon={TrendingUp}
    >
      {state.companiesQuery.isPending ? (
        <ListSkeleton rows={6} />
      ) : state.companiesQuery.isError ? (
        <EmptyState
          message="Unable to load company activity."
          description="The market companies endpoint did not return a usable response."
        />
      ) : state.companiesQuery.data && state.companiesQuery.data.length > 0 ? (
        <div className="space-y-3">
          {state.companiesQuery.data.map((company) => (
            <CompanyRow key={company.companyName} company={company} />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No active hiring companies yet."
          description="This section fills once the feed has active postings with company attribution."
        />
      )}
    </MarketSection>
  );
}
