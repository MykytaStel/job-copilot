import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { getWeeklyGuidance } from '../src/api';

function jsonResponse(body: unknown) {
  return Promise.resolve({
    ok: true,
    status: 200,
    json: async () => body,
  });
}

describe('enrichment api payloads', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
    vi.stubGlobal('window', {
      localStorage: {
        getItem: vi.fn(() => null),
        setItem: vi.fn(),
      },
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('getWeeklyGuidance serializes nested enrichment payloads in engine/ml snake_case', async () => {
    vi.mocked(fetch).mockImplementationOnce(
      () =>
        jsonResponse({
          weekly_summary: 'summary',
          what_is_working: ['focus'],
          what_is_not_working: ['noise'],
          recommended_search_adjustments: ['narrow search'],
          recommended_source_moves: ['prioritize djinni'],
          recommended_role_focus: ['backend developer'],
          funnel_bottlenecks: ['save to apply'],
          next_week_plan: ['apply faster'],
        }) as ReturnType<typeof fetch>,
    );

    await getWeeklyGuidance({
      profileId: 'profile-1',
      analyticsSummary: {
        profileId: 'profile-1',
        feedback: {
          savedJobsCount: 3,
          hiddenJobsCount: 1,
          badFitJobsCount: 1,
          whitelistedCompaniesCount: 1,
          blacklistedCompaniesCount: 0,
        },
        jobsBySource: [{ source: 'djinni', count: 12 }],
        jobsByLifecycle: { total: 12, active: 10, inactive: 1, reactivated: 1 },
        topMatchedRoles: ['backend_developer'],
        topMatchedSkills: ['rust'],
        topMatchedKeywords: ['distributed systems'],
        searchQuality: {
          lowEvidenceJobs: 1,
          weakDescriptionJobs: 1,
          roleMismatchJobs: 0,
          seniorityMismatchJobs: 0,
          sourceMismatchJobs: 0,
          topMissingSignals: ['postgres'],
        },
      },
      behaviorSummary: {
        profileId: 'profile-1',
        searchRunCount: 5,
        topPositiveSources: [
          {
            key: 'djinni',
            saveCount: 3,
            hideCount: 0,
            badFitCount: 0,
            applicationCreatedCount: 1,
            positiveCount: 3,
            negativeCount: 0,
            netScore: 4,
          },
        ],
        topNegativeSources: [],
        topPositiveRoleFamilies: [],
        topNegativeRoleFamilies: [],
        sourceSignalCounts: [],
        roleFamilySignalCounts: [],
      },
      funnelSummary: {
        profileId: 'profile-1',
        impressionCount: 20,
        openCount: 8,
        saveCount: 3,
        hideCount: 1,
        badFitCount: 1,
        applicationCreatedCount: 1,
        fitExplanationRequestedCount: 2,
        applicationCoachRequestedCount: 1,
        coverLetterDraftRequestedCount: 1,
        interviewPrepRequestedCount: 0,
        conversionRates: {
          openRateFromImpressions: 0.4,
          saveRateFromOpens: 0.375,
          applicationRateFromSaves: 0.33,
        },
        impressionsBySource: [{ source: 'djinni', count: 20 }],
        opensBySource: [{ source: 'djinni', count: 8 }],
        savesBySource: [{ source: 'djinni', count: 3 }],
        applicationsBySource: [{ source: 'djinni', count: 1 }],
      },
      llmContext: {
        profileId: 'profile-1',
        analyzedProfile: {
          summary: 'Senior backend engineer',
          primaryRole: 'backend_developer',
          seniority: 'senior',
          skills: ['rust'],
          keywords: ['distributed systems'],
        },
        profileSkills: ['rust'],
        profileKeywords: ['distributed systems'],
        jobsFeedSummary: { total: 12, active: 10, inactive: 1, reactivated: 1 },
        feedbackSummary: {
          savedJobsCount: 3,
          hiddenJobsCount: 1,
          badFitJobsCount: 1,
          whitelistedCompaniesCount: 1,
          blacklistedCompaniesCount: 0,
        },
        topPositiveEvidence: [{ type: 'saved_job', label: 'job-1' }],
        topNegativeEvidence: [{ type: 'bad_fit', label: 'job-2' }],
      },
    });

    expect(fetch).toHaveBeenCalledTimes(1);
    const [url, init] = vi.mocked(fetch).mock.calls[0];
    expect(url).toMatch(/\/v1\/enrichment\/weekly-guidance$/);
    expect(init).toMatchObject({ method: 'POST' });

    const body = JSON.parse(String(init?.body));
    expect(body).toMatchObject({
      profile_id: 'profile-1',
      analytics_summary: {
        feedback: {
          saved_jobs_count: 3,
          hidden_jobs_count: 1,
          bad_fit_jobs_count: 1,
        },
        jobs_by_source: [{ source: 'djinni', count: 12 }],
        jobs_by_lifecycle: { total: 12, active: 10, inactive: 1, reactivated: 1 },
        top_matched_roles: ['backend_developer'],
      },
      behavior_summary: {
        search_run_count: 5,
        top_positive_sources: [{ key: 'djinni', save_count: 3, net_score: 4 }],
      },
      funnel_summary: {
        impression_count: 20,
        conversion_rates: {
          open_rate_from_impressions: 0.4,
          save_rate_from_opens: 0.375,
          application_rate_from_saves: 0.33,
        },
      },
      llm_context: {
        analyzed_profile: {
          primary_role: 'backend_developer',
          skills: ['rust'],
        },
        feedback_summary: {
          whitelisted_companies_count: 1,
        },
        top_positive_evidence: [{ type: 'saved_job', label: 'job-1' }],
      },
    });
    expect(body.llm_context.profile_id).toBeUndefined();
  });
});
