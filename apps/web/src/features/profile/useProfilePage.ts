import { useMemo, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';

import {
  analyzeStoredProfile,
  buildSearchProfile,
  getProfile,
  getRoles,
  getSources,
  getStoredProfileRawText,
  saveProfile,
  type SearchProfileBuildResult,
} from '../../api/profiles';
import { getLlmContext } from '../../api/analytics';
import type { SearchRunResult } from '../../api/jobs';
import { runSearch } from '../../api/jobs';
import { queryKeys } from '../../queryKeys';
import {
  cleanupExtractedResumeText,
  extractPdfText,
  parseKeywordInput,
  toggleValue,
} from './profile.utils';

type ProfileFormState = {
  name: string;
  email: string;
  location: string;
  rawText: string;
  yearsOfExperience: string;
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: string[];
};

function getProfileFormState(
  profile: Awaited<ReturnType<typeof getProfile>> | undefined,
  rawText: string | undefined,
): ProfileFormState {
  return {
    name: profile?.name ?? '',
    email: profile?.email ?? '',
    location: profile?.location ?? '',
    rawText: rawText ?? '',
    yearsOfExperience: profile?.yearsOfExperience?.toString() ?? '',
    salaryMin: profile?.salaryMin?.toString() ?? '',
    salaryMax: profile?.salaryMax?.toString() ?? '',
    salaryCurrency: profile?.salaryCurrency ?? 'USD',
    languages: profile?.languages ?? [],
  };
}

export function useProfilePage() {
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [profileDraft, setProfileDraft] = useState<ProfileFormState | null>(null);
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

  const profileQuery = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });
  const rawTextQuery = useQuery({
    queryKey: queryKeys.profile.rawText(),
    queryFn: getStoredProfileRawText,
    enabled: Boolean(profileQuery.data),
    retry: false,
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

  const baseProfileForm = useMemo(
    () => getProfileFormState(profileQuery.data, rawTextQuery.data),
    [profileQuery.data, rawTextQuery.data],
  );
  const profileForm = profileDraft ?? baseProfileForm;

  function updateProfileDraft(patch: Partial<ProfileFormState>) {
    setProfileDraft((current) => ({
      ...(current ?? profileForm),
      ...patch,
    }));
  }

  const {
    name,
    email,
    location,
    rawText,
    yearsOfExperience,
    salaryMin,
    salaryMax,
    salaryCurrency,
  } = profileForm;
  const languages = profileForm.languages;

  const saveMutation = useMutation({
    mutationFn: (vars: {
      name: string;
      email: string;
      location?: string;
      rawText: string;
      yearsOfExperience?: number;
      salaryMin?: number;
      salaryMax?: number;
      salaryCurrency: string;
      languages: string[];
    }) =>
      saveProfile({
        name: vars.name,
        email: vars.email,
        location: vars.location,
        rawText: vars.rawText,
        yearsOfExperience: vars.yearsOfExperience,
        salaryMin: vars.salaryMin,
        salaryMax: vars.salaryMax,
        salaryCurrency: vars.salaryCurrency,
        languages: vars.languages,
        summary: undefined,
        skills: [],
      }),
    onSuccess: (updated, vars) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      queryClient.setQueryData(queryKeys.profile.rawText(), vars.rawText);
      setProfileDraft(null);
      void queryClient.invalidateQueries({ queryKey: ['ml', 'rerank', updated.id] });
      void queryClient.invalidateQueries({ queryKey: ['analytics'] });
      toast.success('Profile saved');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Error'),
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
      const profileId = (
        queryClient.getQueryData(queryKeys.profile.root()) as { id?: string } | undefined
      )?.id;
      if (profileId) {
        void queryClient.invalidateQueries({ queryKey: ['ml', 'rerank', profileId] });
        void queryClient.invalidateQueries({ queryKey: ['analytics'] });
      }
      toast.success('Profile analyzed');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Error'),
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
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Error'),
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
          updateProfileDraft({ rawText: text });
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
      const cleanedText = typeof text === 'string' ? cleanupExtractedResumeText(text) : '';
      if (cleanedText.trim()) {
        updateProfileDraft({ rawText: cleanedText });
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
      yearsOfExperience: parseOptionalNumber(yearsOfExperience),
      salaryMin: parseOptionalNumber(salaryMin),
      salaryMax: parseOptionalNumber(salaryMax),
      salaryCurrency,
      languages,
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
    yearsOfExperience,
    salaryMin,
    salaryMax,
    salaryCurrency,
    languages,
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
    setName: (value: string) => updateProfileDraft({ name: value }),
    setEmail: (value: string) => updateProfileDraft({ email: value }),
    setLocation: (value: string) => updateProfileDraft({ location: value }),
    setRawText: (value: string) => updateProfileDraft({ rawText: value }),
    setYearsOfExperience: (value: string) => updateProfileDraft({ yearsOfExperience: value }),
    setSalaryMin: (value: string) => updateProfileDraft({ salaryMin: value }),
    setSalaryMax: (value: string) => updateProfileDraft({ salaryMax: value }),
    setSalaryCurrency: (value: string) => updateProfileDraft({ salaryCurrency: value }),
    setIncludeKeywordsInput,
    setExcludeKeywordsInput,
    toggleLanguage: (value: string) =>
      updateProfileDraft({
        languages: toggleValue(languages, value),
      }),
    toggleTargetRegion: (
      value: SearchProfileBuildResult['searchProfile']['targetRegions'][number],
    ) => setTargetRegions((current) => toggleValue(current, value)),
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

function parseOptionalNumber(value: string): number | undefined {
  const normalized = value.trim();
  if (!normalized) {
    return undefined;
  }

  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : undefined;
}
