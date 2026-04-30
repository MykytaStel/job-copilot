import { useMutation, useQueryClient } from '@tanstack/react-query';
import type {
  ExperienceEntry,
  LanguageProficiency,
  WorkModePreference,
} from '@job-copilot/shared/profiles';

import { useToast } from '../../context/ToastContext';
import {
  analyzeStoredProfile,
  activateResume,
  deleteResume,
  getProfile,
  saveProfile,
  updateProfileSkills,
} from '../../api/profiles';
import type { PersistedSearchPreferences } from '../../api/profiles';
import { invalidateProfileAnalysisQueries } from '../../lib/queryInvalidation';
import { queryKeys } from '../../queryKeys';

export function useProfileMutations(clearDraft: () => void) {
  const queryClient = useQueryClient();
  const { showToast } = useToast();

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
      languages: LanguageProficiency[];
      preferredLocations: string[];
      experience: ExperienceEntry[];
      workModePreference: WorkModePreference;
      searchPreferences?: PersistedSearchPreferences;
      portfolioUrl?: string;
      githubUrl?: string;
      linkedinUrl?: string;
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
        preferredLocations: vars.preferredLocations,
        experience: vars.experience,
        workModePreference: vars.workModePreference,
        searchPreferences: vars.searchPreferences,
        summary: undefined,
        skills: [],
        portfolioUrl: vars.portfolioUrl,
        githubUrl: vars.githubUrl,
        linkedinUrl: vars.linkedinUrl,
      }),
    onSuccess: (updated, vars) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      queryClient.setQueryData(queryKeys.profile.rawText(), vars.rawText);
      clearDraft();
      void invalidateProfileAnalysisQueries(queryClient, updated.id);
      showToast({ type: 'success', message: 'Profile saved' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Error' }),
  });

  const analyzeMutation = useMutation({
    mutationFn: analyzeStoredProfile,
    onSuccess: (analysis) => {
      queryClient.setQueryData(
        queryKeys.profile.root(),
        (current: Awaited<ReturnType<typeof getProfile>>) =>
          current ? { ...current, summary: analysis.summary } : current,
      );
      const profileId = (
        queryClient.getQueryData(queryKeys.profile.root()) as { id?: string } | undefined
      )?.id;
      if (profileId) {
        void invalidateProfileAnalysisQueries(queryClient, profileId);
      }
      showToast({ type: 'success', message: 'Profile analyzed' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Error' }),
  });

  const updateSkillsMutation = useMutation({
    mutationFn: (vars: { profileId: string; skills: string[] }) =>
      updateProfileSkills(vars.profileId, vars.skills),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      void invalidateProfileAnalysisQueries(queryClient, updated.id);
      showToast({ type: 'success', message: 'Skills updated' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Error' }),
  });

  const activateResumeMutation = useMutation({
    mutationFn: activateResume,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      showToast({ type: 'success', message: 'CV activated' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Error' }),
  });

  const deleteResumeMutation = useMutation({
    mutationFn: deleteResume,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.active() });
      showToast({ type: 'success', message: 'CV deleted' });
    },
    onError: (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Error' }),
  });

  return {
    saveMutation,
    analyzeMutation,
    updateSkillsMutation,
    activateResumeMutation,
    deleteResumeMutation,
  };
}
