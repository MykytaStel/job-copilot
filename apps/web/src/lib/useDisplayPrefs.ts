import { useCallback, useEffect, useState } from 'react';

import {
  DISPLAY_PREFS_EVENT,
  readDensity,
  readSortMode,
  writeDensity,
  writeSortMode,
  type DensityMode,
  type SortMode,
} from './displayPrefs';

export function useDisplayPrefs() {
  const [density, setDensityState] = useState<DensityMode>(() => readDensity());
  const [sortMode, setSortModeState] = useState<SortMode>(() => readSortMode());

  const refresh = useCallback(() => {
    setDensityState(readDensity());
    setSortModeState(readSortMode());
  }, []);

  useEffect(() => {
    window.addEventListener(DISPLAY_PREFS_EVENT, refresh);
    window.addEventListener('storage', refresh);
    return () => {
      window.removeEventListener(DISPLAY_PREFS_EVENT, refresh);
      window.removeEventListener('storage', refresh);
    };
  }, [refresh]);

  const setDensity = useCallback((value: DensityMode) => {
    writeDensity(value);
    setDensityState(value);
  }, []);

  const setSortMode = useCallback((value: SortMode) => {
    writeSortMode(value);
    setSortModeState(value);
  }, []);

  return {
    density,
    sortMode,
    setDensity,
    setSortMode,
  };
}
