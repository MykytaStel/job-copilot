import { useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { X, ArrowLeft, Plus } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type {
  ActivityType,
  ApplicationNote,
  ApplicationStatus,
  ContactRelationship,
} from '@job-copilot/shared';
import toast from 'react-hot-toast';
import {
  addNote,
  createActivity,
  createContact,
  createTask,
  deleteActivity,
  deleteTask,
  getApplicationDetail,
  getContacts,
  getMatch,
  linkContact,
  patchApplication,
  patchTask,
  unlinkContact,
} from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';

const STATUSES: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];
const ACTIVITY_TYPES: ActivityType[] = ['email', 'call', 'interview', 'follow_up', 'note', 'other'];
const ACTIVITY_LABELS: Record<ActivityType, string> = {
  email: 'Email',
  call: 'Call',
  interview: 'Interview',
  follow_up: 'Follow-up',
  note: 'Note',
  other: 'Other',
};
const RELATIONSHIPS: ContactRelationship[] = ['recruiter', 'hiring_manager', 'interviewer', 'other'];

export default function ApplicationDetail() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();

  // Notes
  const [noteText, setNoteText] = useState('');

  // Contacts
  const [showContactForm, setShowContactForm] = useState(false);
  const [newContactName, setNewContactName] = useState('');
  const [newContactEmail, setNewContactEmail] = useState('');
  const [newContactRole, setNewContactRole] = useState('');
  const [newContactRel, setNewContactRel] = useState<ContactRelationship>('recruiter');
  const [linkingContactId, setLinkingContactId] = useState('');
  const [linkingRel, setLinkingRel] = useState<ContactRelationship>('recruiter');

  // Activities
  const [showActivityForm, setShowActivityForm] = useState(false);
  const [actType, setActType] = useState<ActivityType>('email');
  const [actDesc, setActDesc] = useState('');
  const [actDate, setActDate] = useState(() => new Date().toISOString().slice(0, 16));

  // Tasks
  const [showTaskForm, setShowTaskForm] = useState(false);
  const [taskTitle, setTaskTitle] = useState('');
  const [taskRemindAt, setTaskRemindAt] = useState('');

  const { data: detail, isLoading, error } = useQuery({
    queryKey: queryKeys.applications.detail(id!),
    queryFn: () => getApplicationDetail(id!),
    enabled: !!id,
  });

  const { data: allContacts = [] } = useQuery({
    queryKey: queryKeys.contacts.all(),
    queryFn: getContacts,
  });

  const { data: match } = useQuery({
    queryKey: queryKeys.match.forJob(detail?.jobId ?? ''),
    queryFn: () => getMatch(detail!.jobId),
    enabled: !!detail?.jobId,
  });

  const invalidateDetail = () =>
    queryClient.invalidateQueries({ queryKey: queryKeys.applications.detail(id!) });

  const statusMutation = useMutation({
    mutationFn: (status: ApplicationStatus) => patchApplication(id!, status),
    onSuccess: (_, status) => {
      invalidateDetail();
      queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
      toast.success(`Status → ${status}`);
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const addNoteMutation = useMutation({
    mutationFn: (content: string) => addNote(id!, content),
    onSuccess: () => {
      invalidateDetail();
      setNoteText('');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const createAndLinkMutation = useMutation({
    mutationFn: async (vars: { name: string; email?: string; role?: string; rel: ContactRelationship }) => {
      const contact = await createContact({ name: vars.name, email: vars.email, role: vars.role });
      const link = await linkContact(id!, contact.id, vars.rel);
      return { contact, link };
    },
    onSuccess: ({ contact }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.contacts.all() });
      invalidateDetail();
      setNewContactName(''); setNewContactEmail(''); setNewContactRole('');
      setShowContactForm(false);
      toast.success(`${contact.name} added`);
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const linkExistingMutation = useMutation({
    mutationFn: (vars: { contactId: string; rel: ContactRelationship }) =>
      linkContact(id!, vars.contactId, vars.rel),
    onSuccess: () => {
      invalidateDetail();
      setLinkingContactId('');
      toast.success('Contact linked');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const unlinkMutation = useMutation({
    mutationFn: (linkId: string) => unlinkContact(id!, linkId),
    onSuccess: () => invalidateDetail(),
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const addActivityMutation = useMutation({
    mutationFn: (vars: { type: ActivityType; description: string; happenedAt: string }) =>
      createActivity(id!, vars),
    onSuccess: () => {
      invalidateDetail();
      setActDesc('');
      setShowActivityForm(false);
      toast.success('Activity logged');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const deleteActivityMutation = useMutation({
    mutationFn: (actId: string) => deleteActivity(actId),
    onSuccess: () => invalidateDetail(),
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const addTaskMutation = useMutation({
    mutationFn: (vars: { title: string; remindAt?: string }) => createTask(id!, vars),
    onSuccess: () => {
      invalidateDetail();
      setTaskTitle(''); setTaskRemindAt('');
      setShowTaskForm(false);
      toast.success('Task added');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const toggleTaskMutation = useMutation({
    mutationFn: (vars: { taskId: string; done: boolean }) => patchTask(vars.taskId, { done: vars.done }),
    onSuccess: () => invalidateDetail(),
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const deleteTaskMutation = useMutation({
    mutationFn: (taskId: string) => deleteTask(taskId),
    onSuccess: () => invalidateDetail(),
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const savingContact = createAndLinkMutation.isPending || linkExistingMutation.isPending;

  if (isLoading) return <SkeletonPage />;
  if (!detail) return <p className="error">{error instanceof Error ? error.message : 'Application not found'}</p>;

  const showResume = ['applied', 'interview', 'offer', 'rejected'].includes(detail.status);
  const sortedNotes: ApplicationNote[] = [...detail.notes].sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
  );
  const linkedContactIds = new Set(detail.contacts.map((c) => c.contact.id));
  const unlinkableContacts = allContacts.filter((c) => !linkedContactIds.has(c.id));

  return (
    <div className="appDetail">
      <Link to="/applications" className="linkBtn" style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}><ArrowLeft size={14} /> Back to Applications</Link>

      {/* Header */}
      <div className="pageHeader">
        <div>
          <h1>{detail.job.title}</h1>
          <p className="muted">{detail.job.company}</p>
        </div>
        <select
          className="statusSelect"
          value={detail.status}
          onChange={(e) => statusMutation.mutate(e.target.value as ApplicationStatus)}
        >
          {STATUSES.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>

      {/* Resume sent */}
      {showResume && (
        <section className="card">
          <p className="eyebrow">Resume sent</p>
          {detail.resume ? (
            <p>
              v{detail.resume.version} — {detail.resume.filename}{' '}
              <Link to="/resume" className="linkBtn">View resume →</Link>
            </p>
          ) : (
            <p className="muted">No resume linked to this application.</p>
          )}
        </section>
      )}

      {/* Fit score */}
      {match && (
        <section className="card">
          <p className="eyebrow">Fit Score</p>
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
            {match.notes && <p className="muted">{match.notes}</p>}
          </div>
        </section>
      )}

      {/* ── Contacts ─────────────────────────────────────────────────────── */}
      <section className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
          <p className="eyebrow" style={{ margin: 0 }}>People</p>
          <button
            type="button"
            onClick={() => setShowContactForm((v) => !v)}
            style={{ fontSize: 12, display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            {showContactForm ? 'Cancel' : <><Plus size={12} /> Add person</>}
          </button>
        </div>

        {detail.contacts.length > 0 && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8, marginBottom: 12 }}>
            {detail.contacts.map((link) => (
              <div key={link.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '6px 0', borderBottom: '1px solid var(--border)' }}>
                <div>
                  <strong style={{ fontSize: 14 }}>{link.contact.name}</strong>
                  {link.contact.role && <span className="muted" style={{ fontSize: 12, marginLeft: 6 }}>{link.contact.role}</span>}
                  <span className="pill" style={{ marginLeft: 8, fontSize: 11 }}>{link.relationship}</span>
                  {link.contact.email && (
                    <a href={`mailto:${link.contact.email}`} style={{ marginLeft: 8, fontSize: 12 }}>{link.contact.email}</a>
                  )}
                  {link.contact.linkedinUrl && (
                    <a href={link.contact.linkedinUrl} target="_blank" rel="noreferrer" style={{ marginLeft: 8, fontSize: 12 }}>LinkedIn →</a>
                  )}
                </div>
                <button
                  type="button"
                  onClick={() => unlinkMutation.mutate(link.id)}
                  style={{ background: 'transparent', border: 'none', color: 'var(--muted)', cursor: 'pointer', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                  title="Unlink"
                >
                  <X size={14} />
                </button>
              </div>
            ))}
          </div>
        )}

        {showContactForm && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {/* Create new */}
            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (!newContactName.trim()) return;
                createAndLinkMutation.mutate({
                  name: newContactName.trim(),
                  email: newContactEmail.trim() || undefined,
                  role: newContactRole.trim() || undefined,
                  rel: newContactRel,
                });
              }}
              style={{ display: 'flex', flexDirection: 'column', gap: 8, padding: 12, background: 'var(--surface, #f9f9f9)', borderRadius: 6 }}
            >
              <p style={{ margin: 0, fontWeight: 600, fontSize: 13 }}>New contact</p>
              <input
                placeholder="Name *"
                value={newContactName}
                onChange={(e) => setNewContactName(e.target.value)}
                required
                style={{ fontSize: 13 }}
              />
              <input
                placeholder="Email"
                type="email"
                value={newContactEmail}
                onChange={(e) => setNewContactEmail(e.target.value)}
                style={{ fontSize: 13 }}
              />
              <input
                placeholder="Role (e.g. Senior Recruiter)"
                value={newContactRole}
                onChange={(e) => setNewContactRole(e.target.value)}
                style={{ fontSize: 13 }}
              />
              <select value={newContactRel} onChange={(e) => setNewContactRel(e.target.value as ContactRelationship)} style={{ fontSize: 13 }}>
                {RELATIONSHIPS.map((r) => <option key={r} value={r}>{r}</option>)}
              </select>
              <button type="submit" disabled={savingContact || !newContactName.trim()} style={{ fontSize: 13 }}>
                {savingContact ? 'Saving…' : 'Create & link'}
              </button>
            </form>

            {/* Link existing */}
            {unlinkableContacts.length > 0 && (
              <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                <select value={linkingContactId} onChange={(e) => setLinkingContactId(e.target.value)} style={{ flex: 1, fontSize: 13 }}>
                  <option value="">— or link existing —</option>
                  {unlinkableContacts.map((c) => (
                    <option key={c.id} value={c.id}>{c.name}{c.role ? ` (${c.role})` : ''}</option>
                  ))}
                </select>
                <select value={linkingRel} onChange={(e) => setLinkingRel(e.target.value as ContactRelationship)} style={{ fontSize: 13 }}>
                  {RELATIONSHIPS.map((r) => <option key={r} value={r}>{r}</option>)}
                </select>
                <button
                  type="button"
                  disabled={!linkingContactId || savingContact}
                  onClick={() => linkExistingMutation.mutate({ contactId: linkingContactId, rel: linkingRel })}
                  style={{ fontSize: 13, whiteSpace: 'nowrap' }}
                >
                  Link
                </button>
              </div>
            )}
          </div>
        )}

        {detail.contacts.length === 0 && !showContactForm && (
          <p className="muted" style={{ margin: 0 }}>No people tracked yet.</p>
        )}
      </section>

      {/* ── Activity Log ─────────────────────────────────────────────────── */}
      <section className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
          <p className="eyebrow" style={{ margin: 0 }}>Activity Log</p>
          <button
            type="button"
            onClick={() => setShowActivityForm((v) => !v)}
            style={{ fontSize: 12, display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            {showActivityForm ? 'Cancel' : <><Plus size={12} /> Log activity</>}
          </button>
        </div>

        {showActivityForm && (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (!actDesc.trim()) return;
              addActivityMutation.mutate({
                type: actType,
                description: actDesc.trim(),
                happenedAt: new Date(actDate).toISOString(),
              });
            }}
            style={{ display: 'flex', flexDirection: 'column', gap: 8, marginBottom: 16, padding: 12, background: 'var(--surface, #f9f9f9)', borderRadius: 6 }}
          >
            <div style={{ display: 'flex', gap: 8 }}>
              <select value={actType} onChange={(e) => setActType(e.target.value as ActivityType)} style={{ fontSize: 13 }}>
                {ACTIVITY_TYPES.map((t) => <option key={t} value={t}>{ACTIVITY_LABELS[t]}</option>)}
              </select>
              <input
                type="datetime-local"
                value={actDate}
                onChange={(e) => setActDate(e.target.value)}
                style={{ fontSize: 13, flex: 1 }}
              />
            </div>
            <textarea
              value={actDesc}
              onChange={(e) => setActDesc(e.target.value)}
              placeholder="What happened?"
              rows={2}
              required
              style={{ fontSize: 13, resize: 'vertical' }}
            />
            <button type="submit" disabled={addActivityMutation.isPending || !actDesc.trim()} style={{ fontSize: 13 }}>
              {addActivityMutation.isPending ? 'Saving…' : 'Log'}
            </button>
          </form>
        )}

        {detail.activities.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
            {detail.activities.map((act) => (
              <div key={act.id} style={{ display: 'flex', gap: 10, padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                <div style={{ minWidth: 8, marginTop: 5 }}>
                  <div style={{ width: 8, height: 8, borderRadius: '50%', background: activityColor(act.type) }} />
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                    <span style={{ fontWeight: 600, fontSize: 13 }}>{ACTIVITY_LABELS[act.type]}</span>
                    <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                      <span className="muted" style={{ fontSize: 11 }}>
                        {new Date(act.happenedAt).toLocaleString('uk-UA', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })}
                      </span>
                      <button
                        type="button"
                        onClick={() => deleteActivityMutation.mutate(act.id)}
                        style={{ background: 'transparent', border: 'none', color: 'var(--muted)', cursor: 'pointer', padding: 0, display: 'inline-flex', alignItems: 'center', gap: 4 }}
                        title="Delete"
                      >
                        <X size={14} />
                      </button>
                    </div>
                  </div>
                  <p style={{ margin: '2px 0 0', fontSize: 13 }}>{act.description}</p>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="muted" style={{ margin: 0 }}>No activity logged yet.</p>
        )}
      </section>

      {/* ── Tasks / Follow-ups ───────────────────────────────────────────── */}
      <section className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
          <p className="eyebrow" style={{ margin: 0 }}>
            Tasks{' '}
            {detail.tasks.filter((t) => !t.done).length > 0 && (
              <span style={{ background: 'var(--accent, #38a169)', color: '#fff', borderRadius: 10, padding: '1px 7px', fontSize: 11, marginLeft: 6 }}>
                {detail.tasks.filter((t) => !t.done).length}
              </span>
            )}
          </p>
          <button
            type="button"
            onClick={() => setShowTaskForm((v) => !v)}
            style={{ fontSize: 12, display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            {showTaskForm ? 'Cancel' : <><Plus size={12} /> Add task</>}
          </button>
        </div>

        {showTaskForm && (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (!taskTitle.trim()) return;
              addTaskMutation.mutate({
                title: taskTitle.trim(),
                remindAt: taskRemindAt ? new Date(taskRemindAt).toISOString() : undefined,
              });
            }}
            style={{ display: 'flex', gap: 8, marginBottom: 16, alignItems: 'flex-end' }}
          >
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 6 }}>
              <input
                placeholder="Task title *"
                value={taskTitle}
                onChange={(e) => setTaskTitle(e.target.value)}
                required
                style={{ fontSize: 13 }}
              />
              <input
                type="datetime-local"
                value={taskRemindAt}
                onChange={(e) => setTaskRemindAt(e.target.value)}
                style={{ fontSize: 13 }}
                title="Remind at"
              />
            </div>
            <button type="submit" disabled={addTaskMutation.isPending || !taskTitle.trim()} style={{ fontSize: 13, whiteSpace: 'nowrap' }}>
              {addTaskMutation.isPending ? '…' : 'Add'}
            </button>
          </form>
        )}

        {detail.tasks.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {detail.tasks.map((task) => {
              const overdue = !task.done && task.remindAt && new Date(task.remindAt) < new Date();
              const dueSoon = !task.done && !overdue && task.remindAt && new Date(task.remindAt).getTime() - Date.now() < 48 * 60 * 60 * 1000;
              return (
                <div
                  key={task.id}
                  style={{
                    display: 'flex',
                    gap: 10,
                    alignItems: 'center',
                    padding: '6px 0',
                    borderBottom: '1px solid var(--border)',
                    opacity: task.done ? 0.5 : 1,
                  }}
                >
                  <input
                    type="checkbox"
                    checked={task.done}
                    onChange={() => toggleTaskMutation.mutate({ taskId: task.id, done: !task.done })}
                    style={{ cursor: 'pointer', width: 16, height: 16 }}
                  />
                  <div style={{ flex: 1 }}>
                    <span style={{ fontSize: 13, textDecoration: task.done ? 'line-through' : 'none' }}>
                      {task.title}
                    </span>
                    {task.remindAt && (
                      <span
                        style={{
                          marginLeft: 8,
                          fontSize: 11,
                          color: overdue ? 'var(--danger, #e53e3e)' : dueSoon ? '#d97706' : 'var(--muted)',
                        }}
                      >
                        {overdue ? 'Overdue: ' : dueSoon ? 'Due soon: ' : ''}
                        {new Date(task.remindAt).toLocaleString('uk-UA', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })}
                      </span>
                    )}
                  </div>
                  <button
                    type="button"
                    onClick={() => deleteTaskMutation.mutate(task.id)}
                    style={{ background: 'transparent', border: 'none', color: 'var(--muted)', cursor: 'pointer', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                    title="Delete"
                  >
                    <X size={14} />
                  </button>
                </div>
              );
            })}
          </div>
        ) : (
          <p className="muted" style={{ margin: 0 }}>No tasks yet.</p>
        )}
      </section>

      {/* Notes */}
      <section className="card">
        <p className="eyebrow">Notes</p>
        {sortedNotes.length > 0 ? (
          <div className="notesList">
            {sortedNotes.map((note) => (
              <div key={note.id} className="noteItem">
                <p style={{ margin: 0 }}>{note.content}</p>
                <p className="noteTime">{new Date(note.createdAt).toLocaleString()}</p>
              </div>
            ))}
          </div>
        ) : (
          <p className="muted" style={{ marginBottom: 16 }}>No notes yet.</p>
        )}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (!noteText.trim()) return;
            addNoteMutation.mutate(noteText.trim());
          }}
          className="noteForm"
        >
          <textarea
            value={noteText}
            onChange={(e) => setNoteText(e.target.value)}
            placeholder="Add a note…"
          />
          <button type="submit" disabled={addNoteMutation.isPending || !noteText.trim()}>
            {addNoteMutation.isPending ? 'Adding…' : 'Add Note'}
          </button>
        </form>
      </section>
    </div>
  );
}

function activityColor(type: ActivityType): string {
  const colors: Record<ActivityType, string> = {
    email: '#3b82f6',
    call: '#8b5cf6',
    interview: '#f59e0b',
    follow_up: '#10b981',
    note: '#6b7280',
    other: '#9ca3af',
  };
  return colors[type] ?? '#9ca3af';
}
