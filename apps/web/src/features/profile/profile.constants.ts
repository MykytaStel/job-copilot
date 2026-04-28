import type { WorkModePreference } from '@job-copilot/shared/profiles';
import type { SearchTargetRegion, SearchWorkMode } from '../../api/profiles';

export const TARGET_REGION_OPTIONS: Array<{ id: SearchTargetRegion; label: string }> = [
  { id: 'ua', label: 'Ukraine' },
  { id: 'eu', label: 'EU' },
  { id: 'eu_remote', label: 'EU remote' },
  { id: 'poland', label: 'Poland' },
  { id: 'germany', label: 'Germany' },
  { id: 'uk', label: 'UK' },
  { id: 'us', label: 'US' },
];

export const WORK_MODE_OPTIONS: Array<{ id: SearchWorkMode; label: string }> = [
  { id: 'remote', label: 'Remote' },
  { id: 'hybrid', label: 'Hybrid' },
  { id: 'onsite', label: 'Onsite' },
];

export const PROFILE_LANGUAGE_LEVEL_OPTIONS = [
  { id: 'A1', label: 'A1' },
  { id: 'A2', label: 'A2' },
  { id: 'B1', label: 'B1' },
  { id: 'B2', label: 'B2' },
  { id: 'C1', label: 'C1' },
  { id: 'C2', label: 'C2' },
  { id: 'Native', label: 'Native' },
] as const;

export const PROFILE_SALARY_CURRENCY_OPTIONS = [
  { id: 'USD', label: 'USD' },
  { id: 'EUR', label: 'EUR' },
  { id: 'UAH', label: 'UAH' },
] as const;

export const PROFILE_LOCATION_QUICK_ADD_OPTIONS = ['Remote'] as const;

export const PROFILE_WORK_MODE_OPTIONS: Array<{ id: WorkModePreference; label: string }> = [
  { id: 'remote_only', label: 'Remote only' },
  { id: 'hybrid', label: 'Hybrid' },
  { id: 'onsite', label: 'Onsite' },
  { id: 'any', label: 'Any' },
];
