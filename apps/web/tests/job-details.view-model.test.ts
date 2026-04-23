import { describe, expect, it } from 'vitest';

import { buildJobDetailsViewModel } from '../src/pages/job-details/jobDetails.view-model';

function makeState(
  job: Record<string, unknown> | null,
  fit: Record<string, unknown> | undefined = undefined,
) {
  return { job, fit } as Parameters<typeof buildJobDetailsViewModel>[0];
}

describe('buildJobDetailsViewModel', () => {
  describe('salary formatting', () => {
    it('formats a USD range with both min and max', () => {
      const { salary } = buildJobDetailsViewModel(
        makeState({ salaryMin: 4000, salaryMax: 6000, salaryCurrency: 'USD' }),
      );
      expect(salary).toBe('$4,000 – $6,000');
    });

    it('formats EUR with currency symbol', () => {
      const { salary } = buildJobDetailsViewModel(
        makeState({ salaryMin: 3000, salaryMax: 5000, salaryCurrency: 'EUR' }),
      );
      expect(salary).toBe('€3,000 – €5,000');
    });

    it('returns null when both min and max are missing', () => {
      const { salary } = buildJobDetailsViewModel(makeState({}));
      expect(salary).toBeNull();
    });

    it('formats min-only salary', () => {
      const { salary } = buildJobDetailsViewModel(
        makeState({ salaryMin: 3000, salaryCurrency: 'USD' }),
      );
      expect(salary).toBe('від $3,000');
    });

    it('formats max-only salary', () => {
      const { salary } = buildJobDetailsViewModel(
        makeState({ salaryMax: 5000, salaryCurrency: 'USD' }),
      );
      expect(salary).toBe('до $5,000');
    });
  });

  describe('sourceLabel', () => {
    it('formats underscore-separated source as dot-separated', () => {
      const { sourceLabel } = buildJobDetailsViewModel(
        makeState({ primaryVariant: { source: 'work_ua' } }),
      );
      expect(sourceLabel).toBe('work.ua');
    });

    it('returns "Unknown source" when no primary variant', () => {
      const { sourceLabel } = buildJobDetailsViewModel(makeState({}));
      expect(sourceLabel).toBe('Unknown source');
    });

    it('returns source as-is for single-word sources', () => {
      const { sourceLabel } = buildJobDetailsViewModel(
        makeState({ primaryVariant: { source: 'djinni' } }),
      );
      expect(sourceLabel).toBe('djinni');
    });
  });

  describe('descriptionQuality', () => {
    it('prefers fit descriptionQuality over presentation quality', () => {
      const { descriptionQuality } = buildJobDetailsViewModel(
        makeState(
          { presentation: { descriptionQuality: 'weak', badges: [] } },
          { descriptionQuality: 'strong' },
        ),
      );
      expect(descriptionQuality).toBe('strong');
    });

    it('falls back to presentation quality when no fit', () => {
      const { descriptionQuality } = buildJobDetailsViewModel(
        makeState({ presentation: { descriptionQuality: 'mixed', badges: [] } }),
      );
      expect(descriptionQuality).toBe('mixed');
    });

    it('is undefined when neither fit nor presentation quality exists', () => {
      const { descriptionQuality } = buildJobDetailsViewModel(makeState({}));
      expect(descriptionQuality).toBeUndefined();
    });
  });

  describe('skillBadges', () => {
    it('merges presentation badges and fit matchedTerms', () => {
      const { skillBadges } = buildJobDetailsViewModel(
        makeState(
          { presentation: { badges: ['Remote', 'Senior'], descriptionQuality: 'strong' } },
          { matchedTerms: ['Rust', 'Postgres'] },
        ),
      );
      expect(skillBadges).toEqual(['Remote', 'Senior', 'Rust', 'Postgres']);
    });

    it('caps skillBadges at 10', () => {
      const { skillBadges } = buildJobDetailsViewModel(
        makeState(
          { presentation: { badges: ['a', 'b', 'c', 'd', 'e', 'f'], descriptionQuality: 'strong' } },
          { matchedTerms: ['g', 'h', 'i', 'j', 'k'] },
        ),
      );
      expect(skillBadges).toHaveLength(10);
    });

    it('returns empty array when neither presentation nor fit has badges', () => {
      const { skillBadges } = buildJobDetailsViewModel(makeState({}));
      expect(skillBadges).toEqual([]);
    });
  });

  describe('lifecycleStatus', () => {
    it('uses lifecycleStage from job when present', () => {
      const { lifecycleStatus } = buildJobDetailsViewModel(
        makeState({ lifecycleStage: 'reactivated' }),
      );
      expect(lifecycleStatus).toBe('reactivated');
    });

    it('returns "inactive" when isActive is false and no lifecycleStage', () => {
      const { lifecycleStatus } = buildJobDetailsViewModel(makeState({ isActive: false }));
      expect(lifecycleStatus).toBe('inactive');
    });

    it('returns "active" when job is active and no lifecycleStage', () => {
      const { lifecycleStatus } = buildJobDetailsViewModel(makeState({ isActive: true }));
      expect(lifecycleStatus).toBe('active');
    });

    it('returns "active" when job is null', () => {
      const { lifecycleStatus } = buildJobDetailsViewModel(makeState(null));
      expect(lifecycleStatus).toBe('active');
    });
  });

  describe('topBadges', () => {
    it('includes sourceLabel, seniority, and remoteType when all present', () => {
      const { topBadges } = buildJobDetailsViewModel(
        makeState({
          primaryVariant: { source: 'djinni' },
          seniority: 'Senior',
          remoteType: 'remote',
        }),
      );
      expect(topBadges).toEqual(['djinni', 'Senior', 'remote']);
    });

    it('excludes null seniority and remoteType', () => {
      const { topBadges } = buildJobDetailsViewModel(
        makeState({ primaryVariant: { source: 'djinni' } }),
      );
      expect(topBadges).toEqual(['djinni']);
    });
  });
});
