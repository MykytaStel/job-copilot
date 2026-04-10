import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Download } from 'lucide-react';
import { useQuery, useMutation, useQueries, useQueryClient } from '@tanstack/react-query';
import { DragDropContext, Droppable, Draggable } from '@hello-pangea/dnd';
import type { DropResult } from '@hello-pangea/dnd';
import toast from 'react-hot-toast';
import type { Application, ApplicationStatus, JobPosting, MatchResult } from '@job-copilot/shared';
import { getApplications, getJobs, getMatch, patchApplication, setDueDate } from '../api';
import { queryKeys } from '../queryKeys';

const COLUMNS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];
const ALL = 'all' as const;
type FilterStatus = ApplicationStatus | typeof ALL;

function exportCsv(applications: Application[], jobs: Map<string, JobPosting>) {
  const header = ['Company', 'Title', 'Status', 'Due Date', 'Applied At', 'Updated At'];
  const rows = applications.map((app) => {
    const job = jobs.get(app.jobId);
    return [
      job?.company ?? '',
      job?.title ?? '',
      app.status,
      app.dueDate ?? '',
      app.appliedAt ?? '',
      app.updatedAt,
    ].map((v) => `"${String(v).replace(/"/g, '""')}"`);
  });
  const csv = [header, ...rows].map((r) => r.join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'applications.csv';
  a.click();
  URL.revokeObjectURL(url);
}

function isDueSoon(dueDate?: string): boolean {
  if (!dueDate) return false;
  const diff = new Date(dueDate).getTime() - Date.now();
  return diff > 0 && diff < 48 * 60 * 60 * 1000;
}

function isOverdue(dueDate?: string): boolean {
  if (!dueDate) return false;
  return new Date(dueDate).getTime() < Date.now();
}

export default function ApplicationBoard() {
  const qc = useQueryClient();
  const [filter, setFilter] = useState<FilterStatus>(ALL);
  const [editingDue, setEditingDue] = useState<string | null>(null);

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: jobList = [] } = useQuery<JobPosting[]>({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const jobs = new Map(jobList.map((j) => [j.id, j]));

  // Fetch match for each application in parallel
  const matchQueries = useQueries({
    queries: applications.map((app) => ({
      queryKey: queryKeys.match.forJob(app.jobId),
      queryFn: () => getMatch(app.jobId),
      // Silently ignore — match may not exist yet
      throwOnError: false,
    })),
  });

  const matches = new Map<string, MatchResult>();
  applications.forEach((app, i) => {
    const result = matchQueries[i]?.data;
    if (result) matches.set(app.jobId, result);
  });

  const patchMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: ApplicationStatus }) =>
      patchApplication(id, status),
    onMutate: async ({ id, status }) => {
      await qc.cancelQueries({ queryKey: queryKeys.applications.all() });
      const prev = qc.getQueryData<Application[]>(queryKeys.applications.all());
      qc.setQueryData<Application[]>(queryKeys.applications.all(), (old = []) =>
        old.map((a) => (a.id === id ? { ...a, status } : a)),
      );
      return { prev };
    },
    onError: (err, _vars, ctx) => {
      if (ctx?.prev) {
        qc.setQueryData(queryKeys.applications.all(), ctx.prev);
      }
      toast.error(err instanceof Error ? err.message : 'Failed to update status');
    },
    onSuccess: (_updated, { status }) => {
      toast.success(`Moved to ${status}`);
    },
    onSettled: () => {
      void qc.invalidateQueries({ queryKey: queryKeys.applications.all() });
    },
  });

  const dueDateMutation = useMutation({
    mutationFn: ({ appId, value }: { appId: string; value: string }) =>
      setDueDate(appId, value || null),
    onSuccess: (updated) => {
      qc.setQueryData<Application[]>(queryKeys.applications.all(), (prev = []) =>
        prev.map((a) => (a.id === updated.id ? updated : a)),
      );
      setEditingDue(null);
    },
    onError: (e) => {
      toast.error(e instanceof Error ? e.message : 'Failed to set due date');
      setEditingDue(null);
    },
  });

  function onDragEnd(result: DropResult) {
    const { destination, source, draggableId } = result;
    if (!destination) return;
    if (destination.droppableId === source.droppableId) return;

    const newStatus = destination.droppableId as ApplicationStatus;
    patchMutation.mutate({ id: draggableId, status: newStatus });
  }

  const visibleApps =
    filter === ALL ? applications : applications.filter((a) => a.status === filter);
  const byStatus = (status: ApplicationStatus) =>
    visibleApps.filter((a) => a.status === status);

  return (
    <div>
      <div className="pageHeader">
        <h1>Applications</h1>
        {applications.length > 0 && (
          <button
            onClick={() => exportCsv(applications, jobs)}
            style={{ whiteSpace: 'nowrap', display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            <Download size={14} /> Export CSV
          </button>
        )}
      </div>

      {/* Status filter tabs */}
      {applications.length > 0 && (
        <div style={{ display: 'flex', gap: 6, marginBottom: 16, flexWrap: 'wrap' }}>
          {([ALL, ...COLUMNS] as FilterStatus[]).map((s) => {
            const count =
              s === ALL
                ? applications.length
                : applications.filter((a) => a.status === s).length;
            return (
              <button
                key={s}
                onClick={() => setFilter(s)}
                style={{
                  padding: '4px 12px',
                  borderRadius: 20,
                  border: '1px solid var(--border)',
                  background:
                    filter === s ? 'var(--accent, #38a169)' : 'transparent',
                  color: filter === s ? '#fff' : 'inherit',
                  cursor: 'pointer',
                  fontSize: 13,
                }}
              >
                {s} ({count})
              </button>
            );
          })}
        </div>
      )}

      {applications.length === 0 ? (
        <p className="muted">
          No applications yet. <Link to="/">Go to Dashboard →</Link>
        </p>
      ) : (
        <DragDropContext onDragEnd={onDragEnd}>
          <div className="board">
            {COLUMNS.map((col) => {
              const cards = byStatus(col);
              if (filter !== ALL && filter !== col) return null;
              return (
                <Droppable droppableId={col} key={col}>
                  {(provided, snapshot) => (
                    <div
                      className="boardCol"
                      ref={provided.innerRef}
                      {...provided.droppableProps}
                      style={{
                        background: snapshot.isDraggingOver
                          ? 'var(--surface, #f0f4f8)'
                          : undefined,
                        transition: 'background 0.15s ease',
                        borderRadius: 8,
                      }}
                    >
                      <p className={`colHeader status-${col}`}>
                        {col} <span className="colCount">{cards.length}</span>
                      </p>

                      {cards.map((app, index) => {
                        const job = jobs.get(app.jobId);
                        const score = matches.get(app.jobId)?.score ?? null;
                        const dueSoon = isDueSoon(app.dueDate);
                        const overdue = isOverdue(app.dueDate);

                        return (
                          <Draggable
                            key={app.id}
                            draggableId={app.id}
                            index={index}
                          >
                            {(provided, snapshot) => (
                              <div
                                className="boardCard"
                                ref={provided.innerRef}
                                {...provided.draggableProps}
                                {...provided.dragHandleProps}
                                style={{
                                  ...provided.draggableProps.style,
                                  opacity: snapshot.isDragging ? 0.85 : 1,
                                  boxShadow: snapshot.isDragging
                                    ? '0 8px 24px rgba(0,0,0,0.18)'
                                    : undefined,
                                }}
                              >
                                <Link
                                  to={`/applications/${app.id}`}
                                  className="boardCardTitle"
                                >
                                  {job?.title ?? 'Unknown job'}
                                </Link>
                                <p className="muted boardCardCompany">
                                  {job?.company}
                                </p>

                                {score !== null && (
                                  <span
                                    className="statusPill"
                                    style={{
                                      marginBottom: 6,
                                      display: 'inline-block',
                                    }}
                                  >
                                    {score}% fit
                                  </span>
                                )}

                                {/* Due date */}
                                {editingDue === app.id ? (
                                  <input
                                    type="date"
                                    defaultValue={app.dueDate ?? ''}
                                    autoFocus
                                    style={{
                                      fontSize: 12,
                                      marginBottom: 6,
                                      width: '100%',
                                    }}
                                    onBlur={(e) =>
                                      dueDateMutation.mutate({ appId: app.id, value: e.target.value })
                                    }
                                    onKeyDown={(e) => {
                                      if (e.key === 'Escape') setEditingDue(null);
                                    }}
                                  />
                                ) : (
                                  <div
                                    onClick={() => setEditingDue(app.id)}
                                    style={{
                                      fontSize: 12,
                                      marginBottom: 6,
                                      cursor: 'pointer',
                                      color: overdue
                                        ? 'var(--danger, #e53e3e)'
                                        : dueSoon
                                        ? '#d97706'
                                        : 'var(--muted)',
                                    }}
                                  >
                                    {app.dueDate
                                      ? `${overdue ? 'Overdue: ' : dueSoon ? 'Due soon: ' : 'Due: '}${new Date(app.dueDate).toLocaleDateString('uk-UA')}`
                                      : '+ due date'}
                                  </div>
                                )}
                              </div>
                            )}
                          </Draggable>
                        );
                      })}
                      {provided.placeholder}
                    </div>
                  )}
                </Droppable>
              );
            })}
          </div>
        </DragDropContext>
      )}
    </div>
  );
}
