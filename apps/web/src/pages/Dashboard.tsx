import React, { useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { Plus, ArrowRight, X, Bookmark, Send, CalendarDays, Briefcase, XCircle, BarChart2, Clock } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Application, ApplicationStatus, DashboardStats, JobPosting } from '@job-copilot/shared';
import { getApplications, getDashboardStats, getJobs, updateJobNote } from '../api';
import { queryKeys } from '../queryKeys';

const STATUS_COLUMNS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];

const STATUS_ICONS: Record<ApplicationStatus, React.ReactElement> = {
  saved: <Bookmark size={14} />,
  applied: <Send size={14} />,
  interview: <CalendarDays size={14} />,
  offer: <Briefcase size={14} />,
  rejected: <XCircle size={14} />,
};

// Group jobs by ISO week (Mon–Sun), return last N weeks
function weeklyTimeline(jobs: JobPosting[], weeks = 8) {
  const now = new Date();
  const result: { label: string; count: number }[] = [];

  for (let i = weeks - 1; i >= 0; i--) {
    const weekStart = new Date(now);
    weekStart.setDate(now.getDate() - now.getDay() + 1 - i * 7); // Monday
    weekStart.setHours(0, 0, 0, 0);
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekStart.getDate() + 7);

    const count = jobs.filter((j) => {
      const d = new Date(j.createdAt);
      return d >= weekStart && d < weekEnd;
    }).length;

    const label = weekStart.toLocaleDateString('uk-UA', { day: 'numeric', month: 'short' });
    result.push({ label, count });
  }
  return result;
}

export default function Dashboard() {
  const qc = useQueryClient();
  const [search, setSearch] = useState('');

  // Quick notes
  const [editingNoteId, setEditingNoteId] = useState<string | null>(null);
  const [noteValue, setNoteValue] = useState('');
  const noteRef = useRef<HTMLTextAreaElement>(null);

  const { data: jobs = [], error: jobsError } = useQuery<JobPosting[]>({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: stats } = useQuery<DashboardStats>({
    queryKey: queryKeys.dashboard.stats(),
    queryFn: getDashboardStats,
  });

  const noteMutation = useMutation({
    mutationFn: ({ jobId, note }: { jobId: string; note: string }) =>
      updateJobNote(jobId, note),
    onSuccess: (updated) => {
      qc.setQueryData<JobPosting[]>(queryKeys.jobs.all(), (prev = []) =>
        prev.map((j) => (j.id === updated.id ? updated : j)),
      );
      setEditingNoteId(null);
    },
  });

  const applicationForJob = (jobId: string) =>
    applications.find((a) => a.jobId === jobId) ?? null;

  const filteredJobs = search.trim()
    ? jobs.filter(
        (j) =>
          j.title.toLowerCase().includes(search.toLowerCase()) ||
          j.company.toLowerCase().includes(search.toLowerCase()),
      )
    : jobs;

  function startEditNote(job: JobPosting) {
    setEditingNoteId(job.id);
    setNoteValue(job.notes);
    // Focus is handled imperatively after state update
    setTimeout(() => noteRef.current?.focus(), 0);
  }

  function saveNote(jobId: string) {
    noteMutation.mutate({ jobId, note: noteValue });
  }

  const error = jobsError instanceof Error ? jobsError.message : jobsError ? 'Error' : null;
  const timeline = weeklyTimeline(jobs);
  const timelineMax = Math.max(...timeline.map((t) => t.count), 1);

  return (
    <div>
      <div className="pageHeader">
        <h1>Dashboard</h1>
        <Link to="/jobs/new" className="btn" style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}><Plus size={14} /> Add Job</Link>
      </div>

      {error && <p className="error">{error}</p>}

      {stats && (
        <div className="statsGrid">
          {STATUS_COLUMNS.map((status) => (
            <div key={status} className="statCard">
              <div className="statNumber">{stats.byStatus[status] ?? 0}</div>
              <div className="statLabel">
                <span style={{ display: 'flex', alignItems: 'center', gap: 4, justifyContent: 'center' }}>
                  {STATUS_ICONS[status]}
                  {status}
                </span>
              </div>
            </div>
          ))}
          <div className="statCard">
            <div className="statNumber">
              {stats.avgScore !== null ? `${Math.round(stats.avgScore)}%` : '—'}
            </div>
            <div className="statLabel">
              <span style={{ display: 'flex', alignItems: 'center', gap: 4, justifyContent: 'center' }}>
                <BarChart2 size={14} />
                Avg Score
              </span>
            </div>
          </div>
          {stats.tasksDueSoon > 0 && (
            <Link to="/applications" className="statCard" style={{ textDecoration: 'none', color: 'inherit', border: '2px solid #d97706' }}>
              <div className="statNumber" style={{ color: '#d97706' }}>{stats.tasksDueSoon}</div>
              <div className="statLabel">
                <span style={{ display: 'flex', alignItems: 'center', gap: 4, justifyContent: 'center' }}>
                  <Clock size={14} />
                  Tasks due
                </span>
              </div>
            </Link>
          )}
        </div>
      )}

      {/* Weekly timeline */}
      {jobs.length > 0 && (
        <div className="card" style={{ marginBottom: 24 }}>
          <p className="eyebrow" style={{ marginBottom: 12 }}>Jobs added per week</p>
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 6, height: 64 }}>
            {timeline.map((week) => (
              <div key={week.label} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4 }}>
                <div
                  style={{
                    width: '100%',
                    height: `${(week.count / timelineMax) * 52}px`,
                    minHeight: week.count > 0 ? 4 : 0,
                    background: 'var(--accent, #38a169)',
                    borderRadius: 3,
                    transition: 'height 0.3s ease',
                  }}
                />
                <span style={{ fontSize: 10, color: 'var(--muted)', whiteSpace: 'nowrap' }}>
                  {week.count > 0 ? week.count : ''}
                </span>
              </div>
            ))}
          </div>
          <div style={{ display: 'flex', gap: 6, marginTop: 4 }}>
            {timeline.map((week) => (
              <div key={week.label} style={{ flex: 1, fontSize: 9, color: 'var(--muted)', textAlign: 'center', overflow: 'hidden' }}>
                {week.label}
              </div>
            ))}
          </div>
        </div>
      )}

      {jobs.length > 0 && (
        <input
          style={{ marginBottom: 12, padding: '8px 12px', width: '100%', boxSizing: 'border-box' }}
          placeholder="Search by title or company…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      )}

      {jobs.length === 0 ? (
        <p className="muted">No jobs yet. <Link to="/jobs/new">Add the first one →</Link></p>
      ) : filteredJobs.length === 0 ? (
        <p className="muted">No jobs match "{search}".</p>
      ) : (
        <ul className="jobList">
          {filteredJobs.map((job) => {
            const app = applicationForJob(job.id);
            const isEditingNote = editingNoteId === job.id;
            const savingNote = noteMutation.isPending && noteMutation.variables?.jobId === job.id;
            return (
              <li key={job.id} className="jobItem" style={{ flexDirection: 'column', alignItems: 'stretch', gap: 6 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <strong>{job.title}</strong>
                    <p style={{ margin: 0 }}>{job.company}</p>
                  </div>
                  <div className="jobItemRight">
                    {app && (
                      <span className={`statusPill status-${app.status}`}>{app.status}</span>
                    )}
                    <Link to={`/jobs/${job.id}`} className="linkBtn" style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}>Details <ArrowRight size={13} /></Link>
                  </div>
                </div>

                {/* Quick note */}
                {isEditingNote ? (
                  <div style={{ display: 'flex', gap: 8, alignItems: 'flex-start' }}>
                    <textarea
                      ref={noteRef}
                      value={noteValue}
                      onChange={(e) => setNoteValue(e.target.value)}
                      rows={2}
                      style={{ flex: 1, fontSize: 13, resize: 'vertical' }}
                      placeholder="Quick note…"
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) saveNote(job.id);
                        if (e.key === 'Escape') setEditingNoteId(null);
                      }}
                    />
                    <button onClick={() => saveNote(job.id)} disabled={savingNote} style={{ whiteSpace: 'nowrap' }}>
                      {savingNote ? '…' : 'Save'}
                    </button>
                    <button onClick={() => setEditingNoteId(null)} style={{ background: 'transparent', border: '1px solid var(--border)', display: 'inline-flex', alignItems: 'center', gap: 4 }}>
                      <X size={14} />
                    </button>
                  </div>
                ) : (
                  <div
                    onClick={() => startEditNote(job)}
                    style={{ fontSize: 12, color: job.notes ? 'var(--text)' : 'var(--muted)', cursor: 'pointer', padding: '2px 0' }}
                  >
                    {job.notes || '+ add note'}
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
