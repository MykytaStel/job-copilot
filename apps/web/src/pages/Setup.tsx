import { useEffect, useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowRight, Check, FileText, SearchCheck, Settings2, Upload, UserRound } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import { runSearch, type SearchRunResult } from '../api/jobs';
import {
  advanceProfileOnboarding,
  getProfileOnboarding,
  type OnboardingStep,
  type ProfileOnboarding,
} from '../api/onboarding';
import {
  analyzeStoredProfile,
  buildSearchProfile,
  getProfile,
  getStoredProfileRawText,
  saveOnboardingResume,
  saveProfileSearchPreferences,
  type PersistedSearchPreferences,
  type SearchTargetRegion,
  type SearchWorkMode,
} from '../api/profiles';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { useToast } from '../context/ToastContext';
import { useResumePicker } from '../features/profile/useResumePicker';
import { hasToken } from '../lib/authSession';
import { cn } from '../lib/cn';
import { readProfileId } from '../lib/profileSession';
import { queryKeys } from '../queryKeys';

const STEPS: { id: Exclude<OnboardingStep, 'complete'>; label: string; icon: typeof FileText }[] = [
  { id: 'cv', label: 'CV', icon: FileText },
  { id: 'profile', label: 'Profile', icon: UserRound },
  { id: 'preferences', label: 'Preferences', icon: Settings2 },
  { id: 'recommendations', label: 'Matches', icon: SearchCheck },
];

const REGION_OPTIONS: { id: SearchTargetRegion; label: string }[] = [
  { id: 'ua', label: 'Ukraine' },
  { id: 'eu_remote', label: 'EU remote' },
  { id: 'eu', label: 'Europe' },
  { id: 'uk', label: 'United Kingdom' },
  { id: 'us', label: 'United States' },
];

const WORK_MODE_OPTIONS: { id: SearchWorkMode; label: string }[] = [
  { id: 'remote', label: 'Remote' },
  { id: 'hybrid', label: 'Hybrid' },
  { id: 'onsite', label: 'On-site' },
];

export default function Setup() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { showToast } = useToast();
  const [rawTextOverride, setRawTextOverride] = useState<string | null>(null);
  const [targetRegionsOverride, setTargetRegionsOverride] = useState<SearchTargetRegion[] | null>(null);
  const [workModesOverride, setWorkModesOverride] = useState<SearchWorkMode[] | null>(null);
  const [searchResult, setSearchResult] = useState<SearchRunResult | null>(null);
  const { fileInputRef, openFilePicker, handleFileChange } = useResumePicker(setRawTextOverride);
  const hasSession = hasToken() && !!readProfileId();

  const profileQuery = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
    enabled: hasSession,
  });
  const profile = profileQuery.data;
  const onboardingQuery = useQuery({
    queryKey: queryKeys.onboarding.profile(profile?.id ?? 'none'),
    queryFn: () => getProfileOnboarding(profile!.id),
    enabled: !!profile?.id,
  });
  const rawTextQuery = useQuery({
    queryKey: queryKeys.profile.rawText(),
    queryFn: getStoredProfileRawText,
    enabled: !!profile?.id,
  });

  const rawText = rawTextOverride ?? rawTextQuery.data ?? '';
  const targetRegions = useMemo<SearchTargetRegion[]>(
    () => targetRegionsOverride
      ?? (profile?.searchPreferences?.targetRegions.length
        ? profile.searchPreferences.targetRegions
        : ['eu_remote']),
    [profile, targetRegionsOverride],
  );
  const workModes = useMemo<SearchWorkMode[]>(
    () => workModesOverride
      ?? (profile?.searchPreferences?.workModes.length
        ? profile.searchPreferences.workModes
        : ['remote']),
    [profile, workModesOverride],
  );

  useEffect(() => {
    if (!profileQuery.isLoading && !profile && !hasSession) {
      navigate('/auth', { replace: true });
    }
  }, [hasSession, navigate, profile, profileQuery.isLoading]);

  function storeOnboarding(next: ProfileOnboarding) {
    queryClient.setQueryData(queryKeys.onboarding.profile(next.profile_id), next);
  }

  const cvMutation = useMutation({
    mutationFn: async () => {
      if (!profile) throw new Error('Profile is not available');
      const updatedProfile = await saveOnboardingResume(profile.id, rawText);
      const onboarding = await advanceProfileOnboarding(profile.id, 'cv');
      return { updatedProfile, onboarding };
    },
    onSuccess: ({ updatedProfile, onboarding }) => {
      queryClient.setQueryData(queryKeys.profile.root(), updatedProfile);
      storeOnboarding(onboarding);
    },
    onError: showMutationError(showToast),
  });

  const profileMutation = useMutation({
    mutationFn: async () => {
      if (!profile) throw new Error('Profile is not available');
      await analyzeStoredProfile();
      return advanceProfileOnboarding(profile.id, 'profile');
    },
    onSuccess: (onboarding) => {
      storeOnboarding(onboarding);
      void queryClient.invalidateQueries({ queryKey: queryKeys.profile.root() });
    },
    onError: showMutationError(showToast),
  });

  const preferences = useMemo<PersistedSearchPreferences>(
    () => ({
      targetRegions,
      workModes,
      preferredRoles: profile?.searchPreferences?.preferredRoles ?? [],
      allowedSources: profile?.searchPreferences?.allowedSources ?? [],
      includeKeywords: profile?.searchPreferences?.includeKeywords ?? [],
      excludeKeywords: profile?.searchPreferences?.excludeKeywords ?? [],
      scoringWeights: profile?.searchPreferences?.scoringWeights,
    }),
    [profile?.searchPreferences, targetRegions, workModes],
  );

  const preferencesMutation = useMutation({
    mutationFn: async () => {
      if (!profile) throw new Error('Profile is not available');
      if (targetRegions.length === 0 || workModes.length === 0) {
        throw new Error('Choose at least one region and work mode');
      }
      const updatedProfile = await saveProfileSearchPreferences(profile.id, preferences);
      const onboarding = await advanceProfileOnboarding(profile.id, 'preferences');
      return { updatedProfile, onboarding };
    },
    onSuccess: ({ updatedProfile, onboarding }) => {
      queryClient.setQueryData(queryKeys.profile.root(), updatedProfile);
      storeOnboarding(onboarding);
    },
    onError: showMutationError(showToast),
  });

  const recommendationsMutation = useMutation({
    mutationFn: async () => {
      const built = await buildSearchProfile({ rawText, preferences });
      return runSearch({ searchProfile: built.searchProfile, limit: 10 });
    },
    onSuccess: setSearchResult,
    onError: showMutationError(showToast),
  });

  const completeMutation = useMutation({
    mutationFn: async () => {
      if (!profile) throw new Error('Profile is not available');
      return advanceProfileOnboarding(profile.id, 'recommendations');
    },
    onSuccess: (onboarding) => {
      storeOnboarding(onboarding);
      navigate('/', { replace: true });
    },
    onError: showMutationError(showToast),
  });

  const onboarding = onboardingQuery.data;
  const currentStep = onboarding?.current_step ?? 'cv';
  const stepIndex = currentStep === 'complete' ? STEPS.length : STEPS.findIndex((step) => step.id === currentStep);

  if (profileQuery.isLoading || onboardingQuery.isLoading) {
    return <Page><div className="mx-auto h-72 max-w-3xl animate-pulse rounded-[var(--radius-lg)] bg-surface-muted" /></Page>;
  }

  if (!profile || !onboarding) return null;

  return (
    <Page>
      <div className="mx-auto max-w-4xl space-y-6 py-4 lg:py-8">
        <header>
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-primary">Setup</p>
          <h1 className="mt-2 text-2xl font-bold text-foreground">Prepare your first job shortlist</h1>
          <p className="mt-2 max-w-2xl text-sm text-muted-foreground">
            Confirm the candidate context Job Copilot will use for ranking and explanations.
          </p>
        </header>

        <ol className="grid list-none grid-cols-4 gap-2" aria-label="Onboarding progress">
          {STEPS.map((step, index) => {
            const Icon = step.icon;
            const done = index < stepIndex || currentStep === 'complete';
            const active = index === stepIndex;
            return (
              <li key={step.id} aria-label={`${step.label}${active ? ', current step' : done ? ', complete' : ''}`} className={cn('min-w-0 border-t-2 pt-3', done || active ? 'border-primary' : 'border-border')}>
                <div className="flex items-center justify-center gap-2 sm:justify-start">
                  <span className={cn('flex h-7 w-7 shrink-0 items-center justify-center rounded-full border', done ? 'border-success/30 bg-success/10 text-success' : active ? 'border-primary/40 bg-primary/10 text-primary' : 'border-border text-muted-foreground')}>
                    {done ? <Check className="h-3.5 w-3.5" /> : <Icon className="h-3.5 w-3.5" />}
                  </span>
                  <span className="hidden truncate text-xs font-medium text-foreground sm:inline">{step.label}</span>
                </div>
              </li>
            );
          })}
        </ol>

        <Card className="border-border bg-card">
          <CardHeader>
            <CardTitle>{stepTitle(currentStep)}</CardTitle>
          </CardHeader>
          <CardContent>
            {currentStep === 'cv' && (
              <div className="space-y-4">
                <p className="text-sm text-muted-foreground">Upload a PDF/TXT/MD file or edit the extracted text. This text remains profile-scoped.</p>
                <input ref={fileInputRef} type="file" accept=".pdf,.txt,.md,.text" className="hidden" onChange={handleFileChange} />
                <div className="flex flex-wrap gap-2">
                  <Button type="button" variant="outline" onClick={openFilePicker}><Upload className="h-4 w-4" />Import CV</Button>
                </div>
                <textarea value={rawText} onChange={(event) => setRawTextOverride(event.target.value)} className="min-h-64 w-full resize-y rounded-[var(--radius-md)] border border-border bg-surface-muted p-3 text-sm leading-6 text-foreground focus:border-primary/60 focus:outline-none focus:ring-1 focus:ring-primary/30" placeholder="Paste your CV or describe your experience and skills" />
                <div className="flex justify-end"><Button onClick={() => cvMutation.mutate()} disabled={!rawText.trim() || cvMutation.isPending}>{cvMutation.isPending ? 'Saving CV' : 'Save and analyze'}<ArrowRight className="h-4 w-4" /></Button></div>
              </div>
            )}

            {currentStep === 'profile' && (
              <div className="space-y-5">
                <div className="rounded-[var(--radius-md)] border border-border bg-surface-muted p-4">
                  <p className="text-sm font-semibold text-foreground">{profile.summary ?? 'Profile analysis is ready to run'}</p>
                  <div className="mt-3 flex flex-wrap gap-2">{profile.skills.slice(0, 12).map((skill) => <Badge key={skill} variant="muted">{skill}</Badge>)}</div>
                </div>
                <p className="text-sm text-muted-foreground">You can refine experience, salary, languages, and links later in Profile.</p>
                <div className="flex justify-end"><Button onClick={() => profileMutation.mutate()} disabled={profileMutation.isPending}>{profileMutation.isPending ? 'Analyzing profile' : 'Confirm profile'}<ArrowRight className="h-4 w-4" /></Button></div>
              </div>
            )}

            {currentStep === 'preferences' && (
              <div className="space-y-6">
                <ChoiceGroup title="Target regions" options={REGION_OPTIONS} selected={targetRegions} onToggle={(value) => setTargetRegionsOverride(toggle(targetRegions, value))} />
                <ChoiceGroup title="Work modes" options={WORK_MODE_OPTIONS} selected={workModes} onToggle={(value) => setWorkModesOverride(toggle(workModes, value))} />
                <div className="flex justify-end"><Button onClick={() => preferencesMutation.mutate()} disabled={preferencesMutation.isPending || targetRegions.length === 0 || workModes.length === 0}>{preferencesMutation.isPending ? 'Saving preferences' : 'Save preferences'}<ArrowRight className="h-4 w-4" /></Button></div>
              </div>
            )}

            {currentStep === 'recommendations' && (
              <div className="space-y-5">
                {!searchResult ? (
                  <EmptyState icon={<SearchCheck className="h-5 w-5" />} message="Generate your first shortlist" description="The engine will rank up to 10 current jobs and explain each result." action={<Button onClick={() => recommendationsMutation.mutate()} disabled={recommendationsMutation.isPending}>{recommendationsMutation.isPending ? 'Ranking jobs' : 'Find my matches'}</Button>} />
                ) : (
                  <div className="space-y-3">
                    {searchResult.results.length === 0 ? <EmptyState message="No current matches" description="Your setup is saved. Adjust preferences later as new jobs arrive." /> : searchResult.results.map(({ job, fit }) => (
                      <div key={job.id} className="flex gap-4 border-b border-border py-3 last:border-0">
                        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-primary/10 text-sm font-bold text-primary">{fit.score}</div>
                        <div className="min-w-0"><p className="truncate text-sm font-semibold text-foreground">{job.title}</p><p className="truncate text-xs text-muted-foreground">{job.company}</p><p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{fit.reasons[0] ?? 'Ranked from your profile and preferences'}</p></div>
                      </div>
                    ))}
                    <div className="flex justify-end pt-2"><Button onClick={() => completeMutation.mutate()} disabled={completeMutation.isPending}>{completeMutation.isPending ? 'Finishing setup' : 'Open Dashboard'}<ArrowRight className="h-4 w-4" /></Button></div>
                  </div>
                )}
              </div>
            )}

            {currentStep === 'complete' && <EmptyState icon={<Check className="h-5 w-5" />} message="Setup complete" action={<Button onClick={() => navigate('/', { replace: true })}>Open Dashboard</Button>} />}
          </CardContent>
        </Card>
      </div>
    </Page>
  );
}

function ChoiceGroup<T extends string>({ title, options, selected, onToggle }: { title: string; options: { id: T; label: string }[]; selected: T[]; onToggle: (value: T) => void }) {
  return <fieldset><legend className="mb-3 text-sm font-semibold text-foreground">{title}</legend><div className="flex flex-wrap gap-2">{options.map((option) => <button key={option.id} type="button" aria-pressed={selected.includes(option.id)} onClick={() => onToggle(option.id)} className={cn('rounded-[var(--radius-md)] border px-3 py-2 text-sm transition-colors', selected.includes(option.id) ? 'border-primary/50 bg-primary/10 text-primary' : 'border-border bg-surface text-muted-foreground hover:text-foreground')}>{option.label}</button>)}</div></fieldset>;
}

function toggle<T>(values: T[], value: T): T[] {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

function stepTitle(step: OnboardingStep) {
  if (step === 'cv') return 'Import your CV';
  if (step === 'profile') return 'Confirm the detected profile';
  if (step === 'preferences') return 'Set your search boundaries';
  if (step === 'recommendations') return 'Review your first matches';
  return 'Setup complete';
}

function showMutationError(showToast: ReturnType<typeof useToast>['showToast']) {
  return (error: unknown) => showToast({ type: 'error', message: error instanceof Error ? error.message : 'Could not save onboarding progress' });
}
