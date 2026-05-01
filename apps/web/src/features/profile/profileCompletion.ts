import type { LanguageProficiency } from '@job-copilot/shared/profiles';

export type ProfileCompletionInput = {
  name: string;
  email: string;
  rawText: string;
  skills: string[];
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: LanguageProficiency[];
  preferredLocations: string[];
  targetRegions: string[];
  workModes: string[];
  preferredRoles: string[];
};

export type ProfileCompletionCheckpoint = {
  id:
    | 'name'
    | 'email'
    | 'skills'
    | 'cv'
    | 'salary'
    | 'work_mode'
    | 'preferred_roles'
    | 'location_preference'
    | 'language_preference';
  label: string;
  complete: boolean;
  weight: number;
  targetId: string;
};

export type ProfileCompletionState = {
  percent: number;
  completedWeight: number;
  totalWeight: number;
  checkpoints: ProfileCompletionCheckpoint[];
  missing: ProfileCompletionCheckpoint[];
  missingLabels: string[];
};

export function buildProfileCompletionState(input: ProfileCompletionInput): ProfileCompletionState {
  const hasSalaryExpectation =
    (input.salaryMin.trim().length > 0 || input.salaryMax.trim().length > 0) &&
    input.salaryCurrency.trim().length > 0;

  const checkpoints: ProfileCompletionCheckpoint[] = [
    {
      id: 'name',
      label: 'Name',
      complete: input.name.trim().length > 0,
      weight: 10,
      targetId: 'profile-field-name',
    },
    {
      id: 'email',
      label: 'Email',
      complete: input.email.trim().length > 0,
      weight: 10,
      targetId: 'profile-field-email',
    },
    {
      id: 'skills',
      label: 'At least 3 skills',
      complete: input.skills.length >= 3,
      weight: 20,
      targetId: 'profile-field-cv',
    },
    {
      id: 'cv',
      label: 'CV uploaded',
      complete: input.rawText.trim().length > 0,
      weight: 20,
      targetId: 'profile-field-cv',
    },
    {
      id: 'salary',
      label: 'Salary expectation',
      complete: hasSalaryExpectation,
      weight: 10,
      targetId: 'profile-field-salary',
    },
    {
      id: 'work_mode',
      label: 'Work mode preference',
      complete: input.workModes.length > 0,
      weight: 10,
      targetId: 'profile-field-work-mode',
    },
    {
      id: 'preferred_roles',
      label: 'Preferred roles',
      complete: input.preferredRoles.length > 0,
      weight: 20,
      targetId: 'profile-field-preferred-roles',
    },
    {
      id: 'location_preference',
      label: 'Location preference',
      complete: input.targetRegions.length > 0 || input.preferredLocations.length > 0,
      weight: 10,
      targetId: 'profile-field-preferred-locations',
    },
    {
      id: 'language_preference',
      label: 'Language preference',
      complete: input.languages.length > 0,
      weight: 10,
      targetId: 'profile-field-languages',
    },
  ];

  const completedWeight = checkpoints.reduce(
    (sum, checkpoint) => sum + (checkpoint.complete ? checkpoint.weight : 0),
    0,
  );
  const totalWeight = checkpoints.reduce((sum, checkpoint) => sum + checkpoint.weight, 0);
  const percent = Math.round((completedWeight / totalWeight) * 100);
  const missing = checkpoints.filter((checkpoint) => !checkpoint.complete);

  return {
    percent,
    completedWeight,
    totalWeight,
    checkpoints,
    missing,
    missingLabels: missing.map((checkpoint) => checkpoint.label),
  };
}
