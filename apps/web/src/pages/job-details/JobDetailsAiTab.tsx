import {
  AlertCircle,
  CheckCircle2,
  FileText,
  Lightbulb,
  Loader2,
  MessageSquare,
  Sparkles,
} from 'lucide-react';

import type { CoverLetterDraft, InterviewPrep, JobFitExplanation } from '../../api/enrichment';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import {
  semanticDotClass,
  semanticPanelClass,
  semanticTextClass,
} from '../../components/ui/semanticTone';
import { Section } from './components';

export function JobDetailsAiTab({
  profileId,
  deterministicFit,
  fitExplanation,
  fitExplanationLoading,
  coverLetter,
  coverLetterLoading,
  interviewPrep,
  interviewPrepLoading,
  setGenerateCoverLetter,
  setGenerateInterviewPrep,
}: {
  profileId: string | null;
  deterministicFit: unknown;
  fitExplanation: JobFitExplanation | undefined;
  fitExplanationLoading: boolean;
  coverLetter: CoverLetterDraft | undefined;
  coverLetterLoading: boolean;
  interviewPrep: InterviewPrep | undefined;
  interviewPrepLoading: boolean;
  setGenerateCoverLetter: (v: boolean) => void;
  setGenerateInterviewPrep: (v: boolean) => void;
}) {
  if (!profileId) {
    return <EmptyState message="Create a profile to enable AI analysis." />;
  }

  if (!deterministicFit) {
    return (
      <div className="flex items-center gap-3 rounded-2xl border border-border/70 bg-surface-muted p-6">
        <Loader2 className="h-5 w-5 animate-spin text-primary" />
        <p className="m-0 text-sm text-muted-foreground">Завантажуємо fit аналіз…</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <Section
        title="Why this job?"
        description="LLM-generated analysis of how well this role aligns with your profile."
        icon={Sparkles}
      >
        {fitExplanationLoading ? (
          <div className="flex items-center gap-3 py-4">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
            <p className="m-0 text-sm text-muted-foreground">Ollama аналізує вакансію…</p>
          </div>
        ) : fitExplanation ? (
          <div className="space-y-5">
            {fitExplanation.fitSummary ? (
              <div className={`rounded-2xl border p-4 ${semanticPanelClass.primary}`}>
                <p className="m-0 text-sm leading-7 text-card-foreground">
                  {fitExplanation.fitSummary}
                </p>
              </div>
            ) : null}

            <div className="grid gap-4 xl:grid-cols-2">
              {fitExplanation.whyItMatches.length > 0 ? (
                <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                  <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-success">
                    <CheckCircle2 className="h-4 w-4" />
                    Чому підходить
                  </p>
                  <div className="space-y-2">
                    {fitExplanation.whyItMatches.map((item) => (
                      <div key={item} className="flex items-start gap-3">
                        <span className={`mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full ${semanticDotClass.success}`} />
                        <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}

              {fitExplanation.risks.length > 0 ? (
                <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                  <p className="mb-3 flex items-center gap-2 text-sm font-medium text-content-warning">
                    <AlertCircle className="h-4 w-4" />
                    Ризики
                  </p>
                  <div className="space-y-2">
                    {fitExplanation.risks.map((item) => (
                      <div key={item} className="flex items-start gap-3">
                        <span className={`mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full ${semanticDotClass.warning}`} />
                        <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}
            </div>

            {fitExplanation.missingSignals.length > 0 ? (
              <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                <p className="mb-3 text-sm font-medium text-muted-foreground">Чого бракує</p>
                <div className="flex flex-wrap gap-2">
                  {fitExplanation.missingSignals.map((s) => (
                    <Badge key={s} variant="danger" className="px-3 py-1 text-xs">
                      {s}
                    </Badge>
                  ))}
                </div>
              </div>
            ) : null}

            {fitExplanation.recommendedNextStep ? (
              <div className={`rounded-2xl border p-4 ${semanticPanelClass.primary}`}>
                <p className={`mb-1 flex items-center gap-2 text-sm font-medium ${semanticTextClass.primary}`}>
                  <Lightbulb className="h-4 w-4" />
                  Наступний крок
                </p>
                <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
                  {fitExplanation.recommendedNextStep}
                </p>
              </div>
            ) : null}

            {fitExplanation.applicationAngle ? (
              <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                <p className="mb-1 text-sm font-medium text-card-foreground">Як подаватись</p>
                <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
                  {fitExplanation.applicationAngle}
                </p>
              </div>
            ) : null}
          </div>
        ) : (
          <EmptyState message="Fit explanation not available." />
        )}
      </Section>

      <Section
        title="Cover Letter"
        description="AI-generated cover letter tailored to this vacancy and your profile."
        icon={FileText}
      >
        {coverLetter ? (
          <div className="space-y-4">
            {coverLetter.draftSummary ? (
              <p className="m-0 text-sm italic text-muted-foreground">{coverLetter.draftSummary}</p>
            ) : null}
            <div className="rounded-2xl border border-border/70 bg-surface-muted p-5 space-y-4 text-sm leading-7 text-card-foreground">
              <p className="m-0">{coverLetter.openingParagraph}</p>
              {coverLetter.bodyParagraphs.map((p, i) => (
                <p key={i} className="m-0">
                  {p}
                </p>
              ))}
              <p className="m-0">{coverLetter.closingParagraph}</p>
            </div>
            {coverLetter.keyClaimsUsed.length > 0 ? (
              <div>
                <p className="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
                  Ключові аргументи
                </p>
                <div className="flex flex-wrap gap-2">
                  {coverLetter.keyClaimsUsed.map((c) => (
                    <Badge key={c} variant="muted" className="px-3 py-1 text-xs">
                      {c}
                    </Badge>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        ) : coverLetterLoading ? (
          <div className="flex items-center gap-3 py-4">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
            <p className="m-0 text-sm text-muted-foreground">Ollama генерує cover letter…</p>
          </div>
        ) : (
          <Button
            onClick={() => setGenerateCoverLetter(true)}
            disabled={!fitExplanation}
            variant="outline"
            className="gap-2"
          >
            <FileText className="h-4 w-4" />
            {fitExplanation ? 'Generate Cover Letter' : 'Зачекай на fit analysis…'}
          </Button>
        )}
      </Section>

      <Section
        title="Interview Prep"
        description="Topics, questions, and stories to prepare based on this role."
        icon={MessageSquare}
      >
        {interviewPrep ? (
          <div className="space-y-5">
            {interviewPrep.prepSummary ? (
              <div className={`rounded-2xl border p-4 ${semanticPanelClass.primary}`}>
                <p className="m-0 text-sm leading-7 text-card-foreground">
                  {interviewPrep.prepSummary}
                </p>
              </div>
            ) : null}

            <div className="grid gap-4 xl:grid-cols-2">
              {interviewPrep.technicalFocus.length > 0 ? (
                <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                  <p className="mb-3 text-sm font-medium text-card-foreground">Технічні теми</p>
                  <div className="space-y-2">
                    {interviewPrep.technicalFocus.map((item) => (
                      <div key={item} className="flex items-start gap-3">
                        <span className={`mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full ${semanticDotClass.primary}`} />
                        <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}

              {interviewPrep.behavioralFocus.length > 0 ? (
                <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                  <p className="mb-3 text-sm font-medium text-card-foreground">
                    Поведінкові питання
                  </p>
                  <div className="space-y-2">
                    {interviewPrep.behavioralFocus.map((item) => (
                      <div key={item} className="flex items-start gap-3">
                        <span className={`mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full ${semanticDotClass.info}`} />
                        <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}
            </div>

            {interviewPrep.questionsToAsk.length > 0 ? (
              <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                <p className="mb-3 text-sm font-medium text-card-foreground">
                  Питання до роботодавця
                </p>
                <div className="space-y-2">
                  {interviewPrep.questionsToAsk.map((item, i) => (
                    <div key={i} className="flex items-start gap-3">
                      <span className={`mt-1 shrink-0 text-xs font-semibold ${semanticTextClass.primary}`}>
                        {i + 1}.
                      </span>
                      <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}

            {interviewPrep.storiesToPrepare.length > 0 ? (
              <div className="rounded-2xl border border-border/70 bg-surface-muted p-4">
                <p className="mb-3 text-sm font-medium text-card-foreground">
                  Історії для підготовки
                </p>
                <div className="space-y-2">
                  {interviewPrep.storiesToPrepare.map((item) => (
                    <div key={item} className="flex items-start gap-3">
                      <span className={`mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full ${semanticDotClass.warning}`} />
                      <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        ) : interviewPrepLoading ? (
          <div className="flex items-center gap-3 py-4">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
            <p className="m-0 text-sm text-muted-foreground">Ollama готує interview prep…</p>
          </div>
        ) : (
          <Button
            onClick={() => setGenerateInterviewPrep(true)}
            variant="outline"
            className="gap-2"
          >
            <MessageSquare className="h-4 w-4" />
            Generate Interview Prep
          </Button>
        )}
      </Section>
    </div>
  );
}
