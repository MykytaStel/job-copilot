import { json, mlRequest } from '../client';

export type MlResumeMatchResponse = {
  keyword_coverage_percent: number;
  matched_keywords: string[];
  missing_keywords: string[];
  gap_summary: string;
};

export type ResumeMatch = {
  keywordCoveragePercent: number;
  matchedKeywords: string[];
  missingKeywords: string[];
  gapSummary: string;
};

export async function getResumeMatch(payload: {
  resumeText: string;
  jdText: string;
}): Promise<ResumeMatch> {
  const response = await mlRequest<MlResumeMatchResponse>(
    '/api/v1/enrichment/resume-match',
    json('POST', {
      resume_text: payload.resumeText,
      jd_text: payload.jdText,
    }),
  );

  return {
    keywordCoveragePercent: response.keyword_coverage_percent,
    matchedKeywords: response.matched_keywords,
    missingKeywords: response.missing_keywords,
    gapSummary: response.gap_summary,
  };
}
