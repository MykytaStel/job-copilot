import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';

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

import { buildAiFeedbackState, requireAiRequestContext } from './profileAiAssist.utils';

export function useSearchResultAiAssist({
  analyzedProfile,
  searchProfile,
  result,
  profileId,
  rawProfileText,
  llmContext,
}: {
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'];
  searchProfile: SearchProfileBuildResult['searchProfile'];
  result: RankedJobResult;
  profileId: string | null;
  rawProfileText: string;
  llmContext: LlmContext | null;
}) {
  const [explanation, setExplanation] = useState<JobFitExplanation | null>(null);
  const [coaching, setCoaching] = useState<ApplicationCoach | null>(null);
  const [coverLetterDraft, setCoverLetterDraft] = useState<CoverLetterDraft | null>(null);
  const [interviewPrep, setInterviewPrep] = useState<InterviewPrep | null>(null);

  const explainMutation = useMutation({
    mutationFn: async () => {
      const context = requireAiRequestContext(
        profileId,
        llmContext,
        'Profile is required before requesting fit explanation.',
      );

      return getJobFitExplanation({
        profileId: context.profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        feedbackState: buildAiFeedbackState(context.llmContext, result),
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
      const context = requireAiRequestContext(
        profileId,
        llmContext,
        'Profile is required before requesting application coaching.',
      );

      return getApplicationCoach({
        profileId: context.profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        feedbackState: buildAiFeedbackState(context.llmContext, result),
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
      const context = requireAiRequestContext(
        profileId,
        llmContext,
        'Profile is required before drafting a cover letter.',
      );

      return getCoverLetterDraft({
        profileId: context.profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        applicationCoach: coaching,
        feedbackState: buildAiFeedbackState(context.llmContext, result),
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
      const context = requireAiRequestContext(
        profileId,
        llmContext,
        'Profile is required before preparing interview guidance.',
      );

      return getInterviewPrep({
        profileId: context.profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        applicationCoach: coaching,
        coverLetterDraft,
        feedbackState: buildAiFeedbackState(context.llmContext, result),
        rawProfileText: rawProfileText.trim() ? rawProfileText : null,
      });
    },
    onSuccess: (payload) => {
      setInterviewPrep(payload);
    },
  });

  return {
    explanation,
    coaching,
    coverLetterDraft,
    interviewPrep,
    explainMutation,
    coachMutation,
    coverLetterMutation,
    interviewPrepMutation,
  };
}
