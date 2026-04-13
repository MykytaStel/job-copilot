import { useEffect, useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type {
  ApplicationContact,
  ApplicationDetail as ApplicationDetailType,
  ApplicationStatus,
  ContactInput,
  ContactRelationship,
  OfferStatus,
} from '@job-copilot/shared';

import {
  addNote,
  createContact,
  createOffer,
  getApplicationDetail,
  getContacts,
  linkContact,
  updateApplication,
} from '../api';
import { queryKeys } from '../queryKeys';

const RELATIONSHIP_OPTIONS: ContactRelationship[] = [
  'recruiter',
  'hiring_manager',
  'interviewer',
  'referrer',
  'other',
];

const OFFER_STATUS_OPTIONS: OfferStatus[] = [
  'draft',
  'received',
  'accepted',
  'declined',
  'expired',
];

const APPLICATION_STATUS_OPTIONS: ApplicationStatus[] = [
  'saved',
  'applied',
  'interview',
  'offer',
  'rejected',
];

function fmt(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}

function StatusBadge({ status }: { status: string }) {
  return <span className={`statusPill status-${status}`}>{status}</span>;
}

function SectionHeader({ title }: { title: string }) {
  return <p className="eyebrow" style={{ marginTop: 0 }}>{title}</p>;
}

function EmptyState({ message }: { message: string }) {
  return (
    <p className="muted" style={{ margin: 0, fontStyle: 'italic' }}>
      {message}
    </p>
  );
}

function DescriptionBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const limit = 600;
  const shouldTruncate = text.length > limit;
  const displayed = expanded || !shouldTruncate ? text : text.slice(0, limit) + '...';

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

function normalizeDateInput(value?: string) {
  return value?.slice(0, 10) ?? '';
}

function parseOptionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return undefined;

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function formatRelationship(value: string) {
  return value.replace('_', ' ');
}

function formatCompensation(detail: ApplicationDetailType) {
  const offer = detail.offer;
  if (!offer) return null;

  const min = offer.compensationMin;
  const max = offer.compensationMax;
  const currency = offer.compensationCurrency ?? '';

  if (min == null && max == null) return null;
  if (min != null && max != null) return `${min} - ${max} ${currency}`.trim();
  if (min != null) return `${min}+ ${currency}`.trim();
  return `Up to ${max} ${currency}`.trim();
}

const formStackStyle = {
  display: 'flex',
  flexDirection: 'column',
  gap: 12,
} as const;

const formGridStyle = {
  display: 'grid',
  gap: 12,
  gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
} as const;

export default function ApplicationDetail() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();

  const [noteContent, setNoteContent] = useState('');
  const [existingContactId, setExistingContactId] = useState('');
  const [existingRelationship, setExistingRelationship] =
    useState<ContactRelationship>('recruiter');
  const [newContactRelationship, setNewContactRelationship] =
    useState<ContactRelationship>('recruiter');
  const [newContact, setNewContact] = useState<ContactInput>({
    name: '',
    email: '',
    phone: '',
    linkedinUrl: '',
    company: '',
    role: '',
  });
  const [offerStatus, setOfferStatus] = useState<OfferStatus>('draft');
  const [offerMin, setOfferMin] = useState('');
  const [offerMax, setOfferMax] = useState('');
  const [offerCurrency, setOfferCurrency] = useState('USD');
  const [offerStartsAt, setOfferStartsAt] = useState('');
  const [offerNotes, setOfferNotes] = useState('');
  const [applicationStatus, setApplicationStatus] = useState<ApplicationStatus>('saved');
  const [dueDate, setDueDate] = useState('');

  const detailQuery = useQuery({
    queryKey: queryKeys.applications.detail(id ?? ''),
    enabled: Boolean(id),
    queryFn: () => getApplicationDetail(id!),
  });

  const contactsQuery = useQuery({
    queryKey: queryKeys.contacts.all(),
    queryFn: getContacts,
  });

  const detail = detailQuery.data;

  useEffect(() => {
    if (!detail?.offer) {
      setOfferStatus('draft');
      setOfferMin('');
      setOfferMax('');
      setOfferCurrency('USD');
      setOfferStartsAt('');
      setOfferNotes('');
      return;
    }

    setOfferStatus(detail.offer.status);
    setOfferMin(detail.offer.compensationMin?.toString() ?? '');
    setOfferMax(detail.offer.compensationMax?.toString() ?? '');
    setOfferCurrency(detail.offer.compensationCurrency ?? 'USD');
    setOfferStartsAt(normalizeDateInput(detail.offer.startsAt));
    setOfferNotes(detail.offer.notes ?? '');
  }, [detail?.offer]);

  useEffect(() => {
    if (!detail) return;

    setApplicationStatus(detail.status);
    setDueDate(normalizeDateInput(detail.dueDate));
  }, [detail]);

  const availableContacts = useMemo(() => {
    if (!detail || !contactsQuery.data) return [];

    const attachedIds = new Set(detail.contacts.map((item) => item.contact.id));
    return contactsQuery.data.filter((contact) => !attachedIds.has(contact.id));
  }, [contactsQuery.data, detail]);

  useEffect(() => {
    if (!existingContactId && availableContacts.length > 0) {
      setExistingContactId(availableContacts[0].id);
    }
    if (availableContacts.length === 0) {
      setExistingContactId('');
    }
  }, [availableContacts, existingContactId]);

  async function refreshDetail() {
    if (!id) return;
    await queryClient.invalidateQueries({ queryKey: queryKeys.applications.detail(id) });
  }

  const noteMutation = useMutation({
    mutationFn: (content: string) => addNote(id!, content),
    onSuccess: async () => {
      setNoteContent('');
      await refreshDetail();
      toast.success('Note added');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Failed to add note');
    },
  });

  const linkExistingContactMutation = useMutation({
    mutationFn: (payload: {
      contactId: string;
      relationship: ApplicationContact['relationship'];
    }) => linkContact(id!, payload.contactId, payload.relationship),
    onSuccess: async () => {
      await Promise.all([
        refreshDetail(),
        queryClient.invalidateQueries({ queryKey: queryKeys.contacts.all() }),
      ]);
      toast.success('Contact linked');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Failed to link contact');
    },
  });

  const createAndLinkContactMutation = useMutation({
    mutationFn: async (payload: {
      contact: ContactInput;
      relationship: ApplicationContact['relationship'];
    }) => {
      const created = await createContact(payload.contact);
      return linkContact(id!, created.id, payload.relationship);
    },
    onSuccess: async () => {
      setNewContact({
        name: '',
        email: '',
        phone: '',
        linkedinUrl: '',
        company: '',
        role: '',
      });
      setNewContactRelationship('recruiter');
      await Promise.all([
        refreshDetail(),
        queryClient.invalidateQueries({ queryKey: queryKeys.contacts.all() }),
      ]);
      toast.success('Contact created and linked');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Failed to create contact');
    },
  });

  const offerMutation = useMutation({
    mutationFn: () =>
      createOffer({
        applicationId: id!,
        status: offerStatus,
        compensationMin: parseOptionalNumber(offerMin),
        compensationMax: parseOptionalNumber(offerMax),
        compensationCurrency: offerCurrency.trim() || undefined,
        startsAt: offerStartsAt || undefined,
        notes: offerNotes.trim() || undefined,
      }),
    onSuccess: async () => {
      await refreshDetail();
      toast.success('Offer saved');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Failed to save offer');
    },
  });

  const applicationMutation = useMutation({
    mutationFn: () =>
      updateApplication(id!, {
        status: applicationStatus,
        dueDate: dueDate || null,
      }),
    onSuccess: async () => {
      await refreshDetail();
      toast.success('Application updated');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Failed to update application');
    },
  });

  if (!id) return <p className="error">Application not found</p>;
  if (detailQuery.isLoading) return <p className="muted">Loading...</p>;
  if (detailQuery.error || !detail) {
    return (
      <p className="error">
        {detailQuery.error instanceof Error
          ? detailQuery.error.message
          : 'Application not found'}
      </p>
    );
  }

  const { job } = detail;
  const compensationLabel = formatCompensation(detail);
  const normalizedCurrentDueDate = normalizeDateInput(detail.dueDate);
  const hasApplicationChanges =
    applicationStatus !== detail.status || dueDate !== normalizedCurrentDueDate;

  return (
    <div className="jobDetails">
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

      <section className="card">
        <SectionHeader title="Application" />
        <form
          style={formStackStyle}
          onSubmit={(event) => {
            event.preventDefault();
            if (!hasApplicationChanges) return;
            applicationMutation.mutate();
          }}
        >
          <div style={formGridStyle}>
            <label>
              Status
              <select
                value={applicationStatus}
                onChange={(event) =>
                  setApplicationStatus(event.target.value as ApplicationStatus)
                }
              >
                {APPLICATION_STATUS_OPTIONS.map((value) => (
                  <option key={value} value={value}>
                    {value}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Due date
              <input
                type="date"
                value={dueDate}
                onChange={(event) => setDueDate(event.target.value)}
              />
            </label>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', gap: 12 }}>
            <button
              type="button"
              onClick={() => setDueDate('')}
              disabled={applicationMutation.isPending || !dueDate}
            >
              Clear due date
            </button>
            <button
              type="submit"
              disabled={applicationMutation.isPending || !hasApplicationChanges}
            >
              {applicationMutation.isPending ? 'Saving...' : 'Save application'}
            </button>
          </div>
        </form>
      </section>

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

      <section className="card">
        <SectionHeader title="Notes" />
        <form
          style={{ ...formStackStyle, marginBottom: detail.notes.length > 0 ? 16 : 0 }}
          onSubmit={(event) => {
            event.preventDefault();
            if (!noteContent.trim()) return;
            noteMutation.mutate(noteContent.trim());
          }}
        >
          <textarea
            value={noteContent}
            onChange={(event) => setNoteContent(event.target.value)}
            rows={4}
            placeholder="Add context from recruiter calls, takeaways, or follow-up reminders."
          />
          <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
            <button
              type="submit"
              disabled={noteMutation.isPending || !noteContent.trim()}
            >
              {noteMutation.isPending ? 'Saving...' : 'Add note'}
            </button>
          </div>
        </form>

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

      <section className="card">
        <SectionHeader title="Contacts" />

        <form
          style={{ ...formStackStyle, marginBottom: 16 }}
          onSubmit={(event) => {
            event.preventDefault();
            if (!existingContactId) return;
            linkExistingContactMutation.mutate({
              contactId: existingContactId,
              relationship: existingRelationship,
            });
          }}
        >
          <p className="muted" style={{ margin: 0 }}>Attach existing contact</p>
          {contactsQuery.isLoading ? (
            <p className="muted" style={{ margin: 0 }}>Loading contacts...</p>
          ) : availableContacts.length === 0 ? (
            <EmptyState message="No unlinked contacts available yet." />
          ) : (
            <div style={formGridStyle}>
              <label>
                Contact
                <select
                  value={existingContactId}
                  onChange={(event) => setExistingContactId(event.target.value)}
                >
                  {availableContacts.map((contact) => (
                    <option key={contact.id} value={contact.id}>
                      {contact.name}
                      {contact.company ? ` - ${contact.company}` : ''}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Relationship
                <select
                  value={existingRelationship}
                  onChange={(event) =>
                    setExistingRelationship(event.target.value as ContactRelationship)
                  }
                >
                  {RELATIONSHIP_OPTIONS.map((value) => (
                    <option key={value} value={value}>
                      {formatRelationship(value)}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          )}
          <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
            <button
              type="submit"
              disabled={
                linkExistingContactMutation.isPending ||
                availableContacts.length === 0 ||
                !existingContactId
              }
            >
              {linkExistingContactMutation.isPending ? 'Linking...' : 'Link contact'}
            </button>
          </div>
        </form>

        <form
          style={{ ...formStackStyle, marginBottom: detail.contacts.length > 0 ? 16 : 0 }}
          onSubmit={(event) => {
            event.preventDefault();
            if (!newContact.name.trim()) return;

            createAndLinkContactMutation.mutate({
              relationship: newContactRelationship,
              contact: {
                name: newContact.name.trim(),
                email: newContact.email?.trim() || undefined,
                phone: newContact.phone?.trim() || undefined,
                linkedinUrl: newContact.linkedinUrl?.trim() || undefined,
                company: newContact.company?.trim() || undefined,
                role: newContact.role?.trim() || undefined,
              },
            });
          }}
        >
          <p className="muted" style={{ margin: 0 }}>Create and link new contact</p>
          <div style={formGridStyle}>
            <label>
              Name
              <input
                value={newContact.name}
                onChange={(event) =>
                  setNewContact((current) => ({ ...current, name: event.target.value }))
                }
                placeholder="Jane Recruiter"
                required
              />
            </label>
            <label>
              Relationship
              <select
                value={newContactRelationship}
                onChange={(event) =>
                  setNewContactRelationship(event.target.value as ContactRelationship)
                }
              >
                {RELATIONSHIP_OPTIONS.map((value) => (
                  <option key={value} value={value}>
                    {formatRelationship(value)}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Email
              <input
                type="email"
                value={newContact.email ?? ''}
                onChange={(event) =>
                  setNewContact((current) => ({ ...current, email: event.target.value }))
                }
                placeholder="jane@example.com"
              />
            </label>
            <label>
              Phone
              <input
                value={newContact.phone ?? ''}
                onChange={(event) =>
                  setNewContact((current) => ({ ...current, phone: event.target.value }))
                }
                placeholder="+380..."
              />
            </label>
            <label>
              Company
              <input
                value={newContact.company ?? ''}
                onChange={(event) =>
                  setNewContact((current) => ({ ...current, company: event.target.value }))
                }
                placeholder="NovaLedger"
              />
            </label>
            <label>
              Role
              <input
                value={newContact.role ?? ''}
                onChange={(event) =>
                  setNewContact((current) => ({ ...current, role: event.target.value }))
                }
                placeholder="Recruiter"
              />
            </label>
          </div>
          <label>
            LinkedIn URL
            <input
              value={newContact.linkedinUrl ?? ''}
              onChange={(event) =>
                setNewContact((current) => ({ ...current, linkedinUrl: event.target.value }))
              }
              placeholder="https://linkedin.com/in/..."
            />
          </label>
          <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
            <button
              type="submit"
              disabled={createAndLinkContactMutation.isPending || !newContact.name.trim()}
            >
              {createAndLinkContactMutation.isPending ? 'Saving...' : 'Create contact'}
            </button>
          </div>
        </form>

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
                    <span
                      className="badge badge-secondary"
                      style={{ fontSize: 11, padding: '3px 8px' }}
                    >
                      {formatRelationship(ac.relationship)}
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
                      <a
                        href={c.linkedinUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="linkBtn"
                        style={{ fontSize: 13 }}
                      >
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

      <section className="card">
        <SectionHeader title="Offer" />

        {detail.offer ? (
          <div
            style={{
              display: 'flex',
              flexWrap: 'wrap',
              gap: '6px 16px',
              marginBottom: 16,
            }}
          >
            <span className="statusPill">
              {detail.offer.status}
            </span>
            {compensationLabel && (
              <span className="muted" style={{ fontSize: 13 }}>
                Compensation: {compensationLabel}
              </span>
            )}
            {detail.offer.startsAt && (
              <span className="muted" style={{ fontSize: 13 }}>
                Start: {fmt(detail.offer.startsAt)}
              </span>
            )}
          </div>
        ) : (
          <p className="muted" style={{ marginTop: 0 }}>
            No offer saved yet.
          </p>
        )}

        <form
          style={formStackStyle}
          onSubmit={(event) => {
            event.preventDefault();
            offerMutation.mutate();
          }}
        >
          <div style={formGridStyle}>
            <label>
              Status
              <select
                value={offerStatus}
                onChange={(event) => setOfferStatus(event.target.value as OfferStatus)}
              >
                {OFFER_STATUS_OPTIONS.map((value) => (
                  <option key={value} value={value}>
                    {value}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Currency
              <input
                value={offerCurrency}
                onChange={(event) => setOfferCurrency(event.target.value)}
                placeholder="USD"
              />
            </label>
            <label>
              Compensation min
              <input
                type="number"
                min="0"
                value={offerMin}
                onChange={(event) => setOfferMin(event.target.value)}
                placeholder="5000"
              />
            </label>
            <label>
              Compensation max
              <input
                type="number"
                min="0"
                value={offerMax}
                onChange={(event) => setOfferMax(event.target.value)}
                placeholder="6500"
              />
            </label>
            <label>
              Starts at
              <input
                type="date"
                value={offerStartsAt}
                onChange={(event) => setOfferStartsAt(event.target.value)}
              />
            </label>
          </div>
          <label>
            Notes
            <textarea
              rows={4}
              value={offerNotes}
              onChange={(event) => setOfferNotes(event.target.value)}
              placeholder="Offer notes, package details, or decision context."
            />
          </label>
          <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
            <button type="submit" disabled={offerMutation.isPending}>
              {offerMutation.isPending ? 'Saving...' : detail.offer ? 'Update offer' : 'Save offer'}
            </button>
          </div>
        </form>
      </section>

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
