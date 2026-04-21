import { Link } from 'react-router-dom';
import {
  AlertCircle,
  BarChart3,
  BookmarkCheck,
  CheckCircle2,
  ExternalLink,
  EyeOff,
  Sparkles,
} from 'lucide-react';
import type { JobDetailsPageState } from '../../features/job-details/useJobDetailsPage';

import { EmptyState } from '../../components/ui/EmptyState';
import { FitScoreBox } from '../../components/ui/FitScoreBox';
import { cn } from '../../lib/cn';
import { FeedbackButton, Section } from './components';

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
  } = state;

  if (!job) {
    return null;
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
                      className="flex items-start gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3"
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
    </div>
  );
}
