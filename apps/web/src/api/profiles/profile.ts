import type { CandidateProfile, CandidateProfileInput } from '@job-copilot/shared/profiles';
import { json, readStoredProfileId, request, writeStoredProfileId } from '../client';
import type { EngineAnalyzeProfile, EngineProfile, EngineResume } from '../engine-types';
import { mapProfile } from '../mappers';

export async function getStoredProfileRawText(): Promise<string> {
  const profileId = readStoredProfileId();
  if (!profileId) return '';
  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return profile.raw_text;
}

export async function analyzeStoredProfile(): Promise<EngineAnalyzeProfile> {
  const profileId = readStoredProfileId();
  if (!profileId) {
    throw new Error('Create a profile first');
  }
  return request<EngineAnalyzeProfile>(`/api/v1/profiles/${profileId}/analyze`, json('POST', {}));
}

export async function getProfile(): Promise<CandidateProfile | undefined> {
  const profileId = readStoredProfileId();
  if (!profileId) return undefined;
  const profile = await request<EngineProfile>(`/api/v1/profiles/${profileId}`);
  return mapProfile(profile);
}

export async function saveProfile(
  payload: CandidateProfileInput & { rawText: string },
): Promise<CandidateProfile> {
  const profileId = readStoredProfileId();

  const body = {
    name: payload.name,
    email: payload.email,
    location: payload.location ?? null,
    raw_text: payload.rawText,
    years_of_experience: payload.yearsOfExperience ?? null,
    salary_min: payload.salaryMin ?? null,
    salary_max: payload.salaryMax ?? null,
    salary_currency: payload.salaryCurrency ?? null,
    languages: payload.languages,
  };

  const profile = profileId
    ? await request<EngineProfile>(`/api/v1/profiles/${profileId}`, json('PATCH', body))
    : await request<EngineProfile>('/api/v1/profiles', json('POST', body));

  writeStoredProfileId(profile.id);

  // Upload resume in parallel with analysis so the engine-api fit-score endpoint
  // has an active resume to work with (resumes and profiles are separate records).
  const [analyzed] = await Promise.all([
    request<EngineAnalyzeProfile>(`/api/v1/profiles/${profile.id}/analyze`, json('POST', {})),
    request<EngineResume>(
      '/api/v1/resume/upload',
      json('POST', {
        filename: 'profile.md',
        raw_text: payload.rawText,
      }),
    ).catch(() => null),
  ]);

  return {
    ...mapProfile(profile),
    summary: analyzed.summary,
    skills: analyzed.skills,
  };
}
