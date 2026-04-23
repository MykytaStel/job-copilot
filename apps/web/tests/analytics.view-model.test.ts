import { describe, expect, it } from 'vitest';

import { buildAnalyticsViewModel } from '../src/pages/analytics/analytics.view-model';
import type { AnalyticsSummary, BehaviorSummary, FunnelSummary } from '../src/api/analytics';

function makeSummary(overrides: Partial<AnalyticsSummary> = {}): AnalyticsSummary {
  return {
    profileId: 'profile-1',
    feedback: { savedJobsCount: 0, badFitJobsCount: 0, hiddenJobsCount: 0, ...overrides.feedback },
    jobsBySource: [],
    jobsByLifecycle: { total: 0, active: 0, inactive: 0, reactivated: 0, ...overrides.jobsByLifecycle },
    topMatchedRoles: [],
    topMatchedSkills: [],
    topMatchedKeywords: [],
    searchQuality: {
      lowEvidenceJobs: 0,
      weakDescriptionJobs: 0,
      roleMismatchJobs: 0,
      seniorityMismatchJobs: 0,
      sourceMismatchJobs: 0,
      topMissingSignals: [],
      ...overrides.searchQuality,
    },
    ...overrides,
  } as AnalyticsSummary;
}

function makeFunnel(overrides: Partial<FunnelSummary> = {}): FunnelSummary {
  return {
    profileId: 'profile-1',
    impressionCount: 0,
    openCount: 0,
    saveCount: 0,
    hideCount: 0,
    badFitCount: 0,
    applicationCreatedCount: 0,
    fitExplanationRequestedCount: 0,
    applicationCoachRequestedCount: 0,
    coverLetterDraftRequestedCount: 0,
    interviewPrepRequestedCount: 0,
    conversionRates: {
      openRateFromImpressions: 0,
      saveRateFromOpens: 0,
      applicationRateFromSaves: 0,
    },
    impressionsBySource: [],
    opensBySource: [],
    savesBySource: [],
    applicationsBySource: [],
    ...overrides,
  } as FunnelSummary;
}

function makeBehavior(overrides: Partial<BehaviorSummary> = {}): BehaviorSummary {
  return {
    profileId: 'profile-1',
    searchRunCount: 0,
    topPositiveSources: [],
    topNegativeSources: [],
    topPositiveRoleFamilies: [],
    topNegativeRoleFamilies: [],
    sourceSignalCounts: [],
    roleFamilySignalCounts: [],
    ...overrides,
  } as BehaviorSummary;
}

describe('buildAnalyticsViewModel', () => {
  describe('heroMetrics', () => {
    it('shows indexed jobs from summary.jobsByLifecycle.total', () => {
      const { heroMetrics } = buildAnalyticsViewModel({
        summary: makeSummary({ jobsByLifecycle: { total: 42, active: 40, inactive: 2, reactivated: 0 } }),
        behavior: undefined,
        funnel: undefined,
      });
      const indexed = heroMetrics.find((m) => m.label === 'Indexed jobs');
      expect(indexed?.value).toBe(42);
    });

    it('shows search runs from behavior.searchRunCount', () => {
      const { heroMetrics } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: makeBehavior({ searchRunCount: 15 }),
        funnel: undefined,
      });
      const runs = heroMetrics.find((m) => m.label === 'Search runs');
      expect(runs?.value).toBe(15);
    });

    it('defaults search runs to 0 when behavior is undefined', () => {
      const { heroMetrics } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: undefined,
      });
      expect(heroMetrics.find((m) => m.label === 'Search runs')?.value).toBe(0);
    });

    it('shows applications from funnel.applicationCreatedCount', () => {
      const { heroMetrics } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: makeFunnel({ applicationCreatedCount: 7 }),
      });
      expect(heroMetrics.find((m) => m.label === 'Applications')?.value).toBe(7);
    });

    it('defaults applications to 0 when funnel is undefined', () => {
      const { heroMetrics } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: undefined,
      });
      expect(heroMetrics.find((m) => m.label === 'Applications')?.value).toBe(0);
    });
  });

  describe('feedbackCards', () => {
    it('shows saved jobs count from summary.feedback', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary({ feedback: { savedJobsCount: 12, badFitJobsCount: 3, hiddenJobsCount: 5 } }),
        behavior: undefined,
        funnel: undefined,
      });
      expect(feedbackCards.find((c) => c.title === 'Saved Jobs')?.value).toBe(12);
    });

    it('shows bad fit count from summary.feedback', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary({ feedback: { savedJobsCount: 0, badFitJobsCount: 8, hiddenJobsCount: 0 } }),
        behavior: undefined,
        funnel: undefined,
      });
      expect(feedbackCards.find((c) => c.title === 'Bad Fit')?.value).toBe(8);
    });

    it('formats open rate as percentage rounded to nearest integer', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: makeFunnel({
          conversionRates: { openRateFromImpressions: 0.333, saveRateFromOpens: 0, applicationRateFromSaves: 0 },
        }),
      });
      expect(feedbackCards.find((c) => c.title === 'Open Rate')?.value).toBe('33%');
    });

    it('formats apply rate as percentage rounded to nearest integer', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: makeFunnel({
          conversionRates: { openRateFromImpressions: 0, saveRateFromOpens: 0, applicationRateFromSaves: 0.25 },
        }),
      });
      expect(feedbackCards.find((c) => c.title === 'Apply Rate')?.value).toBe('25%');
    });

    it('shows "0%" for rates when funnel is undefined', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: undefined,
      });
      expect(feedbackCards.find((c) => c.title === 'Open Rate')?.value).toBe('0%');
      expect(feedbackCards.find((c) => c.title === 'Apply Rate')?.value).toBe('0%');
    });

    it('rounds 100% conversion correctly', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: makeFunnel({
          conversionRates: { openRateFromImpressions: 1.0, saveRateFromOpens: 1.0, applicationRateFromSaves: 1.0 },
        }),
      });
      expect(feedbackCards.find((c) => c.title === 'Open Rate')?.value).toBe('100%');
      expect(feedbackCards.find((c) => c.title === 'Apply Rate')?.value).toBe('100%');
    });
  });

  describe('structure', () => {
    it('heroMetrics contains exactly 3 items', () => {
      const { heroMetrics } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: undefined,
      });
      expect(heroMetrics).toHaveLength(3);
    });

    it('feedbackCards contains exactly 4 items', () => {
      const { feedbackCards } = buildAnalyticsViewModel({
        summary: makeSummary(),
        behavior: undefined,
        funnel: undefined,
      });
      expect(feedbackCards).toHaveLength(4);
    });
  });
});
