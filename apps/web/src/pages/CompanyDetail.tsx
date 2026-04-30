import { Link, useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { BriefcaseBusiness, Building2, ShieldCheck, ShieldOff, TrendingUp } from 'lucide-react';

import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
} from '../api/feedback';
import { getMarketCompanyDetail } from '../api/market';
import { EmptyState } from '../components/ui/EmptyState';
import { JobCard, JobCardSkeleton } from '../components/ui/JobCard';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatCard } from '../components/ui/StatCard';
import { cn } from '../lib/cn';
import { invalidateFeedbackViewQueries } from '../lib/queryInvalidation';
import { readProfileId } from '../lib/profileSession';
import { queryKeys } from '../queryKeys';

function formatMoney(value?: number) {
  if (value === undefined) {
    return 'Not enough data';
  }

  return `$${Math.round(value).toLocaleString()}`;
}

function VelocityChart({
  points,
}: {
  points: Array<{
    date: string;
    jobCount: number;
  }>;
}) {
  const max = Math.max(1, ...points.map((point) => point.jobCount));

  return (
    <section className="rounded-lg border border-border bg-card p-5">
      <div className="mb-5 flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <TrendingUp className="h-4 w-4" />
        </div>
        <div>
          <h2 className="m-0 text-base font-semibold text-card-foreground">Hiring Velocity</h2>
          <p className="m-0 mt-1 text-sm text-muted-foreground">New active listings by day</p>
        </div>
      </div>

      <div className="grid h-44 grid-cols-7 items-end gap-2">
        {points.map((point) => {
          const height = Math.max(8, Math.round((point.jobCount / max) * 100));
          const label = new Date(`${point.date}T00:00:00`).toLocaleDateString(undefined, {
            month: 'short',
            day: 'numeric',
          });

          return (
            <div key={point.date} className="flex h-full min-w-0 flex-col justify-end gap-2">
              <div className="flex flex-1 items-end">
                <div
                  className="w-full rounded-t-md bg-primary/70"
                  style={{ height: `${height}%` }}
                  title={`${point.jobCount} jobs on ${label}`}
                />
              </div>
              <div className="text-center">
                <p className="m-0 text-xs font-semibold text-card-foreground">{point.jobCount}</p>
                <p className="m-0 truncate text-[11px] text-muted-foreground">{label}</p>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}

export default function CompanyDetail() {
  const { slug = '' } = useParams<{ slug: string }>();
  const profileId = readProfileId();
  const queryClient = useQueryClient();

  const companyQuery = useQuery({
    queryKey: queryKeys.market.companyDetail(slug, profileId),
    queryFn: () => getMarketCompanyDetail(slug),
    enabled: Boolean(slug),
    staleTime: 5 * 60_000,
  });

  const company = companyQuery.data;
  const status = company?.companyStatus;

  const companyFeedbackMutation = useMutation({
    mutationFn: async (nextStatus: 'whitelist' | 'blacklist') => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      if (!company) {
        throw new Error('Company detail is not loaded yet');
      }

      if (nextStatus === 'whitelist') {
        if (status === 'whitelist') {
          await removeCompanyWhitelist(profileId, company.companyName);
        } else {
          await addCompanyWhitelist(profileId, company.companyName);
        }
        return;
      }

      if (status === 'blacklist') {
        await removeCompanyBlacklist(profileId, company.companyName);
      } else {
        await addCompanyBlacklist(profileId, company.companyName);
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: queryKeys.market.companyDetail(slug, profileId),
      });
      void invalidateFeedbackViewQueries(queryClient, profileId);
      toast.success('Company preference updated');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Unable to update company preference');
    },
  });

  if (companyQuery.isPending) {
    return (
      <Page>
        <PageHeader
          title="Company"
          description="Loading company hiring profile."
          breadcrumb={[{ label: 'Market', href: '/market' }, { label: 'Company' }]}
        />
        <div className="grid gap-4 md:grid-cols-3">
          <StatCard title="Total jobs" value="..." icon={BriefcaseBusiness} />
          <StatCard title="Active jobs" value="..." icon={TrendingUp} />
          <StatCard title="Avg salary" value="..." icon={Building2} />
        </div>
        <div className="space-y-3">
          <JobCardSkeleton />
          <JobCardSkeleton />
        </div>
      </Page>
    );
  }

  if (!company) {
    return (
      <Page>
        <EmptyState
          message="Company not found"
          description={
            companyQuery.error instanceof Error
              ? companyQuery.error.message
              : 'No active market company matched this slug.'
          }
        />
      </Page>
    );
  }

  return (
    <Page>
      <PageHeader
        title={company.companyName}
        description="Company-level hiring footprint from the canonical jobs feed."
        breadcrumb={[{ label: 'Market', href: '/market' }, { label: company.companyName }]}
        actions={
          <>
            <button
              type="button"
              className={cn(
                'inline-flex h-9 items-center gap-2 rounded-[var(--radius-lg)] border px-3.5 text-xs font-semibold transition-colors',
                status === 'whitelist'
                  ? 'border-primary/40 bg-primary/12 text-primary'
                  : 'border-border text-foreground hover:bg-white-a05',
              )}
              disabled={companyFeedbackMutation.isPending}
              onClick={() => companyFeedbackMutation.mutate('whitelist')}
            >
              <ShieldCheck className="h-4 w-4" />
              {status === 'whitelist' ? 'Whitelisted' : 'Whitelist'}
            </button>
            <button
              type="button"
              className={cn(
                'inline-flex h-9 items-center gap-2 rounded-[var(--radius-lg)] border px-3.5 text-xs font-semibold transition-colors',
                status === 'blacklist'
                  ? 'border-destructive/40 bg-destructive/10 text-destructive'
                  : 'border-border text-foreground hover:bg-white-a05',
              )}
              disabled={companyFeedbackMutation.isPending}
              onClick={() => companyFeedbackMutation.mutate('blacklist')}
            >
              <ShieldOff className="h-4 w-4" />
              {status === 'blacklist' ? 'Blacklisted' : 'Blacklist'}
            </button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-4">
        <StatCard title="Total jobs" value={company.totalJobs} icon={BriefcaseBusiness} />
        <StatCard title="Active jobs" value={company.activeJobs} icon={TrendingUp} />
        <StatCard title="Avg salary" value={formatMoney(company.avgSalary)} icon={Building2} />
        <StatCard
          title="Preference"
          value={status ? status[0].toUpperCase() + status.slice(1) : 'None'}
          icon={status === 'blacklist' ? ShieldOff : ShieldCheck}
        />
      </div>

      <VelocityChart points={company.velocity} />

      <section className="space-y-3">
        <div className="flex items-end justify-between gap-3">
          <div>
            <h2 className="m-0 text-base font-semibold text-foreground">Active Listings</h2>
            <p className="m-0 mt-1 text-sm text-muted-foreground">
              {company.jobs.length} open roles currently attributed to this company.
            </p>
          </div>
          <Link
            to="/market"
            className="text-sm font-medium text-primary no-underline hover:underline"
          >
            Back to market
          </Link>
        </div>

        {company.jobs.length > 0 ? (
          <div className="grid gap-3 xl:grid-cols-2">
            {company.jobs.map((job) => (
              <JobCard key={job.id} job={job} compact />
            ))}
          </div>
        ) : (
          <EmptyState
            message="No active listings"
            description="This company exists in the historical feed, but no active listings are currently available."
          />
        )}
      </section>
    </Page>
  );
}
