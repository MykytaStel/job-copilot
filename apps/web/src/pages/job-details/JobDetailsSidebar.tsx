import { useState } from 'react';
import { Link } from 'react-router-dom';
import {
  AlertCircle,
  BarChart3,
  BookmarkCheck,
  CheckCircle2,
  ChevronDown,
  ExternalLink,
  EyeOff,
  Flag,
  Sparkles,
  Star,
  Wifi,
} from 'lucide-react';
import type { JobFeedbackReason, LegitimacySignal, SalaryFeedbackSignal, WorkModeFeedbackSignal } from '@job-copilot/shared/feedback';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { EmptyState } from '../../components/ui/EmptyState';
import { FitScoreBox } from '../../components/ui/FitScoreBox';
import { cn } from '../../lib/cn';
import { FeedbackButton, Section } from './components';

const NEGATIVE_TAGS: JobFeedbackReason[] = [
  'salary_too_low',
  'not_remote',
  'too_junior',
  'too_senior',
  'bad_tech_stack',
  'already_applied',
  'wrong_city',
  'wrong_industry',
  'bad_company_rep',
  'visa_sponsorship_required',
];

const POSITIVE_TAGS: JobFeedbackReason[] = [
  'interesting_challenge',
  'great_company',
  'good_salary',
  'remote_ok',
  'good_tech_stack',
  'fast_growth_company',
  'nice_title',
];

const TAG_LABELS: Record<JobFeedbackReason, string> = {
  salary_too_low: 'Salary too low',
  not_remote: 'Not remote',
  too_junior: 'Too junior',
  too_senior: 'Too senior',
  bad_tech_stack: 'Bad tech stack',
  suspicious_posting: 'Suspicious',
  already_applied: 'Already applied',
  duplicate_posting: 'Duplicate',
  bad_company_rep: 'Bad reputation',
  wrong_city: 'Wrong city',
  wrong_industry: 'Wrong industry',
  visa_sponsorship_required: 'Visa required',
  interesting_challenge: 'Interesting',
  great_company: 'Great company',
  good_salary: 'Good salary',
  remote_ok: 'Remote-friendly',
  good_tech_stack: 'Good tech stack',
  fast_growth_company: 'Fast growth',
  nice_title: 'Nice title',
};

const SALARY_LABELS: Record<SalaryFeedbackSignal, string> = {
  above_expectation: 'Above expectation',
  at_expectation: 'At expectation',
  below_expectation: 'Below expectation',
  not_shown: 'Salary not shown',
};

const WORK_MODE_LABELS: Record<WorkModeFeedbackSignal, string> = {
  matches_preference: 'Matches preference',
  would_accept: 'Would accept',
  deal_breaker: 'Deal-breaker',
};

const INTEREST_CONFIG = [
  { rating: 2, label: '❤', title: 'Love it' },
  { rating: 1, label: '+', title: 'Interested' },
  { rating: 0, label: '~', title: 'Neutral' },
  { rating: -1, label: '-', title: 'Not really' },
  { rating: -2, label: '✗', title: 'Definitely not' },
] as const;

export function JobDetailsSidebar({ state }: { state: JobDetailsPageState }) {
  const {
    profileId,
    fit,
    job,
    existing,
    isHidden,
    isBadFit,
    companyStatus,
    unhideMutation,
    hideMutation,
    unmarkBadFitMutation,
    badFitMutation,
    companyFeedbackMutation,
    interestRatingMutation,
    salarySignalMutation,
    workModeMutation,
    tagsMutation,
    legitimacyMutation,
  } = state;

  const [pendingTags, setPendingTags] = useState<Set<JobFeedbackReason>>(new Set());
  const [reportOpen, setReportOpen] = useState(false);

  if (!job) {
    return null;
  }

  const currentInterest = job.feedback?.interestRating;
  const currentSalary = job.feedback?.salarySignal;
  const currentWorkMode = job.feedback?.workModeSignal;

  function toggleTag(tag: JobFeedbackReason) {
    setPendingTags((prev) => {
      const next = new Set(prev);
      if (next.has(tag)) {
        next.delete(tag);
      } else {
        next.add(tag);
      }
      return next;
    });
  }

  function submitTags() {
    tagsMutation.mutate([...pendingTags]);
  }

  function reportAs(signal: LegitimacySignal) {
    legitimacyMutation.mutate(signal);
    setReportOpen(false);
  }

  return (
    <div className="space-y-6">
      {profileId ? (
        <Section
          title="Fit Snapshot"
          description="Compact view of the current match score and the strongest signals."
          icon={Sparkles}
        >
          {fit ? (
            <div className="space-y-5">
              <FitScoreBox score={fit.score} size="lg" showLabel className="mx-auto" />
              {fit.evidence.length > 0 ? (
                <div className="space-y-3">
                  {fit.evidence.slice(0, 3).map((entry) => (
                    <div
                      key={entry}
                      className="flex items-start gap-3 rounded-2xl border border-border/70 bg-surface-muted px-4 py-3"
                    >
                      <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-fit-excellent" />
                      <p className="m-0 text-sm leading-6 text-card-foreground">{entry}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <EmptyState message="No evidence returned yet." className="px-4 py-4 text-left" />
              )}
            </div>
          ) : (
            <p className="m-0 text-sm text-muted-foreground">Аналізуємо…</p>
          )}
        </Section>
      ) : null}

      {profileId ? (
        <Section
          title="Interest"
          description="How much do you want this role? Helps the model learn your preferences."
          icon={Star}
        >
          <div className="flex gap-1.5">
            {INTEREST_CONFIG.map(({ rating, label, title }) => (
              <button
                key={rating}
                title={title}
                disabled={interestRatingMutation.isPending}
                onClick={() => interestRatingMutation.mutate(rating)}
                className={cn(
                  'flex h-9 flex-1 items-center justify-center rounded-xl border text-sm font-medium transition-colors',
                  currentInterest === rating
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border bg-surface-muted text-muted-foreground hover:border-primary/50 hover:text-foreground',
                )}
              >
                {label}
              </button>
            ))}
          </div>
          {currentInterest !== undefined && (
            <p className="mt-1 text-xs text-muted-foreground">
              Current: {INTEREST_CONFIG.find((c) => c.rating === currentInterest)?.title ?? currentInterest}
            </p>
          )}
        </Section>
      ) : null}

      <Section
        title="Feedback Actions"
        description="Use explicit controls to change ranking behavior for this role and company."
        icon={BarChart3}
      >
        <div className="space-y-3">
          {isHidden ? (
            <FeedbackButton disabled={unhideMutation.isPending} onClick={() => unhideMutation.mutate()}>
              <EyeOff className="h-4 w-4" />
              {unhideMutation.isPending ? 'Показуємо…' : 'Unhide job'}
            </FeedbackButton>
          ) : (
            <FeedbackButton disabled={hideMutation.isPending} onClick={() => hideMutation.mutate()}>
              <EyeOff className="h-4 w-4" />
              {hideMutation.isPending ? 'Ховаємо…' : 'Hide job'}
            </FeedbackButton>
          )}

          {isBadFit ? (
            <FeedbackButton
              disabled={unmarkBadFitMutation.isPending}
              onClick={() => unmarkBadFitMutation.mutate()}
            >
              <AlertCircle className="h-4 w-4" />
              {unmarkBadFitMutation.isPending ? 'Знімаємо…' : 'Remove bad fit'}
            </FeedbackButton>
          ) : (
            <FeedbackButton
              className="text-destructive hover:text-destructive"
              disabled={badFitMutation.isPending}
              onClick={() => badFitMutation.mutate()}
            >
              <AlertCircle className="h-4 w-4" />
              {badFitMutation.isPending ? 'Позначаємо…' : 'Not a good fit'}
            </FeedbackButton>
          )}

          <FeedbackButton
            className={cn(companyStatus === 'whitelist' && 'bg-primary/10 border-primary/30')}
            disabled={companyFeedbackMutation.isPending}
            onClick={() => companyFeedbackMutation.mutate('whitelist')}
          >
            <CheckCircle2 className="h-4 w-4" />
            {companyStatus === 'whitelist' ? 'Unwhitelist company' : 'Whitelist company'}
          </FeedbackButton>

          <FeedbackButton
            className={cn(
              companyStatus === 'blacklist' &&
                'bg-destructive/10 border-destructive/30 text-destructive',
            )}
            disabled={companyFeedbackMutation.isPending}
            onClick={() => companyFeedbackMutation.mutate('blacklist')}
          >
            <AlertCircle className="h-4 w-4" />
            {companyStatus === 'blacklist' ? 'Unblacklist company' : 'Blacklist company'}
          </FeedbackButton>
        </div>
      </Section>

      {profileId && isBadFit ? (
        <Section
          title="Why not a good fit?"
          description="Optional. Select tags to improve future ranking."
          icon={AlertCircle}
        >
          <div className="space-y-3">
            <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              Negatives
            </p>
            <div className="flex flex-wrap gap-1.5">
              {NEGATIVE_TAGS.map((tag) => (
                <button
                  key={tag}
                  onClick={() => toggleTag(tag)}
                  className={cn(
                    'rounded-full border px-2.5 py-1 text-xs transition-colors',
                    pendingTags.has(tag)
                      ? 'border-destructive/60 bg-destructive/10 text-destructive'
                      : 'border-border bg-surface-muted text-muted-foreground hover:border-destructive/40',
                  )}
                >
                  {TAG_LABELS[tag]}
                </button>
              ))}
            </div>
            <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              Positives
            </p>
            <div className="flex flex-wrap gap-1.5">
              {POSITIVE_TAGS.map((tag) => (
                <button
                  key={tag}
                  onClick={() => toggleTag(tag)}
                  className={cn(
                    'rounded-full border px-2.5 py-1 text-xs transition-colors',
                    pendingTags.has(tag)
                      ? 'border-primary/60 bg-primary/10 text-primary'
                      : 'border-border bg-surface-muted text-muted-foreground hover:border-primary/40',
                  )}
                >
                  {TAG_LABELS[tag]}
                </button>
              ))}
            </div>
            {pendingTags.size > 0 && (
              <FeedbackButton
                disabled={tagsMutation.isPending}
                onClick={submitTags}
              >
                <CheckCircle2 className="h-4 w-4" />
                {tagsMutation.isPending ? 'Зберігаємо…' : `Save ${pendingTags.size} tag${pendingTags.size > 1 ? 's' : ''}`}
              </FeedbackButton>
            )}
          </div>
        </Section>
      ) : null}

      {profileId ? (
        <Section
          title="Salary"
          description="Does the salary match your expectations?"
          icon={BarChart3}
        >
          <div className="space-y-1.5">
            {(['above_expectation', 'at_expectation', 'below_expectation', 'not_shown'] as SalaryFeedbackSignal[]).map((signal) => (
              <FeedbackButton
                key={signal}
                disabled={salarySignalMutation.isPending}
                className={cn(currentSalary === signal && 'border-primary/40 bg-primary/10 text-primary')}
                onClick={() => salarySignalMutation.mutate(signal)}
              >
                {SALARY_LABELS[signal]}
              </FeedbackButton>
            ))}
          </div>
        </Section>
      ) : null}

      {profileId ? (
        <Section
          title="Work Mode"
          description="Does the remote/office setup work for you?"
          icon={Wifi}
        >
          <div className="space-y-1.5">
            {(['matches_preference', 'would_accept', 'deal_breaker'] as WorkModeFeedbackSignal[]).map((signal) => (
              <FeedbackButton
                key={signal}
                disabled={workModeMutation.isPending}
                className={cn(
                  currentWorkMode === signal && 'border-primary/40 bg-primary/10 text-primary',
                  signal === 'deal_breaker' && currentWorkMode === signal && 'border-destructive/40 bg-destructive/10 text-destructive',
                )}
                onClick={() => workModeMutation.mutate(signal)}
              >
                {WORK_MODE_LABELS[signal]}
              </FeedbackButton>
            ))}
          </div>
        </Section>
      ) : null}

      <Section
        title="Source Actions"
        description="Open the original posting or jump into the application record when available."
        icon={ExternalLink}
      >
        <div className="space-y-2">
          {job.primaryVariant?.sourceUrl ? (
            <a
              href={job.primaryVariant.sourceUrl}
              target="_blank"
              rel="noreferrer"
              className="block no-underline"
            >
              <FeedbackButton className="justify-center">
                <ExternalLink className="h-4 w-4" />
                Apply on source
              </FeedbackButton>
            </a>
          ) : null}
          {existing ? (
            <Link to={`/applications/${existing.id}`} className="block no-underline">
              <FeedbackButton className="justify-center">
                <BookmarkCheck className="h-4 w-4" />
                Open application record
              </FeedbackButton>
            </Link>
          ) : null}
        </div>
      </Section>

      {profileId ? (
        <Section
          title="Report"
          description="Flag this posting if it looks suspicious, spam, or a duplicate."
          icon={Flag}
        >
          <div className="relative">
            <FeedbackButton
              className="text-muted-foreground"
              onClick={() => setReportOpen((v) => !v)}
              disabled={legitimacyMutation.isPending}
            >
              <Flag className="h-4 w-4" />
              Report posting
              <ChevronDown className={cn('ml-auto h-3.5 w-3.5 transition-transform', reportOpen && 'rotate-180')} />
            </FeedbackButton>
            {reportOpen && (
              <div className="absolute left-0 right-0 z-10 mt-1 rounded-2xl border border-border bg-card p-1 shadow-md">
                {(['suspicious', 'spam', 'duplicate'] as LegitimacySignal[]).map((signal) => (
                  <button
                    key={signal}
                    onClick={() => reportAs(signal)}
                    className="w-full rounded-xl px-3 py-2 text-left text-sm capitalize text-muted-foreground hover:bg-surface-muted hover:text-foreground"
                  >
                    {signal}
                  </button>
                ))}
              </div>
            )}
          </div>
        </Section>
      ) : null}
    </div>
  );
}
