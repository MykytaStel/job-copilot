import { useProfileDraftForm } from './useProfileDraftForm';
import { buildProfileCompletionState } from './profileCompletion';
import { useProfileMutations } from './useProfileMutations';
import { useProfileQueries } from './useProfileQueries';
import { useResumePicker } from './useResumePicker';
import { buildPersistedSearchPreferences } from './searchProfilePreferences';
import { useSearchProfileWorkflow } from './useSearchProfileWorkflow';

export function useProfilePage() {
  const queries = useProfileQueries();
  const draftForm = useProfileDraftForm(queries.profileQuery.data, queries.rawTextQuery.data);
  const mutations = useProfileMutations(draftForm.clearDraft);
  const search = useSearchProfileWorkflow(draftForm.form.rawText, queries.profileQuery.data);
  const picker = useResumePicker(draftForm.setRawText);

  const { form } = draftForm;
  const profileCompletion = buildProfileCompletionState({
    name: form.name,
    email: form.email,
    location: form.location,
    rawText: form.rawText,
    yearsOfExperience: form.yearsOfExperience,
    salaryMin: form.salaryMin,
    salaryMax: form.salaryMax,
    salaryCurrency: form.salaryCurrency,
    languages: form.languages,
    analysisReady: Boolean(
      queries.profileQuery.data?.summary || queries.profileQuery.data?.skills.length,
    ),
  });

  function saveCurrentProfile() {
    mutations.saveMutation.mutate({
      name: form.name,
      email: form.email,
      location: form.location || undefined,
      rawText: form.rawText,
      yearsOfExperience: parseOptionalNumber(form.yearsOfExperience),
      salaryMin: parseOptionalNumber(form.salaryMin),
      salaryMax: parseOptionalNumber(form.salaryMax),
      salaryCurrency: form.salaryCurrency,
      languages: form.languages,
      searchPreferences: buildPersistedSearchPreferences({
        targetRegions: search.targetRegions,
        workModes: search.workModes,
        preferredRoles: search.preferredRoles,
        allowedSources: search.allowedSources,
        includeKeywordsInput: search.includeKeywordsInput,
        excludeKeywordsInput: search.excludeKeywordsInput,
      }),
    });
  }

  return {
    fileInputRef: picker.fileInputRef,
    profile: queries.profileQuery.data,
    roles: queries.rolesQuery.data ?? [],
    sources: queries.sourcesQuery.data ?? [],
    rolesError: queries.rolesQuery.error,
    sourcesError: queries.sourcesQuery.error,
    llmContext: queries.llmContextQuery.data ?? null,
    llmContextError: queries.llmContextQuery.error,
    llmContextLoading: queries.llmContextQuery.isLoading,
    name: form.name,
    email: form.email,
    location: form.location,
    rawText: form.rawText,
    yearsOfExperience: form.yearsOfExperience,
    salaryMin: form.salaryMin,
    salaryMax: form.salaryMax,
    salaryCurrency: form.salaryCurrency,
    languages: form.languages,
    profileCompletion,
    targetRegions: search.targetRegions,
    workModes: search.workModes,
    preferredRoles: search.preferredRoles,
    allowedSources: search.allowedSources,
    includeKeywordsInput: search.includeKeywordsInput,
    excludeKeywordsInput: search.excludeKeywordsInput,
    buildResult: search.buildResult,
    searchResult: search.searchResult,
    searchError: search.searchError,
    saveMutation: mutations.saveMutation,
    analyzeMutation: mutations.analyzeMutation,
    buildMutation: search.buildMutation,
    runMutation: search.runMutation,
    setName: draftForm.setName,
    setEmail: draftForm.setEmail,
    setLocation: draftForm.setLocation,
    setRawText: draftForm.setRawText,
    setYearsOfExperience: draftForm.setYearsOfExperience,
    setSalaryMin: draftForm.setSalaryMin,
    setSalaryMax: draftForm.setSalaryMax,
    setSalaryCurrency: draftForm.setSalaryCurrency,
    setIncludeKeywordsInput: search.setIncludeKeywordsInput,
    setExcludeKeywordsInput: search.setExcludeKeywordsInput,
    toggleLanguage: draftForm.toggleLanguage,
    toggleTargetRegion: search.toggleTargetRegion,
    toggleWorkMode: search.toggleWorkMode,
    togglePreferredRole: search.togglePreferredRole,
    toggleAllowedSource: search.toggleAllowedSource,
    saveCurrentProfile,
    buildCurrentSearchProfile: search.buildCurrentSearchProfile,
    runCurrentSearch: search.runCurrentSearch,
    analyzeProfile: () => mutations.analyzeMutation.mutate(),
    openFilePicker: picker.openFilePicker,
    handleFileChange: picker.handleFileChange,
  };
}

function parseOptionalNumber(value: string): number | undefined {
  const normalized = value.trim();
  if (!normalized) return undefined;
  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : undefined;
}
