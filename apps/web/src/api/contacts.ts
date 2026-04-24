import type { ApplicationContact, Contact, ContactInput } from '@job-copilot/shared/applications';

import { json, request } from './client';
import type { EngineContact, EngineContactsResponse } from './engine-types';
import { mapContact } from './mappers';

export async function getContacts(): Promise<Contact[]> {
  const response = await request<EngineContactsResponse>('/api/v1/contacts');
  return response.contacts.map(mapContact);
}

export async function createContact(payload: ContactInput): Promise<Contact> {
  const contact = await request<EngineContact>(
    '/api/v1/contacts',
    json('POST', {
      name: payload.name,
      email: payload.email,
      phone: payload.phone,
      linkedin_url: payload.linkedinUrl,
      company: payload.company,
      role: payload.role,
    }),
  );

  return mapContact(contact);
}

export async function linkContact(
  applicationId: string,
  contactId: string,
  relationship: ApplicationContact['relationship'],
): Promise<ApplicationContact> {
  const contact = await request<{
    id: string;
    relationship: ApplicationContact['relationship'];
    contact: EngineContact;
  }>(
    `/api/v1/applications/${applicationId}/contacts`,
    json('POST', {
      contact_id: contactId,
      relationship,
    }),
  );

  return {
    id: contact.id,
    relationship: contact.relationship,
    contact: mapContact(contact.contact),
  };
}

