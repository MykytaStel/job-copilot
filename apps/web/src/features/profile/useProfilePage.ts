import { useMemo, useState } from 'react';
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
  const [suggestedSkills, setSuggestedSkills] = useState<string[]>([]);

  const { form } = draftForm;
  const confirmedSkills = queries.profileQuery.data?.skills ?? [];
  const visibleSuggestedSkills = useMemo(() => uniqueSkills(suggestedSkills), [suggestedSkills]);
  const profileCompletion = buildProfileCompletionState({
    name: form.name,
    email: form.email,
    rawText: form.rawText,
    skills: confirmedSkills,
    salaryMin: form.salaryMin,
    salaryMax: form.salaryMax,
    salaryCurrency: form.salaryCurrency,
    languages: form.languages,
    preferredLocations: form.preferredLocations,
    targetRegions: search.targetRegions,
    workModes: search.workModes,
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
      preferredLocations: form.preferredLocations,
      workModePreference: form.workModePreference,
      searchPreferences: buildPersistedSearchPreferences({
        targetRegions: search.targetRegions,
        workModes: search.workModes,
        preferredRoles: search.preferredRoles,
        allowedSources: search.allowedSources,
        includeKeywordsInput: search.includeKeywordsInput,
        excludeKeywordsInput: search.excludeKeywordsInput,
      }),
			portfolioUrl: form.portfolioUrl.trim() || undefined,
			githubUrl: form.githubUrl.trim() || undefined,
			linkedinUrl: form.linkedinUrl.trim() || undefined,
    });
  }

  function analyzeProfile() {
    mutations.analyzeMutation.mutate(undefined, {
      onSuccess: (analysis) => {
        setSuggestedSkills(uniqueSkills(analysis.skills));
      },
    });
  }

  function addSuggestedSkill(skill: string) {
    const profile = queries.profileQuery.data;
    if (!profile) return;

    const mergedSkills = mergeSkills(profile.skills, [skill]);
    mutations.updateSkillsMutation.mutate(
      { profileId: profile.id, skills: mergedSkills },
      {
        onSuccess: () => {
          setSuggestedSkills((current) => removeSkills(current, [skill]));
        },
      },
    );
  }

  function addAllSuggestedSkills() {
    const profile = queries.profileQuery.data;
    if (!profile) return;

    const skillsToAdd = excludeExistingSkills(suggestedSkills, profile.skills);
    const mergedSkills = mergeSkills(profile.skills, skillsToAdd);
    mutations.updateSkillsMutation.mutate(
      { profileId: profile.id, skills: mergedSkills },
      {
        onSuccess: () => setSuggestedSkills([]),
      },
    );
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
    preferredLocations: form.preferredLocations,
    workModePreference: form.workModePreference,
    profileCompletion,
    targetRegions: search.targetRegions,
    workModes: search.workModes,
    preferredRoles: search.preferredRoles,
    allowedSources: search.allowedSources,
    includeKeywordsInput: search.includeKeywordsInput,
    excludeKeywordsInput: search.excludeKeywordsInput,
    buildResult: search.buildResult,
    buildIsCurrent: search.buildIsCurrent,
    buildRestoredFromStorage: search.buildRestoredFromStorage,
    searchResult: search.searchResult,
    searchError: search.searchError,
    suggestedSkills: visibleSuggestedSkills,
    saveMutation: mutations.saveMutation,
    analyzeMutation: mutations.analyzeMutation,
    updateSkillsMutation: mutations.updateSkillsMutation,
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
    setWorkModePreference: draftForm.setWorkModePreference,
    addPreferredLocation: draftForm.addPreferredLocation,
    removePreferredLocation: draftForm.removePreferredLocation,
    setIncludeKeywordsInput: search.setIncludeKeywordsInput,
    setExcludeKeywordsInput: search.setExcludeKeywordsInput,
    addLanguage: draftForm.addLanguage,
    removeLanguage: draftForm.removeLanguage,
    updateLanguageLevel: draftForm.updateLanguageLevel,
    toggleTargetRegion: search.toggleTargetRegion,
    toggleWorkMode: search.toggleWorkMode,
    togglePreferredRole: search.togglePreferredRole,
    toggleAllowedSource: search.toggleAllowedSource,
    saveCurrentProfile,
    buildCurrentSearchProfile: search.buildCurrentSearchProfile,
    runCurrentSearch: search.runCurrentSearch,
    analyzeProfile,
    addSuggestedSkill,
    addAllSuggestedSkills,
    openFilePicker: picker.openFilePicker,
    handleFileChange: picker.handleFileChange,
		portfolioUrl: form.portfolioUrl,
		githubUrl: form.githubUrl,
		linkedinUrl: form.linkedinUrl,
		setPortfolioUrl: draftForm.setPortfolioUrl,
		setGithubUrl: draftForm.setGithubUrl,
		setLinkedinUrl: draftForm.setLinkedinUrl,
  };
}

function normalizeSkill(value: string): string {
  return value.trim().toLowerCase();
}

function uniqueSkills(skills: string[]): string[] {
  const seen = new Set<string>();
  const result: string[] = [];

  for (const skill of skills) {
    const trimmed = skill.trim();
    const normalized = normalizeSkill(trimmed);
    if (!trimmed || seen.has(normalized)) continue;
    seen.add(normalized);
    result.push(trimmed);
  }

  return result;
}

function excludeExistingSkills(skills: string[], existingSkills: string[]): string[] {
  const existing = new Set(existingSkills.map(normalizeSkill));
  return uniqueSkills(skills).filter((skill) => !existing.has(normalizeSkill(skill)));
}

function mergeSkills(existingSkills: string[], skillsToAdd: string[]): string[] {
  return uniqueSkills([...existingSkills, ...skillsToAdd]);
}

function removeSkills(skills: string[], skillsToRemove: string[]): string[] {
  const remove = new Set(skillsToRemove.map(normalizeSkill));
  return skills.filter((skill) => !remove.has(normalizeSkill(skill)));
}

function parseOptionalNumber(value: string): number | undefined {
  const normalized = value.trim();
  if (!normalized) return undefined;
  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : undefined;
}
