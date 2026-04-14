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
