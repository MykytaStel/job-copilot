import type { SearchTargetRegion, SearchWorkMode } from '../../api';

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

export const PROFILE_LANGUAGE_OPTIONS = [
  { id: 'Ukrainian', label: 'Ukrainian' },
  { id: 'English', label: 'English' },
  { id: 'German', label: 'German' },
  { id: 'Polish', label: 'Polish' },
] as const;

export const PROFILE_SALARY_CURRENCY_OPTIONS = [
  { id: 'USD', label: 'USD' },
  { id: 'EUR', label: 'EUR' },
  { id: 'UAH', label: 'UAH' },
] as const;
