import { useQuery } from '@tanstack/react-query';
import { getLlmContext } from '../../api/analytics';
import { getJobsFeed } from '../../api/jobs';
import {
  getProfile,
  getResumes,
  getRoles,
  getSources,
  getStoredProfileRawText,
} from '../../api/profiles';
import { queryKeys } from '../../queryKeys';

export function useProfileQueries() {
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
  const resumesQuery = useQuery({
    queryKey: queryKeys.resumes.all(),
    queryFn: getResumes,
  });
  const activeJobsQuery = useQuery({
    queryKey: queryKeys.jobs.filtered('active', null, profileQuery.data?.id),
    queryFn: () => getJobsFeed({ lifecycle: 'active', limit: 1 }),
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

  return {
    profileQuery,
    rawTextQuery,
    rolesQuery,
    resumesQuery,
    activeJobsQuery,
    sourcesQuery,
    llmContextQuery,
  };
}
