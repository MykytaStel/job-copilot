import { json, request } from './client';

export type OnboardingStep =
  | 'cv'
  | 'profile'
  | 'preferences'
  | 'recommendations'
  | 'complete';

export type OnboardingOutcome = 'completed' | 'skipped';

export type ProfileOnboarding = {
  profile_id: string;
  current_step: OnboardingStep;
  completed_steps: OnboardingStep[];
  skipped_steps: OnboardingStep[];
  completed_at?: string | null;
  updated_at: string;
};

export async function getProfileOnboarding(profileId: string): Promise<ProfileOnboarding> {
  return request<ProfileOnboarding>(`/api/v1/profiles/${profileId}/onboarding`);
}

export async function advanceProfileOnboarding(
  profileId: string,
  step: Exclude<OnboardingStep, 'complete'>,
  outcome: OnboardingOutcome = 'completed',
): Promise<ProfileOnboarding> {
  return request<ProfileOnboarding>(
    `/api/v1/profiles/${profileId}/onboarding`,
    json('PATCH', { step, outcome }),
  );
}
