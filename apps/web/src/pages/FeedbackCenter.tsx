import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Bookmark,
  Building2,
  EyeOff,
  ShieldCheck,
  ShieldOff,
  ThumbsDown,
  Undo2,
} from 'lucide-react';
import toast from 'react-hot-toast';
import type { CompanyFeedbackRecord, JobPosting } from '@job-copilot/shared';

import {
  getFeedback,
  getJobsFeed,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
} from '../api';
import { queryKeys } from '../queryKeys';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

function SummaryCard({
  label,
  count,
  icon,
  color,
}: {
  label: string;
  count: number;
  icon: React.ReactNode;
  color: string;
}) {
  return (
    <Card className="flex items-center gap-3 px-[18px] py-[14px] min-w-[140px]">
      <span style={{ color, opacity: 0.85 }}>{icon}</span>
      <div>
        <div className="text-2xl font-bold text-content-strong leading-none">{count}</div>
        <div className="text-[11px] uppercase tracking-[0.08em] text-content-muted mt-0.5">{label}</div>
      </div>
    </Card>
  );
}

function JobRow({
  job,
  actionLabel,
  onAction,
  isPending,
}: {
  job: JobPosting;
  actionLabel: string;
  onAction: (jobId: string) => void;
  isPending: boolean;
}) {
  return (
    <Card className="flex items-center justify-between gap-4 px-4 py-3">
      <div className="min-w-0">
        <div className="text-sm font-semibold text-content truncate">
          {job.presentation?.title ?? job.title}
        </div>
        <div className="text-xs text-content-muted mt-0.5">
          {job.presentation?.company ?? job.company}
        </div>
      </div>
      <Button variant="ghost" size="sm" onClick={() => onAction(job.id)} disabled={isPending}>
        <Undo2 size={12} />
        {actionLabel}
      </Button>
    </Card>
  );
}

function CompanyRow({
  record,
  onAction,
  isPending,
}: {
  record: CompanyFeedbackRecord;
  onAction: (companyName: string) => void;
  isPending: boolean;
}) {
  return (
    <Card className="flex items-center justify-between gap-4 px-4 py-3">
      <div className="flex items-center gap-2">
        <Building2 size={14} className="text-content-muted shrink-0" />
        <span className="text-sm text-content">{record.companyName}</span>
      </div>
      <Button variant="ghost" size="sm" onClick={() => onAction(record.companyName)} disabled={isPending}>
        <Undo2 size={12} />
        Remove
      </Button>
    </Card>
  );
}

function Section({
  title,
  icon,
  children,
  count,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
  count: number;
}) {
  return (
    <section className="card">
      <div className="flex items-center gap-2 mb-4">
        <span className="text-content-muted">{icon}</span>
        <h2 className="m-0 text-[15px] font-semibold text-content">{title}</h2>
        <Badge variant="muted" className="ml-auto text-xs px-2 py-0.5 rounded-lg">{count}</Badge>
      </div>
      {children}
    </section>
  );
}

export default function FeedbackCenter() {
  const profileId = readProfileId();
  const queryClient = useQueryClient();

  const { data: jobsFeed, isLoading: jobsLoading } = useQuery({
    queryKey: queryKeys.jobs.filtered('all', null, profileId),
    queryFn: () => getJobsFeed({ limit: 500 }),
    enabled: !!profileId,
  });

  const { data: feedbackOverview, isLoading: feedbackLoading } = useQuery({
    queryKey: queryKeys.feedback.profile(profileId ?? ''),
    queryFn: () => getFeedback(profileId!),
    enabled: !!profileId,
  });

  const allJobs = jobsFeed?.jobs ?? [];
  const savedJobs = allJobs.filter((j) => j.feedback?.saved);
  const hiddenJobs = allJobs.filter((j) => j.feedback?.hidden);
  const badFitJobs = allJobs.filter((j) => j.feedback?.badFit);
  const whitelistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (c) => c.status === 'whitelist',
  );
  const blacklistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (c) => c.status === 'blacklist',
  );
  const summary = feedbackOverview?.summary;

  function invalidateAfterAction() {
    void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
    if (profileId) {
      void queryClient.invalidateQueries({ queryKey: queryKeys.feedback.profile(profileId) });
    }
  }

  const unsaveMutation = useMutation({
    mutationFn: (jobId: string) => unsaveJob(profileId!, jobId),
    onSuccess: () => { invalidateAfterAction(); toast.success('Збережено скасовано'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unhideMutation = useMutation({
    mutationFn: (jobId: string) => unhideJob(profileId!, jobId),
    onSuccess: () => { invalidateAfterAction(); toast.success('Вакансію показано знову'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: (jobId: string) => unmarkJobBadFit(profileId!, jobId),
    onSuccess: () => { invalidateAfterAction(); toast.success('Позначку bad fit знято'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const removeWhitelistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyWhitelist(profileId!, companyName),
    onSuccess: () => { invalidateAfterAction(); toast.success('Видалено зі списку дозволених'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const removeBlacklistMutation = useMutation({
    mutationFn: (companyName: string) => removeCompanyBlacklist(profileId!, companyName),
    onSuccess: () => { invalidateAfterAction(); toast.success('Видалено зі списку заблокованих'); },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  if (!profileId) {
    return (
      <div className="jobDetails">
        <p className="emptyState">Create a profile to view feedback.</p>
      </div>
    );
  }

  const isLoading = jobsLoading || feedbackLoading;

  return (
    <div className="jobDetails">
      <div className="pageHeader">
        <div>
          <p className="eyebrow">Personalization</p>
          <h1 className="m-0 text-[22px] font-bold text-content-strong">Feedback Center</h1>
        </div>
      </div>

      {isLoading ? (
        <p className="emptyState">Loading feedback…</p>
      ) : (
        <>
          {summary && (
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
              <SummaryCard label="Saved" count={summary.savedJobsCount} icon={<Bookmark size={18} />} color="#95a7ff" />
              <SummaryCard label="Hidden" count={summary.hiddenJobsCount} icon={<EyeOff size={18} />} color="#9aa8bc" />
              <SummaryCard label="Bad Fit" count={summary.badFitJobsCount} icon={<ThumbsDown size={18} />} color="#ffb4b4" />
              <SummaryCard label="Whitelisted" count={summary.whitelistedCompaniesCount} icon={<ShieldCheck size={18} />} color="#c9fff8" />
              <SummaryCard label="Blacklisted" count={summary.blacklistedCompaniesCount} icon={<ShieldOff size={18} />} color="#ffb4b4" />
            </div>
          )}

          <Section title="Saved Jobs" icon={<Bookmark size={16} />} count={savedJobs.length}>
            {savedJobs.length === 0 ? (
              <p className="emptyState">No saved jobs.</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {savedJobs.map((job) => (
                  <JobRow
                    key={job.id}
                    job={job}
                    actionLabel="Unsave"
                    onAction={(id) => unsaveMutation.mutate(id)}
                    isPending={unsaveMutation.isPending}
                  />
                ))}
              </div>
            )}
          </Section>

          <Section title="Hidden Jobs" icon={<EyeOff size={16} />} count={hiddenJobs.length}>
            {hiddenJobs.length === 0 ? (
              <p className="emptyState">No hidden jobs.</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {hiddenJobs.map((job) => (
                  <JobRow
                    key={job.id}
                    job={job}
                    actionLabel="Unhide"
                    onAction={(id) => unhideMutation.mutate(id)}
                    isPending={unhideMutation.isPending}
                  />
                ))}
              </div>
            )}
          </Section>

          <Section title="Bad Fit" icon={<ThumbsDown size={16} />} count={badFitJobs.length}>
            {badFitJobs.length === 0 ? (
              <p className="emptyState">No jobs marked as bad fit.</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {badFitJobs.map((job) => (
                  <JobRow
                    key={job.id}
                    job={job}
                    actionLabel="Unmark"
                    onAction={(id) => unmarkBadFitMutation.mutate(id)}
                    isPending={unmarkBadFitMutation.isPending}
                  />
                ))}
              </div>
            )}
          </Section>

          <Section title="Whitelisted Companies" icon={<ShieldCheck size={16} />} count={whitelistedCompanies.length}>
            {whitelistedCompanies.length === 0 ? (
              <p className="emptyState">No whitelisted companies.</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {whitelistedCompanies.map((c) => (
                  <CompanyRow
                    key={c.normalizedCompanyName}
                    record={c}
                    onAction={(name) => removeWhitelistMutation.mutate(name)}
                    isPending={removeWhitelistMutation.isPending}
                  />
                ))}
              </div>
            )}
          </Section>

          <Section title="Blacklisted Companies" icon={<ShieldOff size={16} />} count={blacklistedCompanies.length}>
            {blacklistedCompanies.length === 0 ? (
              <p className="emptyState">No blacklisted companies.</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {blacklistedCompanies.map((c) => (
                  <CompanyRow
                    key={c.normalizedCompanyName}
                    record={c}
                    onAction={(name) => removeBlacklistMutation.mutate(name)}
                    isPending={removeBlacklistMutation.isPending}
                  />
                ))}
              </div>
            )}
          </Section>
        </>
      )}
    </div>
  );
}
