import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import type { JobPosting } from '@job-copilot/shared';
import { getJob, getJobs } from '../api';
import { queryKeys } from '../queryKeys';

export default function Compare() {
  const [idA, setIdA] = useState('');
  const [idB, setIdB] = useState('');

  const { data: allJobs = [], error } = useQuery({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const { data: jobA } = useQuery({
    queryKey: queryKeys.jobs.detail(idA),
    queryFn: () => getJob(idA),
    enabled: !!idA,
  });

  const { data: jobB } = useQuery({
    queryKey: queryKeys.jobs.detail(idB),
    queryFn: () => getJob(idB),
    enabled: !!idB,
  });

  return (
    <div>
      <h1>Compare Jobs</h1>
      <p className="muted" style={{ marginBottom: 16 }}>Select two jobs to view side-by-side.</p>

      {error && <p className="error">{error instanceof Error ? error.message : 'Error'}</p>}

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, marginBottom: 24 }}>
        <div>
          <label style={{ display: 'flex', flexDirection: 'column', gap: 6, fontWeight: 500 }}>
            Job A
            <select value={idA} onChange={(e) => setIdA(e.target.value)}>
              <option value="">— select —</option>
              {allJobs.filter((j) => j.id !== idB).map((j) => (
                <option key={j.id} value={j.id}>{j.title} — {j.company}</option>
              ))}
            </select>
          </label>
        </div>
        <div>
          <label style={{ display: 'flex', flexDirection: 'column', gap: 6, fontWeight: 500 }}>
            Job B
            <select value={idB} onChange={(e) => setIdB(e.target.value)}>
              <option value="">— select —</option>
              {allJobs.filter((j) => j.id !== idA).map((j) => (
                <option key={j.id} value={j.id}>{j.title} — {j.company}</option>
              ))}
            </select>
          </label>
        </div>
      </div>

      {(jobA || jobB) && (
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, alignItems: 'start' }}>
          <JobColumn job={jobA ?? null} side="A" />
          <JobColumn job={jobB ?? null} side="B" />
        </div>
      )}

      {!jobA && !jobB && allJobs.length === 0 && (
        <p className="muted">No jobs saved yet.</p>
      )}
    </div>
  );
}

function JobColumn({ job, side }: { job: JobPosting | null; side: string }) {
  if (!job) {
    return (
      <div className="card" style={{ minHeight: 200, opacity: 0.4, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <span className="muted">Select job {side}</span>
      </div>
    );
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      <h2 style={{ margin: 0, fontSize: 18 }}>{job.title}</h2>
      <p className="muted" style={{ margin: 0 }}>{job.company}</p>
      {job.url && (
        <a href={job.url} target="_blank" rel="noreferrer" style={{ fontSize: 12 }}>
          {job.url}
        </a>
      )}
      <hr style={{ border: 'none', borderTop: '1px solid var(--border)', margin: '4px 0' }} />
      <pre className="jobDescription" style={{ fontSize: 12, maxHeight: 500, overflow: 'auto' }}>
        {job.description}
      </pre>
    </div>
  );
}
