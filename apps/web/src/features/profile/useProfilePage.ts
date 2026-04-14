import { useEffect, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';

import {
  analyzeStoredProfile,
  buildSearchProfile,
  getLlmContext,
  getProfile,
  getRoles,
  getSources,
  getStoredProfileRawText,
  runSearch,
  saveProfile,
  type SearchProfileBuildResult,
  type SearchRunResult,
} from '../../api';
import { queryKeys } from '../../queryKeys';
import { extractPdfText, parseKeywordInput, toggleValue } from './profile.utils';

export function useProfilePage() {
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [location, setLocation] = useState('');
  const [rawText, setRawText] = useState('');
  const [targetRegions, setTargetRegions] = useState<SearchProfileBuildResult['searchProfile']['targetRegions']>([]);
  const [workModes, setWorkModes] = useState<SearchProfileBuildResult['searchProfile']['workModes']>([]);
  const [preferredRoles, setPreferredRoles] = useState<string[]>([]);
  const [allowedSources, setAllowedSources] = useState<string[]>([]);
  const [includeKeywordsInput, setIncludeKeywordsInput] = useState('');
  const [excludeKeywordsInput, setExcludeKeywordsInput] = useState('');
  const [buildResult, setBuildResult] = useState<SearchProfileBuildResult | null>(null);
  const [searchResult, setSearchResult] = useState<SearchRunResult | null>(null);
  const [searchError, setSearchError] = useState<string | null>(null);

  const profileQuery = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });
  const rolesQuery = useQuery({
    queryKey: queryKeys.roles.all(),
    queryFn: getRoles,
  });
  const sourcesQuery = useQuery({
    queryKey: queryKeys.sources.all(),
    queryFn: getSources,
  });
  const llmContextQuery = useQuery({
    queryKey: queryKeys.analytics.llmContext(profileQuery.data?.id ?? ''),
    queryFn: () => getLlmContext(profileQuery.data!.id),
    enabled: !!profileQuery.data?.id,
  });

  useEffect(() => {
    if (!profileQuery.data) return;

    setName(profileQuery.data.name);
    setEmail(profileQuery.data.email);
    setLocation(profileQuery.data.location ?? '');

    void getStoredProfileRawText()
      .then(setRawText)
      .catch(() => {});
  }, [profileQuery.data]);

  const saveMutation = useMutation({
    mutationFn: (vars: {
      name: string;
      email: string;
      location?: string;
      rawText: string;
    }) =>
      saveProfile({
        name: vars.name,
        email: vars.email,
        location: vars.location,
        rawText: vars.rawText,
        summary: undefined,
        skills: [],
      }),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      toast.success('Profile saved');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Error'),
  });

  const analyzeMutation = useMutation({
    mutationFn: analyzeStoredProfile,
    onSuccess: (analysis) => {
      queryClient.setQueryData(
        queryKeys.profile.root(),
        (current: Awaited<ReturnType<typeof getProfile>>) =>
          current
            ? {
                ...current,
                summary: analysis.summary,
                skills: analysis.skills,
              }
            : current,
      );
      toast.success('Profile analyzed');
    },
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Error'),
  });

  const runMutation = useMutation({
    mutationFn: (searchProfile: SearchProfileBuildResult['searchProfile']) =>
      runSearch({
        searchProfile,
        limit: 20,
      }),
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
    onError: (error: unknown) =>
      toast.error(error instanceof Error ? error.message : 'Error'),
  });

  async function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    event.target.value = '';

    if (file.type === 'application/pdf') {
      const loadingToast = toast.loading('Читаємо PDF…');

      try {
        const text = await extractPdfText(file);
        if (text.trim()) {
          setRawText(text);
          toast.success(`PDF завантажено: ${file.name}`, { id: loadingToast });
        } else {
          toast.error('PDF порожній або захищений — спробуйте .txt', { id: loadingToast });
        }
      } catch {
        toast.error('Не вдалося прочитати PDF', { id: loadingToast });
      }

      return;
    }

    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const text = loadEvent.target?.result;
      if (typeof text === 'string' && text.trim()) {
        setRawText(text);
        toast.success(`Файл завантажено: ${file.name}`);
      } else {
        toast.error('Файл порожній або не вдалося прочитати');
      }
    };
    reader.onerror = () => toast.error('Помилка читання файлу');
    reader.readAsText(file, 'UTF-8');
  }

  function saveCurrentProfile() {
    saveMutation.mutate({
      name,
      email,
      location: location || undefined,
      rawText,
    });
  }

  return {
    fileInputRef,
    profile: profileQuery.data,
    roles: rolesQuery.data ?? [],
    sources: sourcesQuery.data ?? [],
    rolesError: rolesQuery.error,
    sourcesError: sourcesQuery.error,
    llmContext: llmContextQuery.data ?? null,
    llmContextError: llmContextQuery.error,
    llmContextLoading: llmContextQuery.isLoading,
    name,
    email,
    location,
    rawText,
    targetRegions,
    workModes,
    preferredRoles,
    allowedSources,
    includeKeywordsInput,
    excludeKeywordsInput,
    buildResult,
    searchResult,
    searchError,
    saveMutation,
    analyzeMutation,
    buildMutation,
    runMutation,
    setName,
    setEmail,
    setLocation,
    setRawText,
    setIncludeKeywordsInput,
    setExcludeKeywordsInput,
    toggleTargetRegion: (value: SearchProfileBuildResult['searchProfile']['targetRegions'][number]) =>
      setTargetRegions((current) => toggleValue(current, value)),
    toggleWorkMode: (value: SearchProfileBuildResult['searchProfile']['workModes'][number]) =>
      setWorkModes((current) => toggleValue(current, value)),
    togglePreferredRole: (value: string) =>
      setPreferredRoles((current) => toggleValue(current, value)),
    toggleAllowedSource: (value: string) =>
      setAllowedSources((current) => toggleValue(current, value)),
    saveCurrentProfile,
    buildCurrentSearchProfile: () => buildMutation.mutate(),
    runCurrentSearch: () => buildResult && runMutation.mutate(buildResult.searchProfile),
    analyzeProfile: () => analyzeMutation.mutate(),
    openFilePicker: () => fileInputRef.current?.click(),
    handleFileChange,
  };
}
