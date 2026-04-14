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
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        padding: '14px 18px',
        borderRadius: 16,
        border: '1px solid var(--color-border-soft)',
        background: 'rgba(18, 25, 39, 0.85)',
        minWidth: 140,
      }}
    >
      <span style={{ color, opacity: 0.85 }}>{icon}</span>
      <div>
        <div style={{ fontSize: 24, fontWeight: 700, color: 'var(--color-text-strong)', lineHeight: 1 }}>
          {count}
        </div>
        <div style={{ fontSize: 11, textTransform: 'uppercase', letterSpacing: '0.08em', color: 'var(--color-text-secondary)', marginTop: 3 }}>
          {label}
        </div>
      </div>
    </div>
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
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: 16,
        padding: '12px 16px',
        borderRadius: 12,
        background: 'var(--color-bg-card-soft)',
        border: '1px solid var(--color-border-subtle)',
      }}
    >
      <div style={{ minWidth: 0 }}>
        <div
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: 'var(--color-text-primary)',
            whiteSpace: 'nowrap',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
          }}
        >
          {job.presentation?.title ?? job.title}
        </div>
        <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginTop: 2 }}>
          {job.presentation?.company ?? job.company}
        </div>
      </div>
      <button
        onClick={() => onAction(job.id)}
        disabled={isPending}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          padding: '6px 12px',
          borderRadius: 8,
          border: '1px solid var(--color-border-input)',
          background: 'transparent',
          color: 'var(--color-text-secondary)',
          fontSize: 12,
          cursor: isPending ? 'not-allowed' : 'pointer',
          opacity: isPending ? 0.5 : 1,
          flexShrink: 0,
          whiteSpace: 'nowrap',
        }}
      >
        <Undo2 size={12} />
        {actionLabel}
      </button>
    </div>
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
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: 16,
        padding: '12px 16px',
        borderRadius: 12,
        background: 'var(--color-bg-card-soft)',
        border: '1px solid var(--color-border-subtle)',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <Building2 size={14} style={{ color: 'var(--color-text-secondary)', flexShrink: 0 }} />
        <span style={{ fontSize: 14, color: 'var(--color-text-primary)' }}>
          {record.companyName}
        </span>
      </div>
      <button
        onClick={() => onAction(record.companyName)}
        disabled={isPending}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          padding: '6px 12px',
          borderRadius: 8,
          border: '1px solid var(--color-border-input)',
          background: 'transparent',
          color: 'var(--color-text-secondary)',
          fontSize: 12,
          cursor: isPending ? 'not-allowed' : 'pointer',
          opacity: isPending ? 0.5 : 1,
          flexShrink: 0,
          whiteSpace: 'nowrap',
        }}
      >
        <Undo2 size={12} />
        Remove
      </button>
    </div>
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
    <section
      style={{
        background: 'var(--color-bg-elevated)',
        border: '1px solid var(--color-border-soft)',
        borderRadius: 20,
        padding: '20px 24px',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 16 }}>
        <span style={{ color: 'var(--color-text-secondary)' }}>{icon}</span>
        <h2 style={{ margin: 0, fontSize: 15, fontWeight: 600, color: 'var(--color-text-primary)' }}>
          {title}
        </h2>
        <span
          style={{
            marginLeft: 'auto',
            fontSize: 12,
            color: 'var(--color-text-secondary)',
            background: 'var(--color-bg-hover)',
            borderRadius: 8,
            padding: '2px 8px',
          }}
        >
          {count}
        </span>
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
          <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, color: 'var(--color-text-strong)' }}>
            Feedback Center
          </h1>
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
