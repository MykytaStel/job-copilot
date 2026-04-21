import { useQuery } from '@tanstack/react-query';
import { getLlmContext } from '../../api/analytics';
import { getProfile, getRoles, getSources, getStoredProfileRawText } from '../../api/profiles';
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
  const sourcesQuery = useQuery({
    queryKey: queryKeys.sources.all(),
    queryFn: getSources,
  });
  const llmContextQuery = useQuery({
    queryKey: queryKeys.analytics.llmContext(profileQuery.data?.id ?? ''),
    queryFn: () => getLlmContext(profileQuery.data!.id),
    enabled: !!profileQuery.data?.id,
  });

  return { profileQuery, rawTextQuery, rolesQuery, sourcesQuery, llmContextQuery };
}
