import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { buildSearchProfile, type SearchProfileBuildResult } from '../../api/profiles';
import type { SearchRunResult } from '../../api/jobs';
import { runSearch } from '../../api/jobs';
import { parseKeywordInput, toggleValue } from './profile.utils';

export function useSearchProfileWorkflow(rawText: string) {
  const [targetRegions, setTargetRegions] = useState<
    SearchProfileBuildResult['searchProfile']['targetRegions']
  >([]);
  const [workModes, setWorkModes] = useState<
    SearchProfileBuildResult['searchProfile']['workModes']
  >([]);
  const [preferredRoles, setPreferredRoles] = useState<string[]>([]);
  const [allowedSources, setAllowedSources] = useState<string[]>([]);
  const [includeKeywordsInput, setIncludeKeywordsInput] = useState('');
  const [excludeKeywordsInput, setExcludeKeywordsInput] = useState('');
  const [buildResult, setBuildResult] = useState<SearchProfileBuildResult | null>(null);
  const [searchResult, setSearchResult] = useState<SearchRunResult | null>(null);
  const [searchError, setSearchError] = useState<string | null>(null);

  const runMutation = useMutation({
    mutationFn: (searchProfile: SearchProfileBuildResult['searchProfile']) =>
      runSearch({ searchProfile, limit: 20 }),
    onMutate: () => {
      setSearchError(null);
    },
    onSuccess: (result) => {
      setSearchResult(result);
      toast.success(
        result.results.length > 0
          ? `Ranked ${result.results.length} job${result.results.length === 1 ? '' : 's'}`
          : 'Search completed with no matches',
      );
    },
    onError: (error: unknown) => {
      const message = error instanceof Error ? error.message : 'Search failed';
      setSearchError(message);
      toast.error(message);
    },
  });

  const buildMutation = useMutation({
    mutationFn: () =>
      buildSearchProfile({
        rawText,
        preferences: {
          targetRegions,
          workModes,
          preferredRoles,
          allowedSources,
          includeKeywords: parseKeywordInput(includeKeywordsInput),
          excludeKeywords: parseKeywordInput(excludeKeywordsInput),
        },
      }),
    onSuccess: (result) => {
      setBuildResult(result);
      setSearchResult(null);
      setSearchError(null);
      toast.success('Search profile built');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Error'),
  });

  return {
    targetRegions,
    workModes,
    preferredRoles,
    allowedSources,
    includeKeywordsInput,
    excludeKeywordsInput,
    buildResult,
    searchResult,
    searchError,
    buildMutation,
    runMutation,
    setIncludeKeywordsInput,
    setExcludeKeywordsInput,
    toggleTargetRegion: (
      value: SearchProfileBuildResult['searchProfile']['targetRegions'][number],
    ) => setTargetRegions((current) => toggleValue(current, value)),
    toggleWorkMode: (value: SearchProfileBuildResult['searchProfile']['workModes'][number]) =>
      setWorkModes((current) => toggleValue(current, value)),
    togglePreferredRole: (value: string) =>
      setPreferredRoles((current) => toggleValue(current, value)),
    toggleAllowedSource: (value: string) =>
      setAllowedSources((current) => toggleValue(current, value)),
    buildCurrentSearchProfile: () => buildMutation.mutate(),
    runCurrentSearch: () => buildResult && runMutation.mutate(buildResult.searchProfile),
  };
}
