import { useState } from 'react';
import { Link } from 'react-router-dom';
import type {
  ApplicationContact,
  ApplicationDetail,
  ApplicationStatus,
  Contact,
  ContactInput,
  ContactRelationship,
  OfferStatus,
} from '@job-copilot/shared';

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

export function ApplicationHeader({ detail }: { detail: ApplicationDetail }) {
  return (
    <div className="pageHeader pageHeaderTop">
      <div className="pageTitleBlock">
        <Link to="/applications" className="linkBtn backLink">
          &larr; Back to Board
        </Link>
        <h1 className="applicationItemTitle">{detail.job.title}</h1>
        <p className="muted sectionText">{detail.job.company}</p>
        <div className="cluster">
          <StatusBadge status={detail.status} />
          {detail.appliedAt && (
            <span className="muted metaDate">Applied: {formatDate(detail.appliedAt)}</span>
          )}
          {detail.dueDate && (
            <span className="muted metaDate">Due: {formatDate(detail.dueDate)}</span>
          )}
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
    <section className="card">
      <SectionHeader title="Application" />
      <form
        className="formStack"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <div className="formGrid">
          <label>
            Status
            <select value={status} onChange={(event) => setStatus(event.target.value as ApplicationStatus)}>
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
        <div className="formActions formActions-between">
          <Button type="button" variant="ghost" size="sm" onClick={clearDueDate} disabled={isPending || !dueDate}>
            Clear due date
          </Button>
          <Button type="submit" disabled={isPending || !hasChanges}>
            {isPending ? 'Saving...' : 'Save application'}
          </Button>
        </div>
      </form>
    </section>
  );
}

export function JobDetailsSection({ detail }: { detail: ApplicationDetail }) {
  const { job } = detail;

  return (
    <section className="card">
      <SectionHeader title="Job Details" />
      <div className="inlineMeta">
        {job.url && (
          <span className="muted metaDate">
            Source:{' '}
            <a href={job.url} target="_blank" rel="noopener noreferrer" className="linkBtn">
              {job.url}
            </a>
          </span>
        )}
        <span className="muted metaDate">Posted: {formatDate(job.createdAt)}</span>
      </div>
      <DescriptionBlock text={job.description || 'No description available.'} />
    </section>
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
    <section className="card">
      <SectionHeader title="Notes" />
      <form
        className="formStack"
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
        <div className="formActions">
          <Button type="submit" disabled={isPending || !noteContent.trim()}>
            {isPending ? 'Saving...' : 'Add note'}
          </Button>
        </div>
      </form>

      {notes.length === 0 ? (
        <EmptyState message="No notes yet" />
      ) : (
        <div className="surfaceList">
          {notes.map((note) => (
            <div key={note.id} className="surfaceItem">
              <p className="surfaceItemTitle">{note.content}</p>
              <span className="muted surfaceItemMeta">{formatDate(note.createdAt)}</span>
            </div>
          ))}
        </div>
      )}
    </section>
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
    <section className="card">
      <SectionHeader title="Contacts" />

      <form
        className="formStack"
        onSubmit={(event) => {
          event.preventDefault();
          onLinkExisting();
        }}
      >
        <p className="muted sectionText">Attach existing contact</p>
        {contactsLoading ? (
          <p className="muted sectionText">Loading contacts...</p>
        ) : availableContacts.length === 0 ? (
          <EmptyState message="No unlinked contacts available yet." />
        ) : (
          <div className="formGrid">
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
        )}
        <div className="formActions">
          <Button type="submit" disabled={linkPending || availableContacts.length === 0 || !existingContactId}>
            {linkPending ? 'Linking...' : 'Link contact'}
          </Button>
        </div>
      </form>

      <form
        className="formStack"
        onSubmit={(event) => {
          event.preventDefault();
          onCreateAndLink();
        }}
      >
        <p className="muted sectionText">Create and link new contact</p>
        <div className="formGrid">
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
        <div className="formActions">
          <Button type="submit" disabled={createPending || !newContact.name.trim()}>
            {createPending ? 'Saving...' : 'Create contact'}
          </Button>
        </div>
      </form>

      {detail.contacts.length === 0 ? (
        <EmptyState message="No contacts yet" />
      ) : (
        <div className="surfaceList">
          {detail.contacts.map((applicationContact) => (
            <ContactCard key={applicationContact.id} item={applicationContact} />
          ))}
        </div>
      )}
    </section>
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
    <section className="card">
      <SectionHeader title="Offer" />

      {detail.offer ? (
        <div className="inlineMeta">
          <span className="statusPill">{detail.offer.status}</span>
          {compensationLabel && (
            <span className="muted metaDate">Compensation: {compensationLabel}</span>
          )}
          {detail.offer.startsAt && (
            <span className="muted metaDate">Start: {formatDate(detail.offer.startsAt)}</span>
          )}
        </div>
      ) : (
        <p className="muted sectionText">No offer saved yet.</p>
      )}

      <form
        className="formStack"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <div className="formGrid">
          <label>
            Status
            <select value={status} onChange={(event) => setStatus(event.target.value as OfferStatus)}>
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
        <div className="formActions">
          <Button type="submit" disabled={isPending}>
            {isPending ? 'Saving...' : detail.offer ? 'Update offer' : 'Save offer'}
          </Button>
        </div>
      </form>
    </section>
  );
}

export function ActivitiesSection({ activities }: { activities: ApplicationDetail['activities'] }) {
  return (
    <section className="card">
      <SectionHeader title="Activities" />
      {activities.length === 0 ? (
        <EmptyState message="No activities yet" />
      ) : (
        <div className="stackList">
          {activities.map((activity) => (
            <div key={activity.id} className="activityItem">
              <span className="badge activityType">{formatEnumLabel(activity.type)}</span>
              <div className="stackXs">
                <p className="surfaceItemTitle">{activity.description}</p>
                <span className="muted surfaceItemMeta">{formatDate(activity.happenedAt)}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

export function TasksSection({ tasks }: { tasks: ApplicationDetail['tasks'] }) {
  return (
    <section className="card">
      <SectionHeader title="Tasks" />
      {tasks.length === 0 ? (
        <EmptyState message="No tasks yet" />
      ) : (
        <div className="stackList">
          {tasks.map((task) => (
            <div key={task.id} className={`taskItem ${task.done ? 'taskItem-done' : ''}`}>
              <input type="checkbox" checked={task.done} readOnly className="taskCheckbox" />
              <span className={`taskTitle ${task.done ? 'taskTitle-done' : ''}`}>{task.title}</span>
              {task.remindAt && (
                <span className="muted surfaceItemMeta">Remind: {formatDate(task.remindAt)}</span>
              )}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function DescriptionBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const limit = 600;
  const shouldTruncate = text.length > limit;
  const displayed = expanded || !shouldTruncate ? text : `${text.slice(0, limit)}...`;

  return (
    <div className="resultSection">
      <pre className="jobDescription">{displayed}</pre>
      {shouldTruncate && (
        <button
          type="button"
          className="descriptionToggle"
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? 'Show less' : 'Show more'}
        </button>
      )}
    </div>
  );
}

function ContactCard({ item }: { item: ApplicationContact }) {
  const contact = item.contact;

  return (
    <div className="surfaceItem contactCard">
      <div className="clusterStart">
        <span className="contactName">{contact.name}</span>
        <span className="badge badge-secondary contactRoleTag">{formatEnumLabel(item.relationship)}</span>
      </div>
      {(contact.role || contact.company) && (
        <span className="muted helperText">
          {[contact.role, contact.company].filter(Boolean).join(' at ')}
        </span>
      )}
      <div className="inlineMetaCompact">
        {contact.email && (
          <a href={`mailto:${contact.email}`} className="linkBtn helperText">
            {contact.email}
          </a>
        )}
        {contact.phone && <span className="muted helperText">{contact.phone}</span>}
        {contact.linkedinUrl && (
          <a
            href={contact.linkedinUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="linkBtn helperText"
          >
            LinkedIn
          </a>
        )}
      </div>
    </div>
  );
}
