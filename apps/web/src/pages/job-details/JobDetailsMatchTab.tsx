import { AlertCircle, CheckCircle2, FileText, Globe, MapPin, MinusCircle, Sparkles, Wifi, XCircle } from 'lucide-react';
import type { ReactNode } from 'react';

import type { JobPosting } from '@job-copilot/shared/jobs';
import type { ResumeVersion } from '@job-copilot/shared/profiles';
import type { ResumeMatch } from '../../api/enrichment';
import type { FitAnalysis, MissingSignalDetail } from '../../api/jobs';
import { Badge } from '../../components/ui/Badge';
import { FitScoreCircular } from '../../components/ui/FitScoreBox';
import { Section } from './components';

export function JobDetailsMatchTab({
  fit,
  job,
  activeResume,
  resumeMatch,
  resumeMatchLoading,
  resumeMatchError,
}: {
  fit: FitAnalysis | undefined;
  job?: JobPosting;
  activeResume?: ResumeVersion;
  resumeMatch?: ResumeMatch;
  resumeMatchLoading: boolean;
  resumeMatchError: Error | null;
}) {
  const scoreSignals = job?.presentation?.scoreSignals ?? [];

  if (!fit) {
    return (
      <Section
        title="Why this score?"
        description="Matched signals, score breakdown, and missing gaps from the current fit analysis."
        icon={Sparkles}
      >
        <div className="space-y-5">
          <p className="m-0 text-sm text-muted-foreground">Fit analysis is not ready yet.</p>
          <ResumeMatchPanel
            activeResume={activeResume}
            resumeMatch={resumeMatch}
            resumeMatchLoading={resumeMatchLoading}
            resumeMatchError={resumeMatchError}
          />
        </div>
      </Section>
    );
  }

  const matchedRoleCount = fit.matchedRoles.length;
  const matchedSkillCount = fit.matchedSkills.length;
  const matchedKeywordCount = fit.matchedKeywords.length;

  const matchSummaryParts = [
    matchedRoleCount > 0 && `${matchedRoleCount} role${matchedRoleCount !== 1 ? 's' : ''} matched`,
    matchedSkillCount > 0 && `${matchedSkillCount} skill${matchedSkillCount !== 1 ? 's' : ''} matched`,
    matchedKeywordCount > 0 && `${matchedKeywordCount} keyword${matchedKeywordCount !== 1 ? 's' : ''} matched`,
  ].filter(Boolean);

  return (
    <Section
      title="Why this score?"
      description="Matched signals explain why the job is relevant; missing signals show profile evidence not found strongly enough in this posting."
      icon={Sparkles}
    >
      <div className="space-y-5">
        <div className="flex items-start gap-4 rounded-2xl border border-primary/20 bg-primary/5 p-4">
          <FitScoreCircular score={fit.score} size="md" showLabel />
          <div className="space-y-2">
            <p className="m-0 text-sm leading-7 text-muted-foreground">
              {fit.positiveReasons[0] ??
                fit.negativeReasons[0] ??
                'Fit analysis is based on canonical Rust matching over the stored profile and job signals.'}
            </p>
            {matchSummaryParts.length > 0 ? (
              <p className="m-0 text-xs leading-5 text-muted-foreground">
                {matchSummaryParts.join(' · ')}
              </p>
            ) : null}
          </div>
        </div>

        <div className="grid gap-4 xl:grid-cols-3">
          <ScorePart label="Matching" value={fit.scoreBreakdown.matchingScore} />
          <ScorePart label="Salary" value={fit.scoreBreakdown.salaryScore} />
          <ScorePart label="Freshness" value={fit.scoreBreakdown.freshnessScore} />
        </div>

        <ContextSignals fit={fit} />

        <ResumeMatchPanel
          activeResume={activeResume}
          resumeMatch={resumeMatch}
          resumeMatchLoading={resumeMatchLoading}
          resumeMatchError={resumeMatchError}
        />

        {scoreSignals.length > 0 || fit.scoreBreakdown.penalties.length > 0 ? (
          <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
            <p className="mb-3 flex items-center gap-2 text-sm font-medium text-card-foreground">
              <MinusCircle className="h-4 w-4" />
              Score signals
            </p>
            <div className="flex flex-wrap gap-2">
              {scoreSignals.map((signal) => (
                <Badge
                  key={`${signal.label}-${signal.delta}`}
                  variant={signal.delta >= 0 ? 'success' : 'danger'}
                >
                  {signal.label} ({signal.delta > 0 ? '+' : ''}
                  {signal.delta})
                </Badge>
              ))}
              {fit.scoreBreakdown.penalties.map((penalty) => (
                <Badge key={`${penalty.kind}-${penalty.scoreDelta}`} variant="danger">
                  {penalty.kind} ({penalty.scoreDelta})
                </Badge>
              ))}
            </div>
          </div>
        ) : null}

        <div className="grid gap-4 xl:grid-cols-2">
          <ReasonPanel
            title="Strengths"
            icon={<CheckCircle2 className="h-4 w-4" />}
            tone="success"
            items={fit.positiveReasons}
            empty="No positive signals yet."
          />
          <ReasonPanel
            title="Penalties and risks"
            icon={<AlertCircle className="h-4 w-4" />}
            tone="warning"
            items={fit.negativeReasons}
            empty="No penalties returned."
          />
        </div>

        <TermPanel
          title="Missing signals"
          icon={<AlertCircle className="h-4 w-4" />}
          details={fit.missingSignalDetails}
        />

        <MatchedSignals fit={fit} />
      </div>
    </Section>
  );
}

function ContextSignals({ fit }: { fit: FitAnalysis }) {
  const rows: { label: string; icon: typeof Globe; value: boolean | undefined }[] = [
    { label: 'Source', icon: Globe, value: fit.sourceMatch },
    { label: 'Work mode', icon: Wifi, value: fit.workModeMatch },
    { label: 'Region', icon: MapPin, value: fit.regionMatch },
  ];

  const visibleRows = rows.filter((r) => r.value !== undefined);
  if (visibleRows.length === 0) return null;

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="mb-3 text-sm font-medium text-card-foreground">Context signals</p>
      <div className="space-y-2">
        {visibleRows.map(({ label, icon: Icon, value }) => (
          <div key={label} className="flex items-center gap-3">
            <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">{label}</span>
            {value ? (
              <span className="ml-auto flex items-center gap-1 text-xs font-medium text-content-success">
                <CheckCircle2 className="h-3.5 w-3.5" />
                Matched
              </span>
            ) : (
              <span className="ml-auto flex items-center gap-1 text-xs font-medium text-content-warning">
                <XCircle className="h-3.5 w-3.5" />
                Not matched
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function MatchedSignals({ fit }: { fit: FitAnalysis }) {
  const groups = [
    { label: 'Roles', items: fit.matchedRoles },
    { label: 'Skills', items: fit.matchedSkills },
    { label: 'Keywords', items: fit.matchedKeywords },
  ].filter((g) => g.items.length > 0);

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-success">
        <CheckCircle2 className="h-4 w-4" />
        Matched signals
      </p>
      {groups.length > 0 ? (
        <div className="space-y-4">
          {groups.map(({ label, items }) => (
            <div key={label}>
              <p className="mb-2 text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
                {label}
              </p>
              <div className="flex flex-wrap gap-2">
                {items.map((item) => (
                  <Badge key={item} variant="success" className="px-3 py-1 text-xs">
                    {item}
                  </Badge>
                ))}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <p className="m-0 text-sm text-muted-foreground">No matched signals returned.</p>
      )}
    </div>
  );
}

function ResumeMatchPanel({
  activeResume,
  resumeMatch,
  resumeMatchLoading,
  resumeMatchError,
}: {
  activeResume?: ResumeVersion;
  resumeMatch?: ResumeMatch;
  resumeMatchLoading: boolean;
  resumeMatchError: Error | null;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <div className="mb-3 flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="m-0 flex items-center gap-2 text-sm font-medium text-card-foreground">
            <FileText className="h-4 w-4" />
            Resume keyword coverage
          </p>
          <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
            Deterministic comparison of the active CV text against this job description.
          </p>
        </div>
        {resumeMatch ? (
          <Badge variant={resumeMatch.keywordCoveragePercent >= 70 ? 'success' : 'warning'}>
            {resumeMatch.keywordCoveragePercent.toFixed(1)}% covered
          </Badge>
        ) : null}
      </div>

      {!activeResume ? (
        <p className="m-0 text-sm text-muted-foreground">
          Upload or activate a CV to compare the actual resume text with this JD.
        </p>
      ) : resumeMatchLoading ? (
        <p className="m-0 text-sm text-muted-foreground">Analyzing resume keyword coverage...</p>
      ) : resumeMatchError ? (
        <p className="m-0 text-sm text-content-warning">
          Resume match analysis is unavailable right now.
        </p>
      ) : resumeMatch ? (
        <div className="space-y-4">
          <p className="m-0 text-sm leading-6 text-muted-foreground">{resumeMatch.gapSummary}</p>
          <div className="grid gap-4 xl:grid-cols-2">
            <KeywordList
              title="Covered JD keywords"
              items={resumeMatch.matchedKeywords}
              empty="No covered JD keywords found in the active CV."
              variant="success"
            />
            <KeywordList
              title="Missing JD keywords"
              items={resumeMatch.missingKeywords}
              empty="No high-signal JD keywords are missing."
              variant="danger"
            />
          </div>
        </div>
      ) : (
        <p className="m-0 text-sm text-muted-foreground">
          This job needs a description before resume keyword coverage can run.
        </p>
      )}
    </div>
  );
}

function KeywordList({
  title,
  items,
  empty,
  variant,
}: {
  title: string;
  items: string[];
  empty: string;
  variant: 'success' | 'danger';
}) {
  return (
    <div>
      <p className="mb-2 text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
        {title}
      </p>
      {items.length > 0 ? (
        <div className="flex flex-wrap gap-2">
          {items.map((item) => (
            <Badge key={item} variant={variant} className="px-3 py-1 text-xs">
              {item}
            </Badge>
          ))}
        </div>
      ) : (
        <p className="m-0 text-sm text-muted-foreground">{empty}</p>
      )}
    </div>
  );
}

function ScorePart({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-2 text-xl font-semibold text-card-foreground">
        {value > 0 ? '+' : ''}
        {value}
      </p>
    </div>
  );
}

function ReasonPanel({
  title,
  icon,
  tone,
  items,
  empty,
}: {
  title: string;
  icon: ReactNode;
  tone: 'success' | 'warning';
  items: string[];
  empty: string;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p
        className={`mb-3 flex items-center gap-2 text-sm font-medium ${tone === 'success' ? 'text-content-success' : 'text-content-warning'}`}
      >
        {icon}
        {title}
      </p>
      {items.length > 0 ? (
        <div className="space-y-3">
          {items.map((entry) => (
            <div key={entry} className="flex items-start gap-3">
              <span
                className={`mt-1 h-2 w-2 shrink-0 rounded-full ${tone === 'success' ? 'bg-fit-excellent' : 'bg-content-warning'}`}
              />
              <p className="m-0 text-sm leading-6 text-muted-foreground">{entry}</p>
            </div>
          ))}
        </div>
      ) : (
        <p className="m-0 text-sm text-muted-foreground">{empty}</p>
      )}
    </div>
  );
}

function TermPanel({
  title,
  icon,
  details,
}: {
  title: string;
  icon: ReactNode;
  details: MissingSignalDetail[];
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-warning">
        {icon}
        {title}
      </p>
      {details.length > 0 ? (
        <div className="flex flex-wrap gap-2">
          {details.map((detail) => (
            <Badge
              key={`${detail.category}-${detail.term}`}
              variant="danger"
              className="px-3 py-1 text-xs"
            >
              {detail.term} · {formatCategory(detail.category)}
            </Badge>
          ))}
        </div>
      ) : (
        <p className="m-0 text-sm text-muted-foreground">No missing signals returned.</p>
      )}
    </div>
  );
}

function formatCategory(category: string) {
  return category.replace(/_/g, ' ');
}
