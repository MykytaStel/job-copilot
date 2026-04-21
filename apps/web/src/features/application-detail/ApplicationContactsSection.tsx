import type { ApplicationDetail, Contact, ContactInput, ContactRelationship } from '@job-copilot/shared';
import { Users } from 'lucide-react';

import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { formatEnumLabel } from '../../lib/format';
import { RELATIONSHIP_OPTIONS } from './applicationDetail.constants';
import { ContactCard } from './ApplicationDetailCards';
import { InnerPanel, Panel } from './ApplicationDetailLayout';

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
