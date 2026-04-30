import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Building2, ExternalLink, Mail, Phone, Plus, Search, UserRound, X } from 'lucide-react';

import { createContact, getContacts } from '../api/contacts';
import { useToast } from '../context/ToastContext';
import type { Contact, ContactInput } from '@job-copilot/shared/applications';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { cn } from '../lib/cn';
import { queryKeys } from '../queryKeys';

export default function Contacts() {
  const [search, setSearch] = useState('');
  const [isFormOpen, setIsFormOpen] = useState(false);

  const queryClient = useQueryClient();
  const { showToast } = useToast();

  const { data: contacts = [], isLoading } = useQuery({
    queryKey: queryKeys.contacts.all(),
    queryFn: getContacts,
  });

  const createMutation = useMutation({
    mutationFn: createContact,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.contacts.all() });
      setIsFormOpen(false);
      showToast({ type: 'success', message: 'Contact added' });
    },
    onError: () => {
      showToast({ type: 'error', message: 'Failed to add contact' });
    },
  });

  const filtered = contacts.filter((c) => {
    const q = search.toLowerCase();
    return (
      c.name.toLowerCase().includes(q) ||
      (c.company ?? '').toLowerCase().includes(q) ||
      (c.role ?? '').toLowerCase().includes(q)
    );
  });

  return (
    <Page>
      <PageHeader
        title="Contacts"
        description="Recruiters, hiring managers, and referrers linked to your applications."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Contacts' }]}
      />

      <div className="flex flex-col gap-4">
        <div className="flex items-center gap-3">
          <div className="relative flex-1">
            <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              type="search"
              placeholder="Search by name, company, or role…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-10 w-full rounded-[var(--radius-lg)] border border-border bg-surface-muted pl-9 pr-3 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/40"
            />
          </div>
          <Button onClick={() => setIsFormOpen(true)} size="md">
            <Plus className="h-4 w-4" />
            New contact
          </Button>
        </div>

        {isLoading ? (
          <div className="py-16 text-center text-sm text-muted-foreground">Loading contacts…</div>
        ) : contacts.length === 0 ? (
          <EmptyState
            message="No contacts yet"
            description="Add recruiters, hiring managers, or referrers to keep track of your network."
          />
        ) : filtered.length === 0 ? (
          <EmptyState message="No contacts match your search" />
        ) : (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {filtered.map((contact) => (
              <ContactCard key={contact.id} contact={contact} />
            ))}
          </div>
        )}
      </div>

      {isFormOpen && (
        <NewContactModal
          onClose={() => setIsFormOpen(false)}
          onSubmit={(payload) => createMutation.mutate(payload)}
          isPending={createMutation.isPending}
        />
      )}
    </Page>
  );
}

function ContactCard({ contact }: { contact: Contact }) {
  return (
    <div className="flex flex-col gap-3 rounded-[var(--radius-card)] border border-border bg-card/90 p-4 shadow-[var(--shadow-card)]">
      <div className="flex items-start gap-3">
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
          <UserRound className="h-4 w-4" />
        </div>
        <div className="min-w-0">
          <p className="truncate font-semibold text-foreground">{contact.name}</p>
          {(contact.company || contact.role) && (
            <p className="truncate text-xs text-muted-foreground">
              {[contact.role, contact.company].filter(Boolean).join(' · ')}
            </p>
          )}
        </div>
      </div>

      <div className="flex flex-col gap-1.5">
        {contact.email && (
          <a
            href={`mailto:${contact.email}`}
            className="flex items-center gap-2 text-xs text-muted-foreground hover:text-primary"
          >
            <Mail className="h-3.5 w-3.5 shrink-0" />
            <span className="truncate">{contact.email}</span>
          </a>
        )}
        {contact.phone && (
          <a
            href={`tel:${contact.phone}`}
            className="flex items-center gap-2 text-xs text-muted-foreground hover:text-primary"
          >
            <Phone className="h-3.5 w-3.5 shrink-0" />
            <span className="truncate">{contact.phone}</span>
          </a>
        )}
        {contact.company && !contact.email && !contact.phone && (
          <span className="flex items-center gap-2 text-xs text-muted-foreground">
            <Building2 className="h-3.5 w-3.5 shrink-0" />
            <span className="truncate">{contact.company}</span>
          </span>
        )}
        {contact.linkedinUrl && (
          <a
            href={contact.linkedinUrl}
            target="_blank"
            rel="noreferrer noopener"
            className="flex items-center gap-2 text-xs text-muted-foreground hover:text-primary"
          >
            <ExternalLink className="h-3.5 w-3.5 shrink-0" />
            LinkedIn
          </a>
        )}
      </div>
    </div>
  );
}

function NewContactModal({
  onClose,
  onSubmit,
  isPending,
}: {
  onClose: () => void;
  onSubmit: (payload: ContactInput) => void;
  isPending: boolean;
}) {
  const [form, setForm] = useState<ContactInput>({
    name: '',
    email: '',
    phone: '',
    linkedinUrl: '',
    company: '',
    role: '',
  });

  function update(field: keyof ContactInput, value: string) {
    setForm((prev) => ({ ...prev, [field]: value || undefined }));
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!form.name.trim()) return;
    onSubmit({ ...form, name: form.name.trim() });
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4 py-6 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="new-contact-title"
      onMouseDown={onClose}
    >
      <div
        className="flex w-full max-w-md flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border bg-surface shadow-2xl"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-border/80 px-5 py-4">
          <h2 id="new-contact-title" className="text-base font-semibold text-foreground">
            New contact
          </h2>
          <Button type="button" variant="icon" size="icon" aria-label="Close" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </header>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4 p-5">
          <Field label="Name *" required>
            <input
              type="text"
              required
              placeholder="Jane Smith"
              value={form.name}
              onChange={(e) => update('name', e.target.value)}
              className={inputClass}
            />
          </Field>

          <div className="grid grid-cols-2 gap-3">
            <Field label="Company">
              <input
                type="text"
                placeholder="Acme Corp"
                value={form.company ?? ''}
                onChange={(e) => update('company', e.target.value)}
                className={inputClass}
              />
            </Field>
            <Field label="Role">
              <input
                type="text"
                placeholder="Recruiter"
                value={form.role ?? ''}
                onChange={(e) => update('role', e.target.value)}
                className={inputClass}
              />
            </Field>
          </div>

          <Field label="Email">
            <input
              type="email"
              placeholder="jane@example.com"
              value={form.email ?? ''}
              onChange={(e) => update('email', e.target.value)}
              className={inputClass}
            />
          </Field>

          <Field label="Phone">
            <input
              type="tel"
              placeholder="+380 xx xxx xxxx"
              value={form.phone ?? ''}
              onChange={(e) => update('phone', e.target.value)}
              className={inputClass}
            />
          </Field>

          <Field label="LinkedIn URL">
            <input
              type="url"
              placeholder="https://linkedin.com/in/…"
              value={form.linkedinUrl ?? ''}
              onChange={(e) => update('linkedinUrl', e.target.value)}
              className={inputClass}
            />
          </Field>

          <div className="flex justify-end gap-2 pt-1">
            <Button type="button" variant="outline" onClick={onClose} disabled={isPending}>
              Cancel
            </Button>
            <Button type="submit" disabled={isPending || !form.name.trim()}>
              {isPending ? 'Saving…' : 'Save contact'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}

function Field({
  label,
  required,
  children,
}: {
  label: string;
  required?: boolean;
  children: React.ReactNode;
}) {
  return (
    <label className={cn('flex flex-col gap-1', required && '')}>
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}

const inputClass =
  'h-9 w-full rounded-[var(--radius-md)] border border-border bg-surface-muted px-3 text-sm text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-2 focus:ring-primary/40';
