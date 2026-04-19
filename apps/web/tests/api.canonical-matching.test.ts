import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { analyzeFit, rerankJobs, runSearch } from '../src/api';

const ENGINE_API_URL = import.meta.env.VITE_ENGINE_API_URL?.trim() || 'http://localhost:8080';

function jsonResponse(body: unknown) {
  return Promise.resolve({
    ok: true,
    status: 200,
    json: async () => body,
  });
}

function buildEngineJob() {
  return {
    id: 'job-1',
    title: 'Senior Front-end React Developer',
    company_name: 'Acme',
    description_text:
      'Build reusable frontend systems with React, TypeScript, and strong accessibility practices.',
    location: 'Kyiv',
    remote_type: 'remote',
    seniority: 'senior',
    salary_min: null,
    salary_max: null,
    salary_currency: null,
    posted_at: '2026-04-01T00:00:00Z',
    first_seen_at: '2026-04-01T00:00:00Z',
    last_seen_at: '2026-04-10T00:00:00Z',
    is_active: true,
    inactivated_at: null,
    reactivated_at: null,
    lifecycle_stage: 'active' as const,
    primary_variant: {
      source: 'djinni',
      source_job_id: 'dj-1',
      source_url: 'https://djinni.co/jobs/1',
      fetched_at: '2026-04-10T00:00:00Z',
      last_seen_at: '2026-04-10T00:00:00Z',
      is_active: true,
      inactivated_at: null,
    },
    presentation: {
      title: 'Senior Front-end React Developer',
      company: 'Acme',
      summary: 'Frontend platform work with React and TypeScript.',
      summary_quality: 'strong',
      summary_fallback: false,
      description_quality: 'strong',
      location_label: 'Kyiv',
      work_mode_label: 'Remote',
      source_label: 'Djinni',
      outbound_url: 'https://djinni.co/jobs/1',
      salary_label: null,
      freshness_label: '9d ago',
      badges: ['Remote', 'Senior'],
    },
    feedback: {
      saved: false,
      hidden: false,
      bad_fit: false,
      company_status: null,
    },
  };
}

describe('canonical matching api', () => {
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

  it('rerankJobs uses engine-api bulk match endpoint and maps diagnostics', async () => {
    vi.mocked(fetch).mockImplementationOnce(
      () =>
        jsonResponse({
          profile_id: 'profile-1',
          results: [
            {
              job_id: 'job-b',
              score: 61,
              matched_roles: ['frontend_developer'],
              matched_skills: ['react'],
              matched_keywords: ['typescript'],
              missing_signals: ['design system'],
              source_match: true,
              work_mode_match: true,
              region_match: true,
              description_quality: 'mixed',
              positive_reasons: ['Matched 1 target role'],
              negative_reasons: ['Missing design system'],
              reasons: ['Matched 1 target role', 'Missing design system'],
            },
            {
              job_id: 'job-a',
              score: 92,
              matched_roles: ['frontend_developer'],
              matched_skills: ['react', 'typescript'],
              matched_keywords: ['design system', 'react'],
              missing_signals: [],
              source_match: true,
              work_mode_match: true,
              region_match: true,
              description_quality: 'strong',
              positive_reasons: ['Matched 2 strong frontend skills'],
              negative_reasons: [],
              reasons: ['Matched 2 strong frontend skills'],
            },
          ],
        }) as ReturnType<typeof fetch>,
    );

    const result = await rerankJobs('profile-1', ['job-b', 'job-a', 'job-b']);

    expect(fetch).toHaveBeenCalledWith(
      `${ENGINE_API_URL}/api/v1/profiles/profile-1/jobs/match`,
      expect.objectContaining({ method: 'POST' }),
    );
    expect(result.map((item) => item.jobId)).toEqual(['job-a', 'job-b']);
    expect(result[0]).toMatchObject({
      jobId: 'job-a',
      score: 92,
      matchedTerms: ['frontend_developer', 'react', 'typescript', 'design system'],
      positiveReasons: ['Matched 2 strong frontend skills'],
      negativeReasons: [],
      descriptionQuality: 'strong',
    });
    expect(result[1].missingSignals).toEqual(['design system']);
  });

  it('analyzeFit uses engine-api job match endpoint and prefers positive reasons as evidence', async () => {
    vi.mocked(fetch).mockImplementationOnce(
      () =>
        jsonResponse({
          job_id: 'job-1',
          score: 88,
          matched_roles: ['frontend_developer'],
          matched_skills: ['react', 'typescript'],
          matched_keywords: ['design system'],
          missing_signals: ['storybook'],
          source_match: true,
          work_mode_match: true,
          region_match: true,
          description_quality: 'strong',
          positive_reasons: ['Matched 2 target frontend skills'],
          negative_reasons: ['Missing storybook'],
          reasons: ['Matched 2 target frontend skills', 'Missing storybook'],
        }) as ReturnType<typeof fetch>,
    );

    const result = await analyzeFit('profile-1', 'job-1');

    expect(fetch).toHaveBeenCalledWith(
      `${ENGINE_API_URL}/api/v1/profiles/profile-1/jobs/job-1/match`,
      undefined,
    );
    expect(result).toMatchObject({
      profileId: 'profile-1',
      jobId: 'job-1',
      matchedRoles: ['frontend_developer'],
      matchedSkills: ['react', 'typescript'],
      matchedKeywords: ['design system'],
      missingTerms: ['storybook'],
      descriptionQuality: 'strong',
      positiveReasons: ['Matched 2 target frontend skills'],
      negativeReasons: ['Missing storybook'],
      evidence: ['Matched 2 target frontend skills'],
    });
  });

  it('runSearch maps canonical telemetry and fit diagnostics', async () => {
    vi.mocked(fetch).mockImplementationOnce(
      () =>
        jsonResponse({
          results: [
            {
              job: buildEngineJob(),
              fit: {
                job_id: 'job-1',
                score: 84,
                matched_roles: ['frontend_developer'],
                matched_skills: ['react'],
                matched_keywords: ['typescript'],
                missing_signals: ['storybook'],
                source_match: true,
                work_mode_match: true,
                region_match: true,
                description_quality: 'strong',
                positive_reasons: ['Matched 1 target role'],
                negative_reasons: ['Missing storybook'],
                reasons: ['Matched 1 target role', 'Missing storybook'],
              },
            },
          ],
          meta: {
            total_candidates: 14,
            filtered_out_by_source: 2,
            filtered_out_hidden: 1,
            filtered_out_company_blacklist: 1,
            scored_jobs: 10,
            returned_jobs: 1,
            low_evidence_jobs: 3,
            weak_description_jobs: 2,
            role_mismatch_jobs: 1,
            seniority_mismatch_jobs: 1,
            source_mismatch_jobs: 2,
            top_missing_signals: ['storybook', 'design system'],
          },
        }) as ReturnType<typeof fetch>,
    );

    const result = await runSearch({
      profileId: 'profile-1',
      searchProfile: {
        primaryRole: 'frontend_developer',
        targetRoles: ['frontend_developer'],
        roleCandidates: [],
        seniority: 'senior',
        targetRegions: ['ua'],
        workModes: ['remote'],
        allowedSources: ['djinni'],
        profileSkills: ['react'],
        profileKeywords: ['typescript'],
        searchTerms: ['react', 'typescript'],
        excludeTerms: [],
      },
      limit: 10,
    });

    expect(fetch).toHaveBeenCalledWith(
      `${ENGINE_API_URL}/api/v1/search/run`,
      expect.objectContaining({ method: 'POST' }),
    );
    expect(result.meta).toEqual({
      totalCandidates: 14,
      filteredOutBySource: 2,
      filteredOutHidden: 1,
      filteredOutCompanyBlacklist: 1,
      scoredJobs: 10,
      returnedJobs: 1,
      lowEvidenceJobs: 3,
      weakDescriptionJobs: 2,
      roleMismatchJobs: 1,
      seniorityMismatchJobs: 1,
      sourceMismatchJobs: 2,
      topMissingSignals: ['storybook', 'design system'],
    });
    expect(result.results[0].fit).toMatchObject({
      jobId: 'job-1',
      positiveReasons: ['Matched 1 target role'],
      negativeReasons: ['Missing storybook'],
      descriptionQuality: 'strong',
    });
    expect(result.results[0].job.presentation?.summaryQuality).toBe('strong');
  });
});
