export type DensityMode = 'compact' | 'normal' | 'comfortable';
export type SortMode = 'relevance' | 'date' | 'salary';

const DENSITY_KEY = 'jc_display_density';
const SORT_KEY = 'jc_display_sort';

const DEFAULT_DENSITY: DensityMode = 'normal';
const DEFAULT_SORT: SortMode = 'date';

export function readDensity(): DensityMode {
  try {
    const raw = window.localStorage.getItem(DENSITY_KEY);
    if (raw === 'compact' || raw === 'normal' || raw === 'comfortable') return raw;
    return DEFAULT_DENSITY;
  } catch {
    return DEFAULT_DENSITY;
  }
}

export function writeDensity(value: DensityMode): void {
  window.localStorage.setItem(DENSITY_KEY, value);
}

export function readSortMode(): SortMode {
  try {
    const raw = window.localStorage.getItem(SORT_KEY);
    if (raw === 'relevance' || raw === 'date' || raw === 'salary') return raw;
    return DEFAULT_SORT;
  } catch {
    return DEFAULT_SORT;
  }
}

export function writeSortMode(value: SortMode): void {
  window.localStorage.setItem(SORT_KEY, value);
}

export const DENSITY_GAP: Record<DensityMode, string> = {
  compact: 'space-y-1',
  normal: 'space-y-3',
  comfortable: 'space-y-5',
};

const DASHBOARD_LIFECYCLE_KEY = 'jc_dashboard_lifecycle';
const DASHBOARD_SOURCE_KEY = 'jc_dashboard_source';

export function readPersistedLifecycle(): string | null {
  try {
    return window.localStorage.getItem(DASHBOARD_LIFECYCLE_KEY);
  } catch {
    return null;
  }
}

export function writePersistedLifecycle(value: string | null): void {
  try {
    if (value) {
      window.localStorage.setItem(DASHBOARD_LIFECYCLE_KEY, value);
    } else {
      window.localStorage.removeItem(DASHBOARD_LIFECYCLE_KEY);
    }
  } catch {
		// Ignore write errors
	}
}

export function readPersistedSource(): string | null {
  try {
    return window.localStorage.getItem(DASHBOARD_SOURCE_KEY);
  } catch {
    return null;
  }
}

export function writePersistedSource(value: string | null): void {
  try {
    if (value) {
      window.localStorage.setItem(DASHBOARD_SOURCE_KEY, value);
    } else {
      window.localStorage.removeItem(DASHBOARD_SOURCE_KEY);
    }
  } catch {
		// Ignore write errors
	}
}
