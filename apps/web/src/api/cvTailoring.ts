import type { CvTailoringResponse } from '@job-copilot/shared/cv-tailoring';

import { json, request } from './client';

type EngineCvTailoringGapItem = {
  skill: string;
  suggestion: string;
};

type EngineCvTailoringSuggestions = {
  skills_to_highlight?: string[];
  skills_to_mention?: string[];
  gaps_to_address?: EngineCvTailoringGapItem[];
  summary_rewrite?: string;
  key_phrases?: string[];
};

type EngineCvTailoringResponse = {
  suggestions: EngineCvTailoringSuggestions;
  provider: string;
  generated_at: string;
};

function normalizeStringList(value: string[] | undefined): string[] {
  return Array.isArray(value)
    ? value.map((item) => item.trim()).filter(Boolean)
    : [];
}

function mapCvTailoringResponse(
  response: EngineCvTailoringResponse,
): CvTailoringResponse {
  return {
    suggestions: {
      skillsToHighlight: normalizeStringList(
        response.suggestions.skills_to_highlight,
      ),
      skillsToMention: normalizeStringList(
        response.suggestions.skills_to_mention,
      ),
      gapsToAddress: Array.isArray(response.suggestions.gaps_to_address)
        ? response.suggestions.gaps_to_address
            .map((gap) => ({
              skill: gap.skill.trim(),
              suggestion: gap.suggestion.trim(),
            }))
            .filter((gap) => gap.skill || gap.suggestion)
        : [],
      summaryRewrite: response.suggestions.summary_rewrite?.trim() ?? '',
      keyPhrases: normalizeStringList(response.suggestions.key_phrases),
    },
    provider: response.provider,
    generatedAt: response.generated_at,
  };
}

export async function tailorCvForJob(
  jobId: string,
): Promise<CvTailoringResponse> {
  const response = await request<EngineCvTailoringResponse>(
    '/api/v1/cv/tailor',
    json('POST', {
      job_id: jobId,
    }),
  );

  return mapCvTailoringResponse(response);
}