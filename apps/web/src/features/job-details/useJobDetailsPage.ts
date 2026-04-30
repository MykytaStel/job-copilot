import { useEffect, useRef, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Application } from '@job-copilot/shared';
import type {
  JobFeedbackReason,
  LegitimacySignal,
  SalaryFeedbackSignal,
  WorkModeFeedbackSignal,
} from '@job-copilot/shared/feedback';

import { createApplication, getApplications } from '../../api/applications';
import {
  addCompanyBlacklist,
  addCompanyWhitelist,
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  setJobInterestRating,
  setJobLegitimacySignal,
  setJobSalarySignal,
  setJobWorkModeSignal,
  tagJobFeedback,
  unhideJob,
  unmarkJobBadFit,
  unsaveJob,
} from '../../api/feedback';
import {
  getCoverLetterDraft,
  getInterviewPrep,
  getJobFitExplanation,
  getResumeMatch,
  type CoverLetterDraft,
  type InterviewPrep,
  type JobFitExplanation,
  type ResumeMatch,
} from '../../api/enrichment';
import type { FitAnalysis } from '../../api/jobs';
import { analyzeFit, getJob } from '../../api/jobs';
import { getActiveResume } from '../../api/profiles';
import { fireEvent } from '../../api/events';
import {
  invalidateApplicationSummaryQueries,
  invalidateFeedbackQueries,
  invalidateJobAiQueries,
  invalidateJobQueries,
} from '../../lib/queryInvalidation';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';

export function useJobDetailsPage() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();
  const profileId = readProfileId();
  const [activeTab, setActiveTab] = useState<'overview' | 'match' | 'ai' | 'lifecycle'>('overview');
  const [copied, setCopied] = useState(false);
  const [generateCoverLetter, setGenerateCoverLetter] = useState(false);
  const [generateInterviewPrep, setGenerateInterviewPrep] = useState(false);
  const scrollFiredRef = useRef(false);

  const {
    data: job,
    isLoading,
    error,
  } = useQuery({
    queryKey: queryKeys.jobs.detail(id!, profileId),
    queryFn: () => getJob(id!),
    enabled: !!id,
  });

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  useEffect(() => {
    if (!profileId || !job?.id) {
      return;
    }

    fireEvent(profileId, {
      eventType: 'job_opened',
      jobId: job.id,
      payloadJson: { surface: 'job_details' },
    });

    const returnedKey = `job_visited_${job.id}`;
    if (sessionStorage.getItem(returnedKey)) {
      fireEvent(profileId, { eventType: 'job_returned', jobId: job.id });
    } else {
      sessionStorage.setItem(returnedKey, '1');
    }
  }, [job?.id, profileId]);

  useEffect(() => {
    if (!profileId || !id) return;

    scrollFiredRef.current = false;

    function handleScroll() {
      if (scrollFiredRef.current) return;
      const scrolledPct =
        (window.scrollY + window.innerHeight) / document.documentElement.scrollHeight;
      if (scrolledPct >= 0.9) {
        scrollFiredRef.current = true;
        fireEvent(profileId!, {
          eventType: 'job_scrolled_to_bottom',
          jobId: id!,
        });
      }
    }

    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => window.removeEventListener('scroll', handleScroll);
  }, [id, profileId]);

  const { data: fit } = useQuery<FitAnalysis>({
    queryKey: queryKeys.ml.fit(profileId ?? '', id!),
    queryFn: () => analyzeFit(profileId!, id!),
    enabled: !!profileId && !!id,
    staleTime: 2 * 60_000,
    retry: false,
  });

  const { data: activeResume } = useQuery({
    queryKey: queryKeys.resumes.active(),
    queryFn: getActiveResume,
    enabled: activeTab === 'match' && !!profileId,
    staleTime: 5 * 60_000,
    retry: false,
  });

  const {
    data: resumeMatch,
    isLoading: resumeMatchLoading,
    error: resumeMatchError,
  } = useQuery<ResumeMatch>({
    queryKey: queryKeys.ml.resumeMatch(profileId ?? '', id ?? '', activeResume?.id ?? 'none'),
    queryFn: () =>
      getResumeMatch({
        resumeText: activeResume!.rawText,
        jdText: `${job!.title}\n${job!.description}`,
      }),
    enabled:
      activeTab === 'match' && !!profileId && !!id && !!job?.description && !!activeResume?.rawText,
    staleTime: 10 * 60_000,
    retry: false,
  });

  const deterministicFit =
    fit && job
      ? {
          jobId: fit.jobId,
          score: fit.score,
          scoreBreakdown: fit.scoreBreakdown,
          matchedRoles: fit.matchedRoles,
          matchedSkills: fit.matchedSkills,
          matchedKeywords: fit.matchedKeywords,
          missingSignals: fit.missingTerms,
          sourceMatch: false,
          workModeMatch: undefined,
          regionMatch: undefined,
          descriptionQuality: fit.descriptionQuality,
          positiveReasons: fit.positiveReasons,
          negativeReasons: fit.negativeReasons,
          reasons: [...fit.positiveReasons, ...fit.negativeReasons],
        }
      : null;

  const { data: fitExplanation, isLoading: fitExplanationLoading } = useQuery<JobFitExplanation>({
    queryKey: queryKeys.ml.fitExplanation(profileId ?? '', id ?? ''),
    queryFn: () =>
      getJobFitExplanation({
        profileId: profileId!,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: job!,
        deterministicFit: deterministicFit!,
      }),
    enabled: activeTab === 'ai' && !!profileId && !!deterministicFit,
    staleTime: 10 * 60_000,
    retry: false,
  });

  const { data: coverLetter, isLoading: coverLetterLoading } = useQuery<CoverLetterDraft>({
    queryKey: queryKeys.ml.coverLetter(profileId ?? '', id ?? ''),
    queryFn: () =>
      getCoverLetterDraft({
        profileId: profileId!,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: job!,
        deterministicFit: deterministicFit!,
        jobFitExplanation: fitExplanation ?? null,
      }),
    enabled: generateCoverLetter && !!profileId && !!deterministicFit,
    staleTime: 30 * 60_000,
    retry: false,
  });

  const { data: interviewPrep, isLoading: interviewPrepLoading } = useQuery<InterviewPrep>({
    queryKey: queryKeys.ml.interviewPrep(profileId ?? '', id ?? ''),
    queryFn: () =>
      getInterviewPrep({
        profileId: profileId!,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: job!,
        deterministicFit: deterministicFit!,
        jobFitExplanation: fitExplanation ?? null,
      }),
    enabled: generateInterviewPrep && !!profileId && !!deterministicFit,
    staleTime: 30 * 60_000,
    retry: false,
  });

  const existing = applications.find((application) => application.jobId === id);
  const isSaved = job?.feedback?.saved || !!existing;
  const isHidden = job?.feedback?.hidden;
  const isBadFit = job?.feedback?.badFit;
  const companyStatus = job?.feedback?.companyStatus;

  const invalidateCurrentJobQueries = () => {
    void invalidateJobQueries(queryClient, profileId, id);
    void invalidateJobAiQueries(queryClient, profileId, id);
  };

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      await markJobSaved(profileId, id!);
      if (!existing) {
        await createApplication({ jobId: id!, status: 'saved' });
      }
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      void invalidateApplicationSummaryQueries(queryClient);
      toast.success('Збережено в pipeline');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const unsaveMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      await unsaveJob(profileId, id!);
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      toast.success('Знято з обраного');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const hideMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      await hideJobForProfile(profileId, id!);
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      toast.success('Вакансію приховано');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const unhideMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      await unhideJob(profileId, id!);
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      toast.success('Вакансію показано');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const badFitMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      await markJobBadFit(profileId, id!);
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      toast.success('Позначено як bad fit');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const unmarkBadFitMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }
      await unmarkJobBadFit(profileId, id!);
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      toast.success('Позначку bad fit знято');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const companyFeedbackMutation = useMutation({
    mutationFn: async (nextStatus: 'whitelist' | 'blacklist') => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      if (nextStatus === 'whitelist') {
        if (companyStatus === 'whitelist') {
          await removeCompanyWhitelist(profileId, job!.company);
        } else {
          await addCompanyWhitelist(profileId, job!.company);
        }
        return;
      }

      if (companyStatus === 'blacklist') {
        await removeCompanyBlacklist(profileId, job!.company);
      } else {
        await addCompanyBlacklist(profileId, job!.company);
      }
    },
    onSuccess: (_result, nextStatus) => {
      const nextCompanyStatus = companyStatus === nextStatus ? undefined : nextStatus;

      queryClient.setQueryData(queryKeys.jobs.detail(id!, profileId), (current: typeof job) =>
        current
          ? {
              ...current,
              feedback: {
                ...current.feedback,
                companyStatus: nextCompanyStatus,
              },
            }
          : current,
      );
      void invalidateFeedbackQueries(queryClient, profileId);
      toast.success('Оновлено список компанії');
    },
    onError: (value: unknown) => {
      toast.error(value instanceof Error ? value.message : 'Помилка');
    },
  });

  const handleCopy = async () => {
    if (typeof window === 'undefined' || !navigator.clipboard) {
      return;
    }

    await navigator.clipboard.writeText(window.location.href);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);

    if (profileId && id) {
      fireEvent(profileId, { eventType: 'job_shared', jobId: id });
    }
  };

  const interestRatingMutation = useMutation({
    mutationFn: async (rating: number) => {
      if (!profileId) throw new Error('Create a profile first');
      await setJobInterestRating(profileId, id!, rating);
    },
    onSuccess: () => invalidateCurrentJobQueries(),
    onError: (value: unknown) => toast.error(value instanceof Error ? value.message : 'Помилка'),
  });

  const salarySignalMutation = useMutation({
    mutationFn: async (signal: SalaryFeedbackSignal) => {
      if (!profileId) throw new Error('Create a profile first');
      await setJobSalarySignal(profileId, id!, signal);
    },
    onSuccess: () => invalidateCurrentJobQueries(),
    onError: (value: unknown) => toast.error(value instanceof Error ? value.message : 'Помилка'),
  });

  const workModeMutation = useMutation({
    mutationFn: async (signal: WorkModeFeedbackSignal) => {
      if (!profileId) throw new Error('Create a profile first');
      await setJobWorkModeSignal(profileId, id!, signal);
    },
    onSuccess: () => invalidateCurrentJobQueries(),
    onError: (value: unknown) => toast.error(value instanceof Error ? value.message : 'Помилка'),
  });

  const tagsMutation = useMutation({
    mutationFn: async (tags: JobFeedbackReason[]) => {
      if (!profileId) throw new Error('Create a profile first');
      await tagJobFeedback(profileId, id!, tags);
    },
    onSuccess: () => invalidateCurrentJobQueries(),
    onError: (value: unknown) => toast.error(value instanceof Error ? value.message : 'Помилка'),
  });

  const legitimacyMutation = useMutation({
    mutationFn: async (signal: LegitimacySignal) => {
      if (!profileId) throw new Error('Create a profile first');
      await setJobLegitimacySignal(profileId, id!, signal);
    },
    onSuccess: () => {
      invalidateCurrentJobQueries();
      toast.success('Дякуємо за відгук');
    },
    onError: (value: unknown) => toast.error(value instanceof Error ? value.message : 'Помилка'),
  });

  return {
    id,
    profileId,
    job,
    isLoading,
    error,
    applications,
    fit,
    activeResume,
    resumeMatch,
    resumeMatchLoading,
    resumeMatchError,
    deterministicFit,
    fitExplanation,
    fitExplanationLoading,
    coverLetter,
    coverLetterLoading,
    interviewPrep,
    interviewPrepLoading,
    existing,
    isSaved,
    isHidden,
    isBadFit,
    companyStatus,
    activeTab,
    setActiveTab,
    copied,
    handleCopy,
    generateCoverLetter,
    setGenerateCoverLetter,
    generateInterviewPrep,
    setGenerateInterviewPrep,
    saveMutation,
    unsaveMutation,
    hideMutation,
    unhideMutation,
    badFitMutation,
    unmarkBadFitMutation,
    companyFeedbackMutation,
    interestRatingMutation,
    salarySignalMutation,
    workModeMutation,
    tagsMutation,
    legitimacyMutation,
  };
}

export type JobDetailsPageState = ReturnType<typeof useJobDetailsPage>;
