import { TrendingUp } from 'lucide-react';

import type { MarketCompany } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';
import { MarketTrendBadge } from './MarketTrendBadge';

function CompanyRow({ company }: { company: MarketCompany }) {
  return (
    <div className="grid gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4 lg:grid-cols-[minmax(0,1.3fr)_120px_180px] lg:items-center">
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
