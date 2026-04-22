import { Link } from 'react-router-dom';
import {
  Bookmark,
  BookmarkCheck,
  Building2,
  CalendarClock,
  Check,
  Copy,
  MapPin,
  Sparkles,
  Target,
} from 'lucide-react';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { PageHeader } from '../../components/ui/SectionHeader';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { getJobLifecycleLabels } from '../../lib/jobPresentation';
import { HeroMetric } from './components';

export function JobDetailsHeader({
  state,
  salary,
  sourceLabel,
  descriptionQuality,
  topBadges,
  lifecycleStatus,
}: {
  state: JobDetailsPageState;
  salary: string | null;
  sourceLabel: string;
  descriptionQuality: string | undefined;
  topBadges: string[];
  lifecycleStatus: string;
}) {
  const { job, existing, isSaved, copied, handleCopy, saveMutation, unsaveMutation, fit, isBadFit, companyStatus } =
    state;

  if (!job) {
    return null;
  }

  const lifecycleLabels = getJobLifecycleLabels(job);

  return (
    <>
      <PageHeader
        title={job.title}
        description={job.company}
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Jobs' }, { label: job.company }]}
        actions={
          <>
            {existing ? (
              <Link to={`/applications/${existing.id}`} className="no-underline">
                <Button variant="outline" size="sm" className="bg-primary/10 border-primary/30 text-primary">
                  <BookmarkCheck className="h-4 w-4" />
                  {existing.status}
                </Button>
              </Link>
            ) : isSaved ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => unsaveMutation.mutate()}
                disabled={unsaveMutation.isPending}
                className="bg-primary/10 border-primary/30 text-primary"
              >
                <BookmarkCheck className="h-4 w-4 text-primary" />
                {unsaveMutation.isPending ? 'Знімаємо…' : 'Saved'}
              </Button>
            ) : (
              <Button onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
                <Bookmark className="h-4 w-4" />
                {saveMutation.isPending ? 'Зберігаємо…' : 'Save'}
              </Button>
            )}
            <Button variant="outline" size="sm" onClick={() => void handleCopy()}>
              {copied ? <Check className="h-4 w-4 text-fit-excellent" /> : <Copy className="h-4 w-4" />}
              {copied ? 'Copied' : 'Share'}
            </Button>
          </>
        }
      />

      <div className="overflow-hidden rounded-[var(--radius-card)] border border-border bg-card">
        <div className="relative">
          <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/8 via-accent/6 to-transparent" />
          <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-start lg:justify-between">
            <div className="flex min-w-0 gap-4">
              <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary">
                <Building2 className="h-8 w-8" />
              </div>
              <div className="min-w-0 space-y-4">
                <div className="flex flex-wrap gap-2">
                  {topBadges.map((badge) => (
                    <Badge key={badge} variant="muted" className="px-2.5 py-1 text-xs">
                      {badge}
                    </Badge>
                  ))}
                  <StatusBadge status={lifecycleStatus} />
                  {isBadFit ? <StatusBadge status="bad fit" /> : null}
                  {companyStatus === 'blacklist' ? (
                    <StatusBadge status="blacklist" label="company blacklisted" />
                  ) : null}
                  {companyStatus === 'whitelist' ? (
                    <StatusBadge status="whitelist" label="company whitelisted" />
                  ) : null}
                  {descriptionQuality === 'weak' ? (
                    <StatusBadge status="bad fit" label="description may be incomplete" />
                  ) : null}
                </div>

                <div>
                  <h2 className="m-0 text-2xl font-bold text-card-foreground">{job.title}</h2>
                  <p className="m-0 mt-2 text-base text-muted-foreground">{job.company}</p>
                </div>

                <div className="flex flex-wrap gap-3 text-sm text-muted-foreground">
                  {salary ? (
                    <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-white-a05 px-3 py-1.5">
                      <MapPin className="h-4 w-4" />
                      {salary}
                    </span>
                  ) : null}
                  {lifecycleLabels.map((label) => (
                    <span
                      key={label}
                      className="inline-flex items-center gap-1.5 rounded-full border border-border bg-white-a05 px-3 py-1.5"
                    >
                      <CalendarClock className="h-4 w-4" />
                      {label}
                    </span>
                  ))}
                  <span className="inline-flex items-center gap-1.5 rounded-full border border-border bg-white-a05 px-3 py-1.5">
                    <MapPin className="h-4 w-4" />
                    {sourceLabel}
                  </span>
                </div>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[420px]">
              <HeroMetric label="Fit score" value={fit ? `${fit.score}%` : 'Pending'} icon={Target} />
              <HeroMetric
                label="Matched terms"
                value={fit?.matchedTerms.length ?? 0}
                icon={Sparkles}
              />
              <HeroMetric
                label="Pipeline"
                value={existing ? existing.status : 'Not saved'}
                icon={BookmarkCheck}
              />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
