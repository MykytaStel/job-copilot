import type {
  LanguageLevel,
  LanguageProficiency,
  WorkModePreference,
} from '@job-copilot/shared/profiles';
import { useMemo, useState } from 'react';
import type { getProfile } from '../../api/profiles';

export type ProfileFormState = {
  name: string;
  email: string;
  location: string;
  rawText: string;
  yearsOfExperience: string;
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: LanguageProficiency[];
  preferredLocations: string[];
  workModePreference: WorkModePreference;
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
    preferredLocations: profile?.preferredLocations ?? [],
    workModePreference: profile?.workModePreference ?? 'any',
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
    setWorkModePreference: (value: WorkModePreference) =>
      updateDraft({ workModePreference: value }),
    addPreferredLocation: (location: string) =>
      updateDraft({ preferredLocations: addPreferredLocation(form.preferredLocations, location) }),
    removePreferredLocation: (location: string) =>
      updateDraft({
        preferredLocations: removePreferredLocation(form.preferredLocations, location),
      }),
    addLanguage: (language: string, level: LanguageLevel) =>
      updateDraft({ languages: addLanguage(form.languages, language, level) }),
    removeLanguage: (language: string) =>
      updateDraft({ languages: removeLanguage(form.languages, language) }),
    updateLanguageLevel: (language: string, level: LanguageLevel) =>
      updateDraft({ languages: updateLanguageLevel(form.languages, language, level) }),
  };
}

function normalizeLanguage(value: string): string {
  return value.trim().toLowerCase();
}

function addLanguage(
  languages: LanguageProficiency[],
  language: string,
  level: LanguageLevel,
): LanguageProficiency[] {
  const trimmed = language.trim();
  if (!trimmed) return languages;
  const normalized = normalizeLanguage(trimmed);
  if (languages.some((entry) => normalizeLanguage(entry.language) === normalized)) {
    return languages;
  }

  return [...languages, { language: trimmed, level }];
}

function removeLanguage(languages: LanguageProficiency[], language: string): LanguageProficiency[] {
  const normalized = normalizeLanguage(language);
  return languages.filter((entry) => normalizeLanguage(entry.language) !== normalized);
}

function updateLanguageLevel(
  languages: LanguageProficiency[],
  language: string,
  level: LanguageLevel,
): LanguageProficiency[] {
  const normalized = normalizeLanguage(language);
  return languages.map((entry) =>
    normalizeLanguage(entry.language) === normalized ? { ...entry, level } : entry,
  );
}

function normalizePreferredLocation(value: string): string {
  return value.trim().toLowerCase();
}

function addPreferredLocation(locations: string[], location: string): string[] {
  const trimmed = location.trim();
  if (!trimmed) return locations;
  const normalized = normalizePreferredLocation(trimmed);
  if (locations.some((entry) => normalizePreferredLocation(entry) === normalized)) {
    return locations;
  }

  return [...locations, trimmed];
}

function removePreferredLocation(locations: string[], location: string): string[] {
  const normalized = normalizePreferredLocation(location);
  return locations.filter((entry) => normalizePreferredLocation(entry) !== normalized);
}
