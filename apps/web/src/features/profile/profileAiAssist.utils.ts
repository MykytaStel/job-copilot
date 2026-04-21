import type { LlmContext } from '../../api/analytics';
import type { JobFitExplanationRequest } from '../../api/enrichment';
import type { RankedJobResult } from '../../api/jobs';

export function renderAiErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

export function requireAiRequestContext(
  profileId: string | null,
  llmContext: LlmContext | null,
  missingProfileMessage: string,
) {
  if (!profileId) {
    throw new Error(missingProfileMessage);
  }

  if (!llmContext) {
    throw new Error('Feedback-aware context is not ready yet.');
  }

  return { profileId, llmContext };
}

export function buildAiFeedbackState(
  llmContext: LlmContext,
  result: RankedJobResult,
): NonNullable<JobFitExplanationRequest['feedbackState']> {
  return {
    feedbackSummary: llmContext.feedbackSummary,
    topPositiveEvidence: llmContext.topPositiveEvidence,
    topNegativeEvidence: llmContext.topNegativeEvidence,
    currentJobFeedback: result.job.feedback,
  };
}
