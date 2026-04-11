import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';

import { getJob } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';

export default function JobDetails() {
  const { id } = useParams<{ id: string }>();

  const { data: job, isLoading, error } = useQuery({
    queryKey: queryKeys.jobs.detail(id!),
    queryFn: () => getJob(id!),
    enabled: !!id,
  });

  if (isLoading) return <SkeletonPage />;
  if (!job) {
    return (
      <p className="error">
        {error instanceof Error ? error.message : 'Job not found'}
      </p>
    );
  }

  return (
    <div className="jobDetails">
      <div className="pageHeader">
        <div>
          <h1>{job.title}</h1>
          <p className="muted">{job.company}</p>
        </div>
      </div>

      <section className="card">
        <p className="eyebrow">Description</p>
        <pre className="jobDescription">{job.description}</pre>
      </section>
    </div>
  );
}
