import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { Sparkles } from 'lucide-react';

import type {
  ApplicationCoach,
  CoverLetterDraft,
  InterviewPrep,
  JobFitExplanation,
} from '../../api/enrichment';
import type { LlmContext } from '../../api/analytics';
import type { RankedJobResult, SearchProfileBuildResult } from '../../api/jobs';
import {
  getApplicationCoach,
  getCoverLetterDraft,
  getInterviewPrep,
  getJobFitExplanation,
} from '../../api/enrichment';
import { Button } from '../../components/ui/Button';
import {
  ApplicationCoachPanel,
  CoverLetterDraftPanel,
  FitExplanationPanel,
  InterviewPrepPanel,
} from './FitPanels';

function renderErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

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
  const [explanation, setExplanation] = useState<JobFitExplanation | null>(null);
  const [coaching, setCoaching] = useState<ApplicationCoach | null>(null);
  const [coverLetterDraft, setCoverLetterDraft] = useState<CoverLetterDraft | null>(null);
  const [interviewPrep, setInterviewPrep] = useState<InterviewPrep | null>(null);

  const explainMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before requesting fit explanation.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getJobFitExplanation({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
      });
    },
    onSuccess: (payload) => {
      setExplanation(payload);
      setCoverLetterDraft(null);
      setInterviewPrep(null);
    },
  });

  const coachMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before requesting application coaching.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getApplicationCoach({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
      });
    },
    onSuccess: (payload) => {
      setCoaching(payload);
      setCoverLetterDraft(null);
      setInterviewPrep(null);
    },
  });

  const coverLetterMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before drafting a cover letter.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getCoverLetterDraft({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        applicationCoach: coaching,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
        rawProfileText: rawProfileText.trim() ? rawProfileText : null,
      });
    },
    onSuccess: (payload) => {
      setCoverLetterDraft(payload);
      setInterviewPrep(null);
    },
  });

  const interviewPrepMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before preparing interview guidance.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getInterviewPrep({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        applicationCoach: coaching,
        coverLetterDraft,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
        rawProfileText: rawProfileText.trim() ? rawProfileText : null,
      });
    },
    onSuccess: (payload) => {
      setInterviewPrep(payload);
    },
  });

  return (
    <div className="resultSection" style={{ marginTop: 12 }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 12,
          marginBottom: explanation || explainMutation.isPending || explainMutation.error ? 10 : 0,
        }}
      >
        <span className="detailLabel">LLM fit explanation</span>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          disabled={
            explainMutation.isPending ||
            !profileId ||
            llmContextLoading ||
            !!llmContextError ||
            !llmContext
          }
          onClick={() => explainMutation.mutate()}
        >
          <Sparkles size={13} />
          {explainMutation.isPending
            ? 'Explaining…'
            : explanation
              ? 'Refresh explanation'
              : 'Explain fit'}
        </Button>
      </div>

      {llmContextLoading && (
        <p className="m-0 text-sm leading-6 text-muted-foreground">
          Feedback-aware context is loading. Fit explanation will be available once it is ready.
        </p>
      )}

      {!llmContextLoading && Boolean(llmContextError) && (
        <p className="error" style={{ marginBottom: 0 }}>
          {renderErrorMessage(llmContextError, 'Feedback-aware context is unavailable right now.')}
        </p>
      )}

      {explainMutation.error && (
        <p className="error" style={{ marginBottom: 0 }}>
          {renderErrorMessage(explainMutation.error, 'Fit explanation is unavailable right now.')}
        </p>
      )}

      {explanation && <FitExplanationPanel explanation={explanation} />}

      {!llmContextLoading && !llmContextError && llmContext && (
        <div style={{ marginTop: explanation ? 12 : 0 }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              gap: 12,
              marginBottom: coaching || coachMutation.isPending || coachMutation.error ? 10 : 0,
            }}
          >
            <span className="detailLabel">Application coaching</span>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={coachMutation.isPending || !profileId}
              onClick={() => coachMutation.mutate()}
            >
              <Sparkles size={13} />
              {coachMutation.isPending
                ? 'Coaching…'
                : coaching
                  ? 'Refresh coaching'
                  : 'Coach application'}
            </Button>
          </div>

          {coachMutation.error && (
            <p className="error" style={{ marginBottom: 0 }}>
              {renderErrorMessage(
                coachMutation.error,
                'Application coaching is unavailable right now.',
              )}
            </p>
          )}

          {coaching && <ApplicationCoachPanel coaching={coaching} />}

          <div style={{ marginTop: coaching ? 12 : 0 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 12,
                marginBottom:
                  coverLetterDraft || coverLetterMutation.isPending || coverLetterMutation.error
                    ? 10
                    : 0,
              }}
            >
              <span className="detailLabel">Cover letter draft</span>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                disabled={coverLetterMutation.isPending || !profileId}
                onClick={() => coverLetterMutation.mutate()}
              >
                <Sparkles size={13} />
                {coverLetterMutation.isPending
                  ? 'Drafting…'
                  : coverLetterDraft
                    ? 'Refresh draft'
                    : 'Draft cover letter'}
              </Button>
            </div>

            {coverLetterMutation.error && (
              <p className="error" style={{ marginBottom: 0 }}>
                {renderErrorMessage(
                  coverLetterMutation.error,
                  'Cover letter drafting is unavailable right now.',
                )}
              </p>
            )}

            {coverLetterDraft && <CoverLetterDraftPanel draft={coverLetterDraft} />}
          </div>

          <div style={{ marginTop: coverLetterDraft ? 12 : 0 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 12,
                marginBottom:
                  interviewPrep || interviewPrepMutation.isPending || interviewPrepMutation.error
                    ? 10
                    : 0,
              }}
            >
              <span className="detailLabel">Interview prep pack</span>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                disabled={interviewPrepMutation.isPending || !profileId}
                onClick={() => interviewPrepMutation.mutate()}
              >
                <Sparkles size={13} />
                {interviewPrepMutation.isPending
                  ? 'Preparing…'
                  : interviewPrep
                    ? 'Refresh prep'
                    : 'Prepare interview'}
              </Button>
            </div>

            {interviewPrepMutation.error && (
              <p className="error" style={{ marginBottom: 0 }}>
                {renderErrorMessage(
                  interviewPrepMutation.error,
                  'Interview prep is unavailable right now.',
                )}
              </p>
            )}

            {interviewPrep && <InterviewPrepPanel prep={interviewPrep} />}
          </div>
        </div>
      )}
    </div>
  );
}
