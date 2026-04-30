import { useMemo } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { Application, ApplicationStatus, JobPosting } from '@job-copilot/shared';

import { useToast } from '../../context/ToastContext';

import { getApplications, patchApplication } from '../../api/applications';
import { getJobs } from '../../api/jobs';
import { queryKeys } from '../../queryKeys';

function exportCsv(applications: Application[], jobs: Map<string, JobPosting>) {
  const header = ['Company', 'Title', 'Status', 'Applied At', 'Updated At'];

  const rows = applications.map((application) => {
    const job = jobs.get(application.jobId);

    return [
      job?.company ?? '',
      job?.title ?? '',
      application.status,
      application.appliedAt ?? '',
      application.updatedAt,
    ].map((value) => `"${String(value).replace(/"/g, '""')}"`);
  });

  const csv = [header, ...rows].map((row) => row.join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');

  anchor.href = url;
  anchor.download = 'applications.csv';
  anchor.click();

  URL.revokeObjectURL(url);
}

export function useApplicationBoard() {
  const queryClient = useQueryClient();
  const { showToast } = useToast();

  const {
    data: applications = [],
    error,
    isLoading: applicationsLoading,
  } = useQuery({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: jobs = [], isLoading: jobsLoading } = useQuery({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const jobsById = useMemo(
    () => new Map(jobs.map((job) => [job.id, job])),
    [jobs],
  );

  const rejectedCount = applications.filter(
    (application) => application.status === 'rejected',
  ).length;

  const activeCount = applications.length - rejectedCount;

  const latestUpdatedAt = applications
    .slice()
    .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))[0]?.updatedAt;

  const moveMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: ApplicationStatus }) =>
      patchApplication(id, status),
    onSuccess: (updated) => {
      queryClient.setQueryData(
        queryKeys.applications.all(),
        (prev: Application[] | undefined) =>
          prev?.map((item) => (item.id === updated.id ? updated : item)) ?? [updated],
      );

      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
    },
    onError: (value: unknown) => {
      showToast({ type: 'error', message: value instanceof Error ? value.message : 'Помилка' });
    },
  });

  return {
    applications,
    jobsById,
    activeCount,
    latestUpdatedAt,
    error,
    isLoading: applicationsLoading || jobsLoading,
    moveMutation,
    exportCsv: () => exportCsv(applications, jobsById),
  };
}
