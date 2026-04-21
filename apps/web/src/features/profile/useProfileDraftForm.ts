import { useMemo, useState } from 'react';
import type { getProfile } from '../../api/profiles';
import { toggleValue } from './profile.utils';

export type ProfileFormState = {
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

function buildFormState(
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

export function useProfileDraftForm(
  profileData: Awaited<ReturnType<typeof getProfile>> | undefined,
  rawText: string | undefined,
) {
  const [profileDraft, setProfileDraft] = useState<ProfileFormState | null>(null);

  const baseForm = useMemo(() => buildFormState(profileData, rawText), [profileData, rawText]);
  const form = profileDraft ?? baseForm;

  function updateDraft(patch: Partial<ProfileFormState>) {
    setProfileDraft((current) => ({
      ...(current ?? form),
      ...patch,
    }));
  }

  return {
    form,
    clearDraft: () => setProfileDraft(null),
    updateDraft,
    setName: (value: string) => updateDraft({ name: value }),
    setEmail: (value: string) => updateDraft({ email: value }),
    setLocation: (value: string) => updateDraft({ location: value }),
    setRawText: (value: string) => updateDraft({ rawText: value }),
    setYearsOfExperience: (value: string) => updateDraft({ yearsOfExperience: value }),
    setSalaryMin: (value: string) => updateDraft({ salaryMin: value }),
    setSalaryMax: (value: string) => updateDraft({ salaryMax: value }),
    setSalaryCurrency: (value: string) => updateDraft({ salaryCurrency: value }),
    toggleLanguage: (value: string) =>
      updateDraft({ languages: toggleValue(form.languages, value) }),
  };
}
