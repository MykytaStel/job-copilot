import type { LlmContext } from '../../api/analytics';
import type { RankedJobResult, SearchProfileBuildResult } from '../../api/jobs';

import {
  ApplicationCoachPanel,
  CoverLetterDraftPanel,
  FitExplanationPanel,
  InterviewPrepPanel,
} from './FitPanels';
import { SearchResultAiActionSection } from './SearchResultAiActionSection';
import { renderAiErrorMessage } from './profileAiAssist.utils';
import { useSearchResultAiAssist } from './useSearchResultAiAssist';

export function SearchResultFitExplanation({
  analyzedProfile,
  searchProfile,
  result,
  profileId,
  rawProfileText,
  llmContext,
  llmContextError,
  llmContextLoading,
}: {
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'];
  searchProfile: SearchProfileBuildResult['searchProfile'];
  result: RankedJobResult;
  profileId: string | null;
  rawProfileText: string;
  llmContext: LlmContext | null;
  llmContextError: unknown;
  llmContextLoading: boolean;
}) {
  const {
    explanation,
    coaching,
    coverLetterDraft,
    interviewPrep,
    explainMutation,
    coachMutation,
    coverLetterMutation,
    interviewPrepMutation,
  } = useSearchResultAiAssist({
    analyzedProfile,
    searchProfile,
    result,
    profileId,
    rawProfileText,
    llmContext,
  });

  const aiContextReady = !llmContextLoading && !llmContextError && Boolean(llmContext);

  return (
    <div className="resultSection" style={{ marginTop: 12 }}>
      <SearchResultAiActionSection
        label="LLM fit explanation"
        actionLabel="Explain fit"
        refreshLabel="Refresh explanation"
        pendingLabel="Explaining…"
        isPending={explainMutation.isPending}
        hasContent={Boolean(explanation)}
        disabled={
          explainMutation.isPending ||
          !profileId ||
          llmContextLoading ||
          Boolean(llmContextError) ||
          !llmContext
        }
        onAction={() => explainMutation.mutate()}
        error={explainMutation.error}
        errorFallback="Fit explanation is unavailable right now."
      >
        {llmContextLoading && (
          <p className="m-0 text-sm leading-6 text-muted-foreground">
            Feedback-aware context is loading. Fit explanation will be available once it is ready.
          </p>
        )}

        {!llmContextLoading && Boolean(llmContextError) && (
          <p className="error" style={{ marginBottom: 0 }}>
            {renderAiErrorMessage(
              llmContextError,
              'Feedback-aware context is unavailable right now.',
            )}
          </p>
        )}

        {explanation && <FitExplanationPanel explanation={explanation} />}
      </SearchResultAiActionSection>

      {aiContextReady && (
        <div style={{ marginTop: explanation ? 12 : 0 }}>
          <SearchResultAiActionSection
            label="Application coaching"
            actionLabel="Coach application"
            refreshLabel="Refresh coaching"
            pendingLabel="Coaching…"
            isPending={coachMutation.isPending}
            hasContent={Boolean(coaching)}
            disabled={coachMutation.isPending || !profileId}
            onAction={() => coachMutation.mutate()}
            error={coachMutation.error}
            errorFallback="Application coaching is unavailable right now."
          >
            {coaching && <ApplicationCoachPanel coaching={coaching} />}
          </SearchResultAiActionSection>

          <div style={{ marginTop: coaching ? 12 : 0 }}>
            <SearchResultAiActionSection
              label="Cover letter draft"
              actionLabel="Draft cover letter"
              refreshLabel="Refresh draft"
              pendingLabel="Drafting…"
              isPending={coverLetterMutation.isPending}
              hasContent={Boolean(coverLetterDraft)}
              disabled={coverLetterMutation.isPending || !profileId}
              onAction={() => coverLetterMutation.mutate()}
              error={coverLetterMutation.error}
              errorFallback="Cover letter drafting is unavailable right now."
            >
              {coverLetterDraft && <CoverLetterDraftPanel draft={coverLetterDraft} />}
            </SearchResultAiActionSection>
          </div>

          <div style={{ marginTop: coverLetterDraft ? 12 : 0 }}>
            <SearchResultAiActionSection
              label="Interview prep pack"
              actionLabel="Prepare interview"
              refreshLabel="Refresh prep"
              pendingLabel="Preparing…"
              isPending={interviewPrepMutation.isPending}
              hasContent={Boolean(interviewPrep)}
              disabled={interviewPrepMutation.isPending || !profileId}
              onAction={() => interviewPrepMutation.mutate()}
              error={interviewPrepMutation.error}
              errorFallback="Interview prep is unavailable right now."
            >
              {interviewPrep && <InterviewPrepPanel prep={interviewPrep} />}
            </SearchResultAiActionSection>
          </div>
        </div>
      )}
    </div>
  );
}
