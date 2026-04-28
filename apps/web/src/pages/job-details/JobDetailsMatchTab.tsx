import { AlertCircle, CheckCircle2, MinusCircle, Sparkles } from 'lucide-react';
import type { ReactNode } from 'react';

import type { JobPosting } from '@job-copilot/shared/jobs';
import type { FitAnalysis, MissingSignalDetail } from '../../api/jobs';
import { Badge } from '../../components/ui/Badge';
import { FitScoreCircular } from '../../components/ui/FitScoreBox';
import { Section } from './components';

export function JobDetailsMatchTab({
  fit,
  job,
}: {
  fit: FitAnalysis | undefined;
  job?: JobPosting;
}) {
  const scoreSignals = job?.presentation?.scoreSignals ?? [];

  if (!fit) {
    return (
      <Section
        title="Match Breakdown"
        description="Detailed evidence, matched terms, and missing signals from the current fit analysis."
        icon={Sparkles}
      >
        <p className="m-0 text-sm text-muted-foreground">Fit analysis is not ready yet.</p>
      </Section>
    );
  }

  return (
    <Section
      title="Match Breakdown"
      description="Matched roles explain why the job is relevant; missing signals show target/profile evidence not found strongly enough in this posting."
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
            <p className="m-0 text-xs leading-5 text-muted-foreground">
              A Mobile Engineer role can match through mobile, Kotlin, iOS, or Android signals while
              React Native remains missing when that specific term is not evidenced strongly enough.
            </p>
          </div>
        </div>

        <div className="grid gap-4 xl:grid-cols-3">
          <ScorePart label="Matching" value={fit.scoreBreakdown.matchingScore} />
          <ScorePart label="Salary" value={fit.scoreBreakdown.salaryScore} />
          <ScorePart label="Freshness" value={fit.scoreBreakdown.freshnessScore} />
        </div>

        {scoreSignals.length > 0 || fit.scoreBreakdown.penalties.length > 0 ? (
          <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
            <p className="mb-3 flex items-center gap-2 text-sm font-medium text-card-foreground">
              <MinusCircle className="h-4 w-4" />
              Score signals
            </p>
            <div className="flex flex-wrap gap-2">
              {scoreSignals.map((signal) => (
                <Badge key={`${signal.label}-${signal.delta}`} variant={signal.delta >= 0 ? 'success' : 'danger'}>
                  {signal.label} ({signal.delta > 0 ? '+' : ''}{signal.delta})
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

        <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
          <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-success">
            <CheckCircle2 className="h-4 w-4" />
            Matched terms
          </p>
          {fit.matchedTerms.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {fit.matchedTerms.map((term) => (
                <Badge key={term} variant="success" className="px-3 py-1 text-xs">
                  {term}
                </Badge>
              ))}
            </div>
          ) : (
            <p className="m-0 text-sm text-muted-foreground">No matched terms returned.</p>
          )}
        </div>
      </div>
    </Section>
  );
}

function ScorePart({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-2 text-xl font-semibold text-card-foreground">
        {value > 0 ? '+' : ''}{value}
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
      <p className={`mb-3 flex items-center gap-2 text-sm font-medium ${tone === 'success' ? 'text-content-success' : 'text-content-warning'}`}>
        {icon}
        {title}
      </p>
      {items.length > 0 ? (
        <div className="space-y-3">
          {items.map((entry) => (
            <div key={entry} className="flex items-start gap-3">
              <span className={`mt-1 h-2 w-2 shrink-0 rounded-full ${tone === 'success' ? 'bg-fit-excellent' : 'bg-content-warning'}`} />
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
            <Badge key={`${detail.category}-${detail.term}`} variant="danger" className="px-3 py-1 text-xs">
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
