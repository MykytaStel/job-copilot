import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import toast from 'react-hot-toast';
import type { ApplicationDetail as ApplicationDetailType } from '@job-copilot/shared';
import { getApplicationDetail } from '../api';

function fmt(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}

function StatusBadge({ status }: { status: string }) {
  return (
    <span className={`statusPill status-${status}`}>{status}</span>
  );
}

function SectionHeader({ title }: { title: string }) {
  return <p className="eyebrow" style={{ marginTop: 0 }}>{title}</p>;
}

function EmptyState({ message }: { message: string }) {
  return <p className="muted" style={{ margin: 0, fontStyle: 'italic' }}>{message}</p>;
}

function DescriptionBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const limit = 600;
  const shouldTruncate = text.length > limit;
  const displayed = expanded || !shouldTruncate ? text : text.slice(0, limit) + '…';

  return (
    <div>
      <pre className="jobDescription">{displayed}</pre>
      {shouldTruncate && (
        <button
          onClick={() => setExpanded((v) => !v)}
          style={{
            marginTop: 8,
            padding: '6px 12px',
            fontSize: 12,
            background: 'rgba(255,255,255,0.07)',
            color: '#9aa8bc',
            borderRadius: 8,
          }}
        >
          {expanded ? 'Show less' : 'Show more'}
        </button>
      )}
    </div>
  );
}

export default function ApplicationDetail() {
  const { id } = useParams<{ id: string }>();
  const [detail, setDetail] = useState<ApplicationDetailType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    setLoading(true);
    getApplicationDetail(id)
      .then(setDetail)
      .catch((err) => {
        const msg = err instanceof Error ? err.message : 'Failed to load application';
        setError(msg);
        toast.error(msg);
      })
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) return <p className="muted">Loading…</p>;
  if (error || !detail) {
    return <p className="error">{error ?? 'Application not found'}</p>;
  }

  const { job } = detail;

  return (
    <div className="jobDetails">
      {/* Header */}
      <div className="pageHeader" style={{ alignItems: 'flex-start' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <Link to="/applications" className="linkBtn" style={{ marginBottom: 4 }}>
            &larr; Back to Board
          </Link>
          <h1 style={{ margin: 0 }}>{job.title}</h1>
          <p className="muted" style={{ margin: 0 }}>{job.company}</p>
          <div style={{ display: 'flex', gap: 10, alignItems: 'center', marginTop: 4 }}>
            <StatusBadge status={detail.status} />
            {detail.appliedAt && (
              <span className="muted" style={{ fontSize: 13 }}>
                Applied: {fmt(detail.appliedAt)}
              </span>
            )}
            {detail.dueDate && (
              <span className="muted" style={{ fontSize: 13 }}>
                Due: {fmt(detail.dueDate)}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Job details */}
      <section className="card">
        <SectionHeader title="Job Details" />
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px 16px', marginBottom: 16 }}>
          {job.url && (
            <span className="muted" style={{ fontSize: 13 }}>
              Source:{' '}
              <a href={job.url} target="_blank" rel="noopener noreferrer" className="linkBtn">
                {job.url}
              </a>
            </span>
          )}
          <span className="muted" style={{ fontSize: 13 }}>
            Posted: {fmt(job.createdAt)}
          </span>
        </div>
        <DescriptionBlock text={job.description || 'No description available.'} />
      </section>

      {/* Notes */}
      <section className="card">
        <SectionHeader title="Notes" />
        {detail.notes.length === 0 ? (
          <EmptyState message="No notes yet" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {detail.notes.map((note) => (
              <div
                key={note.id}
                style={{
                  background: 'rgba(255,255,255,0.04)',
                  borderRadius: 12,
                  padding: '12px 14px',
                }}
              >
                <p style={{ margin: '0 0 6px', fontSize: 14, lineHeight: 1.6 }}>{note.content}</p>
                <span className="muted" style={{ fontSize: 12 }}>{fmt(note.createdAt)}</span>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Contacts */}
      <section className="card">
        <SectionHeader title="Contacts" />
        {detail.contacts.length === 0 ? (
          <EmptyState message="No contacts yet" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {detail.contacts.map((ac) => {
              const c = ac.contact;
              return (
                <div
                  key={ac.id}
                  style={{
                    background: 'rgba(255,255,255,0.04)',
                    borderRadius: 12,
                    padding: '12px 14px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 4,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    <span style={{ fontWeight: 600, fontSize: 14 }}>{c.name}</span>
                    <span className="badge badge-secondary" style={{ fontSize: 11, padding: '3px 8px' }}>
                      {ac.relationship.replace('_', ' ')}
                    </span>
                  </div>
                  {(c.role || c.company) && (
                    <span className="muted" style={{ fontSize: 13 }}>
                      {[c.role, c.company].filter(Boolean).join(' at ')}
                    </span>
                  )}
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '2px 12px', marginTop: 2 }}>
                    {c.email && (
                      <a href={`mailto:${c.email}`} className="linkBtn" style={{ fontSize: 13 }}>
                        {c.email}
                      </a>
                    )}
                    {c.phone && (
                      <span className="muted" style={{ fontSize: 13 }}>{c.phone}</span>
                    )}
                    {c.linkedinUrl && (
                      <a href={c.linkedinUrl} target="_blank" rel="noopener noreferrer" className="linkBtn" style={{ fontSize: 13 }}>
                        LinkedIn
                      </a>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </section>

      {/* Activities */}
      <section className="card">
        <SectionHeader title="Activities" />
        {detail.activities.length === 0 ? (
          <EmptyState message="No activities yet" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {detail.activities.map((activity) => (
              <div
                key={activity.id}
                style={{
                  display: 'flex',
                  gap: 12,
                  alignItems: 'flex-start',
                  paddingBottom: 10,
                  borderBottom: '1px solid rgba(255,255,255,0.05)',
                }}
              >
                <span
                  className="badge"
                  style={{ fontSize: 11, padding: '3px 8px', whiteSpace: 'nowrap', flexShrink: 0 }}
                >
                  {activity.type.replace('_', ' ')}
                </span>
                <div style={{ flex: 1 }}>
                  <p style={{ margin: '0 0 4px', fontSize: 14 }}>{activity.description}</p>
                  <span className="muted" style={{ fontSize: 12 }}>{fmt(activity.happenedAt)}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Tasks */}
      <section className="card">
        <SectionHeader title="Tasks" />
        {detail.tasks.length === 0 ? (
          <EmptyState message="No tasks yet" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {detail.tasks.map((task) => (
              <div
                key={task.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  padding: '10px 12px',
                  background: 'rgba(255,255,255,0.04)',
                  borderRadius: 10,
                  opacity: task.done ? 0.55 : 1,
                }}
              >
                <input
                  type="checkbox"
                  checked={task.done}
                  readOnly
                  style={{ accentColor: '#536dfe', width: 16, height: 16, flexShrink: 0 }}
                />
                <span
                  style={{
                    flex: 1,
                    fontSize: 14,
                    textDecoration: task.done ? 'line-through' : 'none',
                    color: task.done ? '#9aa8bc' : '#e9eef6',
                  }}
                >
                  {task.title}
                </span>
                {task.remindAt && (
                  <span className="muted" style={{ fontSize: 12, whiteSpace: 'nowrap' }}>
                    Remind: {fmt(task.remindAt)}
                  </span>
                )}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
