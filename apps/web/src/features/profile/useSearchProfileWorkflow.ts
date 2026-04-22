import { useEffect, useMemo, useRef, useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import {
  buildSearchProfile,
  saveProfileSearchPreferences,
  type SearchProfileBuildResult,
} from '../../api/profiles';
import type { SearchRunResult } from '../../api/jobs';
import { runSearch } from '../../api/jobs';
import { queryKeys } from '../../queryKeys';
import type { getProfile } from '../../api/profiles';
import { toggleValue } from './profile.utils';
import {
  createSearchProfileBuildKey,
  readStoredSearchProfileBuild,
  writeStoredSearchProfileBuild,
} from './searchProfileBuildState';
import { readSearchProfileDraft, writeSearchProfileDraft } from './searchProfileDraft';
import {
  buildPersistedSearchPreferences,
  resolveSearchProfileDraft,
} from './searchProfilePreferences';

export function useSearchProfileWorkflow(
  rawText: string,
  profile: Awaited<ReturnType<typeof getProfile>> | undefined,
) {
  const queryClient = useQueryClient();
  const initialDraft = useMemo(
    () =>
      resolveSearchProfileDraft(profile?.searchPreferences, readSearchProfileDraft(profile?.id)),
    [profile?.id, profile?.searchPreferences],
  );
  const hydrationKey = useMemo(
    () =>
      JSON.stringify({
        profileId: profile?.id ?? null,
        searchPreferences: profile?.searchPreferences ?? null,
      }),
    [profile?.id, profile?.searchPreferences],
  );
  const lastHydratedKey = useRef<string | null>(null);
  const [targetRegions, setTargetRegions] = useState<
    SearchProfileBuildResult['searchProfile']['targetRegions']
  >(initialDraft?.targetRegions ?? []);
  const [workModes, setWorkModes] = useState<
    SearchProfileBuildResult['searchProfile']['workModes']
  >(initialDraft?.workModes ?? []);
  const [preferredRoles, setPreferredRoles] = useState<string[]>(
    initialDraft?.preferredRoles ?? [],
  );
  const [allowedSources, setAllowedSources] = useState<string[]>(
    initialDraft?.allowedSources ?? [],
  );
  const [includeKeywordsInput, setIncludeKeywordsInput] = useState(
    initialDraft?.includeKeywordsInput ?? '',
  );
  const [excludeKeywordsInput, setExcludeKeywordsInput] = useState(
    initialDraft?.excludeKeywordsInput ?? '',
  );
  const [buildResult, setBuildResult] = useState<SearchProfileBuildResult | null>(null);
  const [builtInputKey, setBuiltInputKey] = useState<string | null>(null);
  const [searchResult, setSearchResult] = useState<SearchRunResult | null>(null);
  const [searchError, setSearchError] = useState<string | null>(null);

  const currentPreferences = useMemo(
    () =>
      buildPersistedSearchPreferences({
        targetRegions,
        workModes,
        preferredRoles,
        allowedSources,
        includeKeywordsInput,
        excludeKeywordsInput,
      }),
    [
      allowedSources,
      excludeKeywordsInput,
      includeKeywordsInput,
      preferredRoles,
      targetRegions,
      workModes,
    ],
  );
  const currentInputKey = useMemo(
    () => createSearchProfileBuildKey(profile?.id ?? null, rawText, currentPreferences),
    [currentPreferences, profile?.id, rawText],
  );
  const buildIsCurrent = builtInputKey !== null && builtInputKey === currentInputKey;
  const restoredBuildResult = useMemo(
    () =>
      readStoredSearchProfileBuild({
        profileId: profile?.id ?? null,
        rawText,
        preferences: currentPreferences,
      }),
    [currentPreferences, profile?.id, rawText],
  );
  const activeBuildResult = buildIsCurrent ? buildResult : restoredBuildResult;
  const buildRestoredFromStorage = !buildIsCurrent && Boolean(restoredBuildResult);
  const activeSearchResult = buildIsCurrent ? searchResult : null;
  const activeSearchError = buildIsCurrent ? searchError : null;

  useEffect(() => {
    if (lastHydratedKey.current === hydrationKey) {
      return;
    }

    const draft = resolveSearchProfileDraft(
      profile?.searchPreferences,
      readSearchProfileDraft(profile?.id),
    );
    setTargetRegions(draft?.targetRegions ?? []);
    setWorkModes(draft?.workModes ?? []);
    setPreferredRoles(draft?.preferredRoles ?? []);
    setAllowedSources(draft?.allowedSources ?? []);
    setIncludeKeywordsInput(draft?.includeKeywordsInput ?? '');
    setExcludeKeywordsInput(draft?.excludeKeywordsInput ?? '');
    setBuildResult(null);
    setBuiltInputKey(null);
    setSearchResult(null);
    setSearchError(null);
    lastHydratedKey.current = hydrationKey;
  }, [hydrationKey, profile?.id, profile?.searchPreferences]);

  useEffect(() => {
    writeSearchProfileDraft(
      {
        targetRegions,
        workModes,
        preferredRoles,
        allowedSources,
        includeKeywordsInput,
        excludeKeywordsInput,
      },
      profile?.id,
    );
  }, [
    allowedSources,
    excludeKeywordsInput,
    includeKeywordsInput,
    profile?.id,
    preferredRoles,
    targetRegions,
    workModes,
  ]);

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
    mutationFn: async () => {
      const profileScopeId = profile?.id ?? null;
      const buildInputKey = createSearchProfileBuildKey(
        profileScopeId,
        rawText,
        currentPreferences,
      );
      let persistedProfile = null;
      let persistenceError: string | null = null;

      if (profileScopeId) {
        try {
          persistedProfile = await saveProfileSearchPreferences(profileScopeId, currentPreferences);
        } catch (error) {
          persistenceError = error instanceof Error ? error.message : 'Failed to save preferences';
        }
      }

      const result = await buildSearchProfile({
        rawText,
        preferences: currentPreferences,
      });

      return {
        persistedProfile,
        persistenceError,
        result,
        rawText,
        preferences: currentPreferences,
        profileScopeId,
        buildInputKey,
      };
    },
    onSuccess: (result) => {
      if (result.persistedProfile) {
        queryClient.setQueryData(queryKeys.profile.root(), result.persistedProfile);
      }
      setBuildResult(result.result);
      setBuiltInputKey(result.buildInputKey);
      setSearchResult(null);
      setSearchError(null);
      writeStoredSearchProfileBuild({
        profileId: result.profileScopeId,
        rawText: result.rawText,
        preferences: result.preferences,
        result: result.result,
      });
      if (result.persistenceError) {
        toast.error(result.persistenceError);
      }
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
    buildResult: activeBuildResult,
    buildIsCurrent: Boolean(activeBuildResult),
    buildRestoredFromStorage,
    searchResult: activeSearchResult,
    searchError: activeSearchError,
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
    runCurrentSearch: () =>
      activeBuildResult && runMutation.mutate(activeBuildResult.searchProfile),
  };
}
