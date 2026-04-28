import { json, request } from './client';

export async function resetProfileData(profileId: string): Promise<void> {
  await request<void>(
    '/api/v1/data/reset',
    json('POST', {
      profile_id: profileId,
    }),
  );
}
