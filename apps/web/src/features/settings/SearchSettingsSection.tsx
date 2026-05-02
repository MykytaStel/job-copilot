import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';

import {
  DEFAULT_SCORING_WEIGHTS,
  getProfile,
  updateScoringWeights,
  type ScoringWeights,
} from '../../api/profiles';
import { Button } from '../../components/ui/Button';
import { cn } from '../../lib/cn';
import {
  INGESTION_FREQUENCY_OPTIONS,
  readIngestionFrequency,
  writeIngestionFrequency,
  type IngestionFrequencyMinutes,
} from '../../lib/ingestionFrequencyPrefs';
import { queryKeys } from '../../queryKeys';
import { SettingsSection, SettingRow } from './settingsShared';

const SCORING_WEIGHT_CONTROLS: {
  key: keyof ScoringWeights;
  title: string;
  description: string;
}[] = [
  {
    key: 'skillMatchImportance',
    title: 'Skill match importance',
    description: 'Prioritizes jobs that mention your strongest skills and profile keywords.',
  },
  {
    key: 'salaryFitImportance',
    title: 'Salary fit importance',
    description: 'Changes how strongly salary fit can boost or penalize a job.',
  },
  {
    key: 'jobFreshnessImportance',
    title: 'Job freshness importance',
    description: 'Changes how strongly old job posts are penalized.',
  },
  {
    key: 'remoteWorkImportance',
    title: 'Remote work importance',
    description: 'Changes how strongly remote/work-mode match affects ranking.',
  },
];

export function SearchSettingsSection() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [ingestionFrequency, setIngestionFrequencyState] = useState(() => readIngestionFrequency());

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });

  const scoringWeightsMutation = useMutation({
    mutationFn: updateScoringWeights,
    onSuccess: (updatedProfile) => {
      queryClient.setQueryData(queryKeys.profile.root(), updatedProfile);
      void queryClient.invalidateQueries({ queryKey: queryKeys.profile.root() });
    },
  });

  function setIngestionFrequency(value: IngestionFrequencyMinutes) {
    writeIngestionFrequency(value);
    setIngestionFrequencyState(value);
  }

  const persistedSearchPreferences = profile?.searchPreferences;
  const scoringWeights = persistedSearchPreferences?.scoringWeights ?? DEFAULT_SCORING_WEIGHTS;

  const persistedPreferenceCount =
    (persistedSearchPreferences?.targetRegions.length ?? 0) +
    (persistedSearchPreferences?.workModes.length ?? 0) +
    (persistedSearchPreferences?.preferredRoles.length ?? 0) +
    (persistedSearchPreferences?.allowedSources.length ?? 0) +
    (persistedSearchPreferences?.includeKeywords.length ?? 0) +
    (persistedSearchPreferences?.excludeKeywords.length ?? 0);

  function updateScoringWeight(key: keyof ScoringWeights, value: number) {
    scoringWeightsMutation.mutate({ ...scoringWeights, [key]: value });
  }

  return (
    <SettingsSection title="Search Preferences">
      <div className="space-y-4">
        <div className="grid gap-3 md:grid-cols-2">
          <SettingRow
            label="Active filters"
            value={
              persistedPreferenceCount > 0
                ? `${persistedPreferenceCount} active filters`
                : 'None set'
            }
          />
          <SettingRow
            label="Persistence"
            value={persistedSearchPreferences ? 'Profile-scoped' : 'Local until saved'}
          />
          <SettingRow
            label="Target regions"
            value={String(persistedSearchPreferences?.targetRegions.length ?? 0)}
          />
          <SettingRow
            label="Preferred roles"
            value={String(persistedSearchPreferences?.preferredRoles.length ?? 0)}
          />
          <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
            <div className="mb-4">
              <p className="text-sm font-semibold text-foreground">Ingestion frequency</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Choose how often you want new jobs to appear refreshed in the UI.
              </p>
            </div>

            <div className="grid gap-3 sm:grid-cols-3">
              {INGESTION_FREQUENCY_OPTIONS.map((value) => {
                const selected = ingestionFrequency === value;
                return (
                  <button
                    key={value}
                    type="button"
                    onClick={() => setIngestionFrequency(value)}
                    className={cn(
                      'rounded-[var(--radius-md)] border px-4 py-3 text-left transition-colors',
                      selected
                        ? 'border-primary bg-primary/10 text-primary'
                        : 'border-border bg-surface text-muted-foreground hover:border-border/80 hover:text-foreground',
                    )}
                  >
                    <span className="block text-sm font-semibold">Every {value} min</span>
                    <span className="mt-1 block text-xs text-muted-foreground">
                      Display preference
                    </span>
                  </button>
                );
              })}
            </div>

            <p className="mt-4 text-xs text-muted-foreground">
              Saved locally. Scraping currently runs every 60 min by system default unless the
              ingestion daemon is started with a different interval.
            </p>
          </div>

          <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
            <div className="mb-4">
              <p className="text-sm font-semibold text-foreground">Scoring weights</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Tune how Job Copilot ranks jobs for this profile. Changes are saved to engine-api
                and affect the next search run.
              </p>
            </div>

            <div className="space-y-5">
              {SCORING_WEIGHT_CONTROLS.map(({ key, title, description }) => (
                <div key={key} className="space-y-2">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <p className="text-sm font-medium text-foreground">{title}</p>
                      <p className="mt-1 text-xs text-muted-foreground">{description}</p>
                    </div>
                    <span className="rounded-full border border-border bg-surface px-2 py-0.5 text-xs font-semibold text-foreground">
                      {scoringWeights[key]}/10
                    </span>
                  </div>
                  <input
                    type="range"
                    min={1}
                    max={10}
                    step={1}
                    value={scoringWeights[key]}
                    disabled={scoringWeightsMutation.isPending}
                    onChange={(event) =>
                      updateScoringWeight(key, Number(event.currentTarget.value))
                    }
                    className="w-full accent-primary"
                  />
                </div>
              ))}
            </div>

            {scoringWeightsMutation.isError && (
              <p className="mt-3 text-sm text-danger">
                Could not save scoring weights. Please try again.
              </p>
            )}
          </div>
        </div>
        <p className="text-sm text-muted-foreground">
          Full search preference editing lives in Profile &amp; Search.
        </p>
        <Button variant="outline" onClick={() => navigate('/profile')}>
          Open Profile &amp; Search
        </Button>
      </div>
    </SettingsSection>
  );
}
