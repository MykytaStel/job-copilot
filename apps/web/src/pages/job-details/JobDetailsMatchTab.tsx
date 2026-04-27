import { AlertCircle, CheckCircle2, Sparkles } from 'lucide-react';

import type { JobPosting } from '@job-copilot/shared/jobs'; import type { FitAnalysis } from '../../api/jobs';
import { Badge } from '../../components/ui/Badge';
import { FitScoreCircular } from '../../components/ui/FitScoreBox';
import { Section } from './components';

export function JobDetailsMatchTab({ fit, job }: { fit: FitAnalysis | undefined; job?: JobPosting }) {
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
      description="Detailed evidence, matched terms, and missing signals from the current fit analysis."
      icon={Sparkles}
    >
      <div className="space-y-5">
        <div className="flex items-start gap-4 rounded-2xl border border-primary/20 bg-primary/5 p-4">
          <FitScoreCircular score={fit.score} size="md" showLabel />
          <p className="m-0 text-sm leading-7 text-muted-foreground">
            {fit.positiveReasons[0] ??
              fit.negativeReasons[0] ??
              'Fit analysis is based on canonical Rust matching over the stored profile and job signals.'}
          </p>
        </div>

        <div className="grid gap-4 xl:grid-cols-2">
          <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
            <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-success">
              <CheckCircle2 className="h-4 w-4" />
              {scoreSignals.length > 0 ? (
        <div className="mb-4 flex flex-wrap gap-2">
          {scoreSignals.map((signal) => (
            <span
              key={`${signal.label}-${signal.delta}`}
              className={
                signal.delta >= 0
                  ? 'rounded-full border border-emerald-400/30 bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-200'
                  : 'rounded-full border border-red-400/30 bg-red-500/10 px-3 py-1 text-xs font-medium text-red-200'
              }
            >
              {signal.label} ({signal.delta > 0 ? '+' : ''}{signal.delta})
            </span>
          ))}
        </div>
      ) : null}

      Strengths
            </p>
            {fit.positiveReasons.length > 0 ? (
              <div className="space-y-3">
                {fit.positiveReasons.map((entry) => (
                  <div key={entry} className="flex items-start gap-3">
                    <span className="mt-1 h-2 w-2 shrink-0 rounded-full bg-fit-excellent" />
                    <p className="m-0 text-sm leading-6 text-muted-foreground">{entry}</p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">No positive signals yet.</p>
            )}
          </div>

          <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
            <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-warning">
              <AlertCircle className="h-4 w-4" />
              Penalties and risks
            </p>
            {fit.negativeReasons.length > 0 ? (
              <div className="space-y-3">
                {fit.negativeReasons.map((entry) => (
                  <div key={entry} className="flex items-start gap-3">
                    <span className="mt-1 h-2 w-2 shrink-0 rounded-full bg-content-warning" />
                    <p className="m-0 text-sm leading-6 text-muted-foreground">{entry}</p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">No penalties returned.</p>
            )}
          </div>
        </div>

        <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
          <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-warning">
            <AlertCircle className="h-4 w-4" />
            Missing signals
          </p>
          {fit.missingTerms.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {fit.missingTerms.map((term) => (
                <Badge key={term} variant="danger" className="px-3 py-1 text-xs">
                  {term}
                </Badge>
              ))}
            </div>
          ) : (
            <p className="m-0 text-sm text-muted-foreground">No missing signals returned.</p>
          )}
        </div>

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
