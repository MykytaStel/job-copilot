import { useState } from 'react';
import { Link } from 'react-router-dom';
import {
  Activity,
  ArrowLeft,
  BriefcaseBusiness,
  CalendarClock,
  FileText,
  Handshake,
  ListTodo,
  NotebookPen,
  Users,
} from 'lucide-react';
import type {
  ApplicationContact,
  ApplicationDetail,
  ApplicationStatus,
  Contact,
  ContactInput,
  ContactRelationship,
  OfferStatus,
} from '@job-copilot/shared';

import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { SectionHeader } from '../../components/ui/SectionHeader';
import { StatusBadge } from '../../components/ui/StatusBadge';
import { formatDate, formatEnumLabel } from '../../lib/format';
import {
  APPLICATION_STATUS_OPTIONS,
  OFFER_STATUS_OPTIONS,
  RELATIONSHIP_OPTIONS,
} from './applicationDetail.constants';

function Panel({
  title,
  description,
  icon,
  children,
}: {
  title: string;
  description: string;
  icon: typeof CalendarClock;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
      <SectionHeader title={title} description={description} icon={icon} />
      {children}
    </section>
  );
}

function InnerPanel({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-4 rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <div>
        <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
        {description ? (
          <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {children}
    </div>
  );
}

function SummaryMetric({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">{label}</p>
      <p className="m-0 mt-2 text-sm font-semibold text-card-foreground">{value}</p>
    </div>
  );
}

export function ApplicationHeader({ detail }: { detail: ApplicationDetail }) {
  return (
    <div className="overflow-hidden rounded-[28px] border border-border bg-card/85 shadow-[var(--shadow-hero)]">
      <div className="relative">
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/10 via-accent/6 to-transparent" />
        <div className="relative space-y-6 p-7">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-4">
              <Link
                to="/applications"
                className="inline-flex items-center gap-2 text-sm text-primary no-underline hover:underline"
              >
                <ArrowLeft className="h-4 w-4" />
                Back to board
              </Link>

              <div className="space-y-3">
                <div className="flex flex-wrap gap-2">
                  <Badge
                    variant="default"
                    className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                  >
                    Application record
                  </Badge>
                  <Badge
                    variant="muted"
                    className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                  >
                    Notes, contacts, offer, tasks
                  </Badge>
                </div>
                <div>
                  <h1 className="m-0 text-2xl font-bold text-card-foreground">
                    {detail.job.title}
                  </h1>
                  <p className="m-0 mt-2 text-base text-muted-foreground">{detail.job.company}</p>
                </div>
              </div>

              <div className="flex flex-wrap items-center gap-3">
                <StatusBadge status={detail.status} />
                {detail.appliedAt ? (
                  <span className="rounded-full border border-border bg-white/[0.05] px-3 py-1.5 text-xs text-muted-foreground">
                    Applied {formatDate(detail.appliedAt)}
                  </span>
                ) : null}
                {detail.dueDate ? (
                  <span className="rounded-full border border-border bg-white/[0.05] px-3 py-1.5 text-xs text-muted-foreground">
                    Due {formatDate(detail.dueDate)}
                  </span>
                ) : null}
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 lg:min-w-[360px]">
              <SummaryMetric label="Contacts" value={detail.contacts.length} />
              <SummaryMetric label="Notes" value={detail.notes.length} />
              <SummaryMetric label="Tasks" value={detail.tasks.length} />
              <SummaryMetric label="Activities" value={detail.activities.length} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function ApplicationFormSection({
  status,
  dueDate,
  isPending,
  hasChanges,
  setStatus,
  setDueDate,
  clearDueDate,
  onSubmit,
}: {
  status: ApplicationStatus;
  dueDate: string;
  isPending: boolean;
  hasChanges: boolean;
  setStatus: (value: ApplicationStatus) => void;
  setDueDate: (value: string) => void;
  clearDueDate: () => void;
  onSubmit: () => void;
}) {
  return (
    <Panel
      title="Pipeline Status"
      description="Keep the application stage and due date aligned with the current process."
      icon={CalendarClock}
    >
      <form
        className="space-y-5"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <div className="grid gap-4 md:grid-cols-2">
          <label>
            Status
            <select
              value={status}
              onChange={(event) => setStatus(event.target.value as ApplicationStatus)}
            >
              {APPLICATION_STATUS_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {formatEnumLabel(value)}
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

        <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
          <p className="m-0 text-sm text-muted-foreground">
            Save only when something actually changed to keep the activity trail clean.
          </p>
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={clearDueDate}
              disabled={isPending || !dueDate}
            >
              Clear due date
            </Button>
            <Button type="submit" disabled={isPending || !hasChanges}>
              {isPending ? 'Saving...' : 'Save application'}
            </Button>
          </div>
        </div>
      </form>
    </Panel>
  );
}

export function JobDetailsSection({ detail }: { detail: ApplicationDetail }) {
  const { job } = detail;

  return (
    <Panel
      title="Role Snapshot"
      description="Reference copy of the linked job posting and its source metadata."
      icon={BriefcaseBusiness}
    >
      <div className="grid gap-4 md:grid-cols-2">
        {job.url ? (
          <InnerPanel title="Source link" description="Original posting used for the application.">
            <a
              href={job.url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-primary no-underline hover:underline"
            >
              {job.url}
            </a>
          </InnerPanel>
        ) : null}

        <InnerPanel
          title="Posting timeline"
          description="Current metadata from the linked job record."
        >
          <div className="space-y-2 text-sm text-muted-foreground">
            <div className="flex items-center justify-between gap-3">
              <span>Created</span>
              <span className="font-medium text-card-foreground">
                {formatDate(job.createdAt) ?? 'n/a'}
              </span>
            </div>
          </div>
        </InnerPanel>
      </div>

      <DescriptionBlock text={job.description || 'No description available.'} />
    </Panel>
  );
}

export function NotesSection({
  notes,
  noteContent,
  isPending,
  setNoteContent,
  onSubmit,
}: {
  notes: ApplicationDetail['notes'];
  noteContent: string;
  isPending: boolean;
  setNoteContent: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <Panel
      title="Notes"
      description="Capture recruiter context, interview takeaways, and decision rationale."
      icon={NotebookPen}
    >
      <form
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <textarea
          value={noteContent}
          onChange={(event) => setNoteContent(event.target.value)}
          rows={4}
          placeholder="Add context from recruiter calls, takeaways, or follow-up reminders."
        />
        <div className="flex justify-end">
          <Button type="submit" disabled={isPending || !noteContent.trim()}>
            {isPending ? 'Saving...' : 'Add note'}
          </Button>
        </div>
      </form>

      {notes.length === 0 ? (
        <EmptyState message="No notes yet" />
      ) : (
        <div className="space-y-3">
          {notes.map((note) => (
            <div
              key={note.id}
              className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4"
            >
              <p className="m-0 text-sm leading-7 text-card-foreground">{note.content}</p>
              <p className="m-0 mt-3 text-xs text-muted-foreground">{formatDate(note.createdAt)}</p>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}

export function ContactsSection({
  detail,
  contactsLoading,
  availableContacts,
  existingContactId,
  existingRelationship,
  linkPending,
  setExistingContactId,
  setExistingRelationship,
  onLinkExisting,
  newContact,
  newContactRelationship,
  createPending,
  setNewContactField,
  setNewContactRelationship,
  onCreateAndLink,
}: {
  detail: ApplicationDetail;
  contactsLoading: boolean;
  availableContacts: Contact[];
  existingContactId: string;
  existingRelationship: ContactRelationship;
  linkPending: boolean;
  setExistingContactId: (value: string) => void;
  setExistingRelationship: (value: ContactRelationship) => void;
  onLinkExisting: () => void;
  newContact: ContactInput;
  newContactRelationship: ContactRelationship;
  createPending: boolean;
  setNewContactField: <K extends keyof ContactInput>(field: K, value: ContactInput[K]) => void;
  setNewContactRelationship: (value: ContactRelationship) => void;
  onCreateAndLink: () => void;
}) {
  return (
    <Panel
      title="Contacts"
      description="Link known people to the application or create fresh recruiter and hiring-manager records."
      icon={Users}
    >
      <form
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          onLinkExisting();
        }}
      >
        <InnerPanel
          title="Attach existing contact"
          description="Reuse a contact that already exists in the CRM layer."
        >
          {contactsLoading ? (
            <p className="m-0 text-sm text-muted-foreground">Loading contacts...</p>
          ) : availableContacts.length === 0 ? (
            <EmptyState message="No unlinked contacts available yet." />
          ) : (
            <>
              <div className="grid gap-4 md:grid-cols-2">
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
                        {formatEnumLabel(value)}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <div className="flex justify-end">
                <Button
                  type="submit"
                  disabled={linkPending || availableContacts.length === 0 || !existingContactId}
                >
                  {linkPending ? 'Linking...' : 'Link contact'}
                </Button>
              </div>
            </>
          )}
        </InnerPanel>
      </form>

      <form
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          onCreateAndLink();
        }}
      >
        <InnerPanel
          title="Create and link new contact"
          description="Store a new person record and attach it to this application."
        >
          <div className="grid gap-4 md:grid-cols-2">
            <label>
              Name
              <input
                value={newContact.name}
                onChange={(event) => setNewContactField('name', event.target.value)}
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
                    {formatEnumLabel(value)}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Email
              <input
                type="email"
                value={newContact.email ?? ''}
                onChange={(event) => setNewContactField('email', event.target.value)}
                placeholder="jane@example.com"
              />
            </label>
            <label>
              Phone
              <input
                value={newContact.phone ?? ''}
                onChange={(event) => setNewContactField('phone', event.target.value)}
                placeholder="+380..."
              />
            </label>
            <label>
              Company
              <input
                value={newContact.company ?? ''}
                onChange={(event) => setNewContactField('company', event.target.value)}
                placeholder="NovaLedger"
              />
            </label>
            <label>
              Role
              <input
                value={newContact.role ?? ''}
                onChange={(event) => setNewContactField('role', event.target.value)}
                placeholder="Recruiter"
              />
            </label>
          </div>
          <label>
            LinkedIn URL
            <input
              value={newContact.linkedinUrl ?? ''}
              onChange={(event) => setNewContactField('linkedinUrl', event.target.value)}
              placeholder="https://linkedin.com/in/..."
            />
          </label>
          <div className="flex justify-end">
            <Button type="submit" disabled={createPending || !newContact.name.trim()}>
              {createPending ? 'Saving...' : 'Create contact'}
            </Button>
          </div>
        </InnerPanel>
      </form>

      {detail.contacts.length === 0 ? (
        <EmptyState message="No contacts yet" />
      ) : (
        <div className="space-y-3">
          {detail.contacts.map((applicationContact) => (
            <ContactCard key={applicationContact.id} item={applicationContact} />
          ))}
        </div>
      )}
    </Panel>
  );
}

export function OfferSection({
  detail,
  compensationLabel,
  status,
  min,
  max,
  currency,
  startsAt,
  notes,
  isPending,
  setStatus,
  setMin,
  setMax,
  setCurrency,
  setStartsAt,
  setNotes,
  onSubmit,
}: {
  detail: ApplicationDetail;
  compensationLabel: string | null;
  status: OfferStatus;
  min: string;
  max: string;
  currency: string;
  startsAt: string;
  notes: string;
  isPending: boolean;
  setStatus: (value: OfferStatus) => void;
  setMin: (value: string) => void;
  setMax: (value: string) => void;
  setCurrency: (value: string) => void;
  setStartsAt: (value: string) => void;
  setNotes: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <Panel
      title="Offer Tracking"
      description="Record package details, status, and final decision context in one place."
      icon={Handshake}
    >
      {detail.offer ? (
        <div className="grid gap-4 md:grid-cols-3">
          <InnerPanel title="Status">
            <StatusBadge status={detail.offer.status} />
          </InnerPanel>
          <InnerPanel title="Compensation">
            <p className="m-0 text-sm text-card-foreground">{compensationLabel ?? 'Not set yet'}</p>
          </InnerPanel>
          <InnerPanel title="Starts at">
            <p className="m-0 text-sm text-card-foreground">
              {detail.offer.startsAt ? formatDate(detail.offer.startsAt) : 'Not set yet'}
            </p>
          </InnerPanel>
        </div>
      ) : (
        <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
          <p className="m-0 text-sm text-muted-foreground">No offer saved yet.</p>
        </div>
      )}

      <form
        className="space-y-5"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <div className="grid gap-4 md:grid-cols-2">
          <label>
            Status
            <select
              value={status}
              onChange={(event) => setStatus(event.target.value as OfferStatus)}
            >
              {OFFER_STATUS_OPTIONS.map((value) => (
                <option key={value} value={value}>
                  {formatEnumLabel(value)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Currency
            <input
              value={currency}
              onChange={(event) => setCurrency(event.target.value)}
              placeholder="USD"
            />
          </label>
          <label>
            Compensation min
            <input
              type="number"
              min="0"
              value={min}
              onChange={(event) => setMin(event.target.value)}
              placeholder="5000"
            />
          </label>
          <label>
            Compensation max
            <input
              type="number"
              min="0"
              value={max}
              onChange={(event) => setMax(event.target.value)}
              placeholder="6500"
            />
          </label>
          <label>
            Starts at
            <input
              type="date"
              value={startsAt}
              onChange={(event) => setStartsAt(event.target.value)}
            />
          </label>
        </div>
        <label>
          Notes
          <textarea
            rows={4}
            value={notes}
            onChange={(event) => setNotes(event.target.value)}
            placeholder="Offer notes, package details, or decision context."
          />
        </label>
        <div className="flex justify-end">
          <Button type="submit" disabled={isPending}>
            {isPending ? 'Saving...' : detail.offer ? 'Update offer' : 'Save offer'}
          </Button>
        </div>
      </form>
    </Panel>
  );
}

export function ActivitiesSection({ activities }: { activities: ApplicationDetail['activities'] }) {
  return (
    <Panel
      title="Activities"
      description="Timeline of synced events and manual updates for this application."
      icon={Activity}
    >
      {activities.length === 0 ? (
        <EmptyState message="No activities yet" />
      ) : (
        <div className="space-y-3">
          {activities.map((activity) => (
            <div
              key={activity.id}
              className="flex items-start gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4"
            >
              <Badge
                variant="muted"
                className="mt-0.5 px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {formatEnumLabel(activity.type)}
              </Badge>
              <div className="min-w-0">
                <p className="m-0 text-sm leading-6 text-card-foreground">{activity.description}</p>
                <p className="m-0 mt-2 text-xs text-muted-foreground">
                  {formatDate(activity.happenedAt)}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}

export function TasksSection({ tasks }: { tasks: ApplicationDetail['tasks'] }) {
  return (
    <Panel
      title="Tasks"
      description="Outstanding follow-ups and reminders attached to this application."
      icon={ListTodo}
    >
      {tasks.length === 0 ? (
        <EmptyState message="No tasks yet" />
      ) : (
        <div className="space-y-3">
          {tasks.map((task) => (
            <div
              key={task.id}
              className="flex items-start gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4"
            >
              <input type="checkbox" checked={task.done} readOnly className="mt-1 h-4 w-4" />
              <div className="min-w-0">
                <p
                  className={`m-0 text-sm leading-6 ${
                    task.done ? 'text-muted-foreground line-through' : 'text-card-foreground'
                  }`}
                >
                  {task.title}
                </p>
                {task.remindAt ? (
                  <p className="m-0 mt-2 text-xs text-muted-foreground">
                    Remind: {formatDate(task.remindAt)}
                  </p>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}

function DescriptionBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const limit = 1200;
  const shouldTruncate = text.length > limit;
  const displayed = expanded || !shouldTruncate ? text : `${text.slice(0, limit)}...`;

  return (
    <div className="space-y-3 rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <div className="flex items-center gap-2">
        <FileText className="h-4 w-4 text-primary" />
        <p className="m-0 text-sm font-semibold text-card-foreground">Job description</p>
      </div>
      <div className="whitespace-pre-wrap text-sm leading-7 text-muted-foreground">{displayed}</div>
      {shouldTruncate ? (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="px-0 text-primary hover:text-primary"
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? 'Show less' : 'Show more'}
        </Button>
      ) : null}
    </div>
  );
}

function ContactCard({ item }: { item: ApplicationContact }) {
  const contact = item.contact;

  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4">
      <div className="flex flex-wrap items-center gap-2">
        <p className="m-0 text-sm font-semibold text-card-foreground">{contact.name}</p>
        <Badge variant="muted" className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]">
          {formatEnumLabel(item.relationship)}
        </Badge>
      </div>
      {contact.role || contact.company ? (
        <p className="m-0 mt-2 text-sm text-muted-foreground">
          {[contact.role, contact.company].filter(Boolean).join(' at ')}
        </p>
      ) : null}
      <div className="mt-3 flex flex-wrap gap-3 text-xs text-muted-foreground">
        {contact.email ? (
          <a href={`mailto:${contact.email}`} className="text-primary no-underline hover:underline">
            {contact.email}
          </a>
        ) : null}
        {contact.phone ? <span>{contact.phone}</span> : null}
        {contact.linkedinUrl ? (
          <a
            href={contact.linkedinUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary no-underline hover:underline"
          >
            LinkedIn
          </a>
        ) : null}
      </div>
    </div>
  );
}
