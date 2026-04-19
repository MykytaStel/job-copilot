import { useEffect, useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type {
  ApplicationContact,
  ApplicationStatus,
  ContactInput,
  ContactRelationship,
  OfferStatus,
} from '@job-copilot/shared';

import {
  addNote,
  createOffer,
  getApplicationDetail,
  updateApplication,
} from '../../api/applications';
import { getContacts, linkContact, createContact } from '../../api/contacts';
import { normalizeDateInput, parseOptionalNumber } from '../../lib/format';
import { queryKeys } from '../../queryKeys';
import { EMPTY_CONTACT_INPUT } from './applicationDetail.constants';
import { formatCompensation } from './applicationDetail.utils';

export function useApplicationDetail(id?: string) {
  const queryClient = useQueryClient();

  const [noteContent, setNoteContent] = useState('');
  const [existingContactId, setExistingContactId] = useState('');
  const [existingRelationship, setExistingRelationship] =
    useState<ContactRelationship>('recruiter');
  const [newContactRelationship, setNewContactRelationship] =
    useState<ContactRelationship>('recruiter');
  const [newContact, setNewContact] = useState<ContactInput>(EMPTY_CONTACT_INPUT);
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
      setNewContact(EMPTY_CONTACT_INPUT);
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

  const normalizedCurrentDueDate = normalizeDateInput(detail?.dueDate);
  const hasApplicationChanges = detail
    ? applicationStatus !== detail.status || dueDate !== normalizedCurrentDueDate
    : false;

  function saveApplication() {
    if (!hasApplicationChanges) return;
    applicationMutation.mutate();
  }

  function saveOffer() {
    offerMutation.mutate();
  }

  function addApplicationNote() {
    const content = noteContent.trim();
    if (!content) return;
    noteMutation.mutate(content);
  }

  function linkExistingContact() {
    if (!existingContactId) return;

    linkExistingContactMutation.mutate({
      contactId: existingContactId,
      relationship: existingRelationship,
    });
  }

  function createAndLinkContact() {
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
  }

  function setNewContactField<K extends keyof ContactInput>(
    field: K,
    value: ContactInput[K],
  ) {
    setNewContact((current) => ({ ...current, [field]: value }));
  }

  return {
    detailQuery,
    contactsQuery,
    detail,
    availableContacts,
    compensationLabel: detail ? formatCompensation(detail) : null,
    applicationForm: {
      applicationStatus,
      dueDate,
      hasApplicationChanges,
      isPending: applicationMutation.isPending,
      setApplicationStatus,
      setDueDate,
      clearDueDate: () => setDueDate(''),
      saveApplication,
    },
    noteForm: {
      noteContent,
      isPending: noteMutation.isPending,
      setNoteContent,
      addApplicationNote,
    },
    existingContactForm: {
      existingContactId,
      existingRelationship,
      isPending: linkExistingContactMutation.isPending,
      setExistingContactId,
      setExistingRelationship,
      linkExistingContact,
    },
    newContactForm: {
      newContact,
      newContactRelationship,
      isPending: createAndLinkContactMutation.isPending,
      setNewContactField,
      setNewContactRelationship,
      createAndLinkContact,
    },
    offerForm: {
      offerStatus,
      offerMin,
      offerMax,
      offerCurrency,
      offerStartsAt,
      offerNotes,
      isPending: offerMutation.isPending,
      setOfferStatus,
      setOfferMin,
      setOfferMax,
      setOfferCurrency,
      setOfferStartsAt,
      setOfferNotes,
      saveOffer,
    },
  };
}
