export type ProfileCompletionInput = {
  name: string;
  email: string;
  location: string;
  rawText: string;
  yearsOfExperience: string;
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: string[];
  analysisReady: boolean;
};

export type ProfileCompletionCheckpoint = {
  id:
    | 'name'
    | 'email'
    | 'location'
    | 'raw_text'
    | 'experience'
    | 'compensation'
    | 'languages'
    | 'analysis';
  label: string;
  complete: boolean;
};

export type ProfileCompletionState = {
  percent: number;
  completed: number;
  total: number;
  checkpoints: ProfileCompletionCheckpoint[];
  missingLabels: string[];
};

export function buildProfileCompletionState(input: ProfileCompletionInput): ProfileCompletionState {
  const compensationComplete =
    input.salaryMin.trim().length > 0 &&
    input.salaryMax.trim().length > 0 &&
    input.salaryCurrency.trim().length > 0;

  const checkpoints: ProfileCompletionCheckpoint[] = [
    { id: 'name', label: 'Name', complete: input.name.trim().length > 0 },
    { id: 'email', label: 'Email', complete: input.email.trim().length > 0 },
    { id: 'location', label: 'Location', complete: input.location.trim().length > 0 },
    { id: 'raw_text', label: 'Resume text', complete: input.rawText.trim().length > 0 },
    {
      id: 'experience',
      label: 'Experience',
      complete: input.yearsOfExperience.trim().length > 0,
    },
    { id: 'compensation', label: 'Compensation', complete: compensationComplete },
    { id: 'languages', label: 'Languages', complete: input.languages.length > 0 },
    { id: 'analysis', label: 'Analysis', complete: input.analysisReady },
  ];

  const completed = checkpoints.filter((checkpoint) => checkpoint.complete).length;
  const total = checkpoints.length;
  const percent = Math.round((completed / total) * 100);

  return {
    percent,
    completed,
    total,
    checkpoints,
    missingLabels: checkpoints
      .filter((checkpoint) => !checkpoint.complete)
      .map((checkpoint) => checkpoint.label),
  };
}
