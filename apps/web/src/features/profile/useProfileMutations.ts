import { useMutation, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { analyzeStoredProfile, getProfile, saveProfile } from '../../api/profiles';
import type { PersistedSearchPreferences } from '../../api/profiles';
import { invalidateProfileAnalysisQueries } from '../../lib/queryInvalidation';
import { queryKeys } from '../../queryKeys';

export function useProfileMutations(clearDraft: () => void) {
  const queryClient = useQueryClient();

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
      searchPreferences?: PersistedSearchPreferences;
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
        searchPreferences: vars.searchPreferences,
        summary: undefined,
        skills: [],
      }),
    onSuccess: (updated, vars) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      queryClient.setQueryData(queryKeys.profile.rawText(), vars.rawText);
      clearDraft();
      void invalidateProfileAnalysisQueries(queryClient, updated.id);
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
          current ? { ...current, summary: analysis.summary, skills: analysis.skills } : current,
      );
      const profileId = (
        queryClient.getQueryData(queryKeys.profile.root()) as { id?: string } | undefined
      )?.id;
      if (profileId) {
        void invalidateProfileAnalysisQueries(queryClient, profileId);
      }
      toast.success('Profile analyzed');
    },
    onError: (error: unknown) => toast.error(error instanceof Error ? error.message : 'Error'),
  });

  return { saveMutation, analyzeMutation };
}
