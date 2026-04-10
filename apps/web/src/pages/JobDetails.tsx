import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Trash2, Zap } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { Application } from '@job-copilot/shared';
import { createApplication, deleteJob, getJob, getMatch, patchApplication, runMatch } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';

export default function JobDetails() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [application, setApplication] = useState<Application | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { data: job, isLoading } = useQuery({
    queryKey: queryKeys.jobs.detail(id!),
    queryFn: () => getJob(id!),
    enabled: !!id,
  });

  const { data: match } = useQuery({
    queryKey: queryKeys.match.forJob(id!),
    queryFn: () => getMatch(id!).catch(() => null),
    enabled: !!id,
  });

  const runMatchMutation = useMutation({
    mutationFn: () => runMatch(id!),
    onSuccess: (result) => {
      queryClient.setQueryData(queryKeys.match.forJob(id!), result);
      setError(null);
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'Error'),
  });

  const saveMutation = useMutation({
    mutationFn: () => createApplication({ jobId: id!, status: 'saved' }),
    onSuccess: (app) => {
      setApplication(app);
      queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'Error'),
  });

  const deleteMutation = useMutation({
    mutationFn: () => deleteJob(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      navigate('/');
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'Error'),
  });

  const statusMutation = useMutation({
    mutationFn: (status: Application['status']) => patchApplication(application!.id, status),
    onSuccess: (updated) => setApplication(updated),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'Error'),
  });

  if (isLoading || !job) return <SkeletonPage />;

  return (
    <div className="jobDetails">
      <div className="pageHeader">
        <div>
          <h1>{job.title}</h1>
          <p className="muted">{job.company}</p>
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        {!application ? (
          <button onClick={() => saveMutation.mutate()}>Save to board</button>
        ) : (
          <select
            className="statusSelect"
            value={application.status}
            onChange={(e) => statusMutation.mutate(e.target.value as Application['status'])}
          >
            {(['saved', 'applied', 'interview', 'offer', 'rejected'] as const).map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        )}
        <button
          onClick={() => {
            if (!confirm('Delete this job and all related data?')) return;
            deleteMutation.mutate();
          }}
          style={{ background: 'transparent', color: 'var(--danger, #e53e3e)', border: '1px solid currentColor', padding: '6px 12px', display: 'inline-flex', alignItems: 'center', gap: 4 }}
        >
          <Trash2 size={14} /> Delete
        </button>
        </div>
      </div>

      {error && <p className="error">{error}</p>}

      {/* Fit Score */}
      <section className="card matchCard">
        <div className="matchHeader">
          <h2>Fit Score</h2>
          <button onClick={() => runMatchMutation.mutate()} disabled={runMatchMutation.isPending} style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}>
            <Zap size={14} /> {runMatchMutation.isPending ? 'Scoring…' : match ? 'Re-score' : 'Run Score'}
          </button>
        </div>

        {match ? (
          <div className="matchBody">
            <div className="scoreCircle" style={{ '--score': match.score } as React.CSSProperties}>
              <span className="scoreNumber">{match.score}%</span>
            </div>
            <div className="matchLists">
              <div>
                <p className="matchLabel matched">Matched ({match.matchedSkills.length})</p>
                <ul className="skillList">
                  {match.matchedSkills.map((s) => <li key={s} className="pill">{s}</li>)}
                </ul>
              </div>
              <div>
                <p className="matchLabel missing">Missing ({match.missingSkills.length})</p>
                <ul className="skillList">
                  {match.missingSkills.map((s) => <li key={s} className="pill pill-missing">{s}</li>)}
                </ul>
              </div>
            </div>
            <p className="muted">{match.notes}</p>
          </div>
        ) : (
          <p className="muted">Натисни «Run Score» щоб порівняти з резюме.</p>
        )}
      </section>

      {/* Description */}
      <section className="card">
        <h2>Description</h2>
        <pre className="jobDescription">{job.description}</pre>
      </section>
    </div>
  );
}
