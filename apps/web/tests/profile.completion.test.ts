import { describe, expect, it } from 'vitest';

import { buildProfileCompletionState } from '../src/features/profile/profileCompletion';

describe('buildProfileCompletionState', () => {
  it('counts completed weighted checkpoints and reports missing labels', () => {
    const state = buildProfileCompletionState({
      name: 'Jane Doe',
      email: 'jane@example.com',
      rawText: 'Senior frontend engineer',
      skills: [],
      salaryMin: '4000',
      salaryMax: '',
      salaryCurrency: 'USD',
      languages: [],
      preferredLocations: [],
      targetRegions: [],
      workModes: [],
      preferredRoles: [],
    });

    expect(state.completedWeight).toBe(50);
    expect(state.totalWeight).toBe(120);
    expect(state.percent).toBe(42);
    expect(state.missingLabels).toEqual([
      'At least 3 skills',
      'Work mode preference',
      'Preferred roles',
      'Location preference',
      'Language preference',
    ]);
  });

  it('marks profile complete when all weighted checkpoints are complete', () => {
    const state = buildProfileCompletionState({
      name: 'Jane Doe',
      email: 'jane@example.com',
      rawText: 'Senior frontend engineer',
      skills: ['react', 'typescript', 'node.js'],
      salaryMin: '4000',
      salaryMax: '5500',
      salaryCurrency: 'USD',
      languages: [{ language: 'english', proficiency: 'b2' }],
      preferredLocations: ['Kyiv'],
      targetRegions: ['ua'],
      workModes: ['remote'],
      preferredRoles: ['frontend-engineer'],
    });

    expect(state.percent).toBe(100);
    expect(state.completedWeight).toBe(120);
    expect(state.totalWeight).toBe(120);
    expect(state.missingLabels).toEqual([]);
  });

  it('marks salary incomplete when amount is present but currency is missing', () => {
    const state = buildProfileCompletionState({
      name: 'Jane Doe',
      email: 'jane@example.com',
      rawText: 'Senior frontend engineer',
      skills: ['react', 'typescript', 'node.js'],
      salaryMin: '4000',
      salaryMax: '',
      salaryCurrency: '',
      languages: [{ language: 'english', proficiency: 'b2' }],
      preferredLocations: ['Kyiv'],
      targetRegions: [],
      workModes: ['remote'],
      preferredRoles: [],
    });

    expect(state.missingLabels).toContain('Salary expectation');
  });
});