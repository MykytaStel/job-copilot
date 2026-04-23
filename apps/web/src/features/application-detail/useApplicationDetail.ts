import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type {
  ApplicationDetail,
  ApplicationContact,
  ApplicationOutcome,
  ApplicationStatus,
  ContactInput,
  ContactRelationship,
  OfferStatus,
  RejectionStage,
} from '@job-copilot/shared';

import {
  addNote,
  createOffer,
  getApplicationDetail,
  updateApplication,
} from '../../api/applications';
import { getContacts, linkContact, createContact } from '../../api/contacts';
import { normalizeDateInput, parseOptionalNumber } from '../../lib/format';
import { invalidateApplicationSummaryQueries } from '../../lib/queryInvalidation';
import { queryKeys } from '../../queryKeys';
import { EMPTY_CONTACT_INPUT } from './applicationDetail.constants';
import { formatCompensation } from './applicationDetail.utils';

type OfferFormState = {
  status: OfferStatus;
  min: string;
  max: string;
  currency: string;
  startsAt: string;
  notes: string;
};

type ApplicationFormState = {
  status: ApplicationStatus;
  dueDate: string;
  outcome: ApplicationOutcome | '';
  outcomeDate: string;
  rejectionStage: RejectionStage | '';
};

function getOfferFormState(offer: ApplicationDetail['offer']): OfferFormState {
  if (!offer) {
    return {
      status: 'draft',
      min: '',
      max: '',
      currency: 'USD',
      startsAt: '',
      notes: '',
    };
  }

  return {
    status: offer.status,
    min: offer.compensationMin?.toString() ?? '',
    max: offer.compensationMax?.toString() ?? '',
    currency: offer.compensationCurrency ?? 'USD',
    startsAt: normalizeDateInput(offer.startsAt),
    notes: offer.notes ?? '',
  };
}

function getApplicationFormState(
  detail: { status: ApplicationStatus; dueDate?: string; outcome?: ApplicationOutcome; outcomeDate?: string; rejectionStage?: RejectionStage } | undefined,
): ApplicationFormState {
  return {
    status: detail?.status ?? 'saved',
    dueDate: normalizeDateInput(detail?.dueDate),
    outcome: detail?.outcome ?? '',
    outcomeDate: normalizeDateInput(detail?.outcomeDate),
    rejectionStage: detail?.rejectionStage ?? '',
  };
}

export function useApplicationDetail(id?: string) {
  const queryClient = useQueryClient();

  const [noteContent, setNoteContent] = useState('');
  const [selectedExistingContactId, setSelectedExistingContactId] = useState('');
  const [existingRelationship, setExistingRelationship] =
    useState<ContactRelationship>('recruiter');
  const [newContactRelationship, setNewContactRelationship] =
    useState<ContactRelationship>('recruiter');
  const [newContact, setNewContact] = useState<ContactInput>(EMPTY_CONTACT_INPUT);
  const [offerDraft, setOfferDraft] = useState<OfferFormState | null>(null);
  const [applicationDraft, setApplicationDraft] = useState<ApplicationFormState | null>(null);

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
  const baseOfferForm = useMemo(() => getOfferFormState(detail?.offer), [detail?.offer]);
  const offerForm = offerDraft ?? baseOfferForm;
  const baseApplicationForm = useMemo(() => getApplicationFormState(detail), [detail]);
  const applicationForm = applicationDraft ?? baseApplicationForm;

  const availableContacts = useMemo(() => {
    if (!detail || !contactsQuery.data) return [];

    const attachedIds = new Set(detail.contacts.map((item) => item.contact.id));
    return contactsQuery.data.filter((contact) => !attachedIds.has(contact.id));
  }, [contactsQuery.data, detail]);
  const existingContactId =
    selectedExistingContactId &&
    availableContacts.some((contact) => contact.id === selectedExistingContactId)
      ? selectedExistingContactId
      : (availableContacts[0]?.id ?? '');

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
        status: offerForm.status,
        compensationMin: parseOptionalNumber(offerForm.min),
        compensationMax: parseOptionalNumber(offerForm.max),
        compensationCurrency: offerForm.currency.trim() || undefined,
        startsAt: offerForm.startsAt || undefined,
        notes: offerForm.notes.trim() || undefined,
      }),
    onSuccess: async () => {
      setOfferDraft(null);
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
        status: applicationForm.status,
        dueDate: applicationForm.dueDate || null,
        outcome: applicationForm.outcome || null,
        outcomeDate: applicationForm.outcomeDate || null,
        rejectionStage: applicationForm.rejectionStage || null,
      }),
    onSuccess: async () => {
      setApplicationDraft(null);
      await Promise.all([refreshDetail(), invalidateApplicationSummaryQueries(queryClient)]);
      toast.success('Application updated');
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : 'Failed to update application');
    },
  });

  const hasApplicationChanges = detail
    ? applicationForm.status !== baseApplicationForm.status ||
      applicationForm.dueDate !== baseApplicationForm.dueDate ||
      applicationForm.outcome !== baseApplicationForm.outcome ||
      applicationForm.outcomeDate !== baseApplicationForm.outcomeDate ||
      applicationForm.rejectionStage !== baseApplicationForm.rejectionStage
    : false;

  function setApplicationStatus(value: ApplicationStatus) {
    setApplicationDraft((current) => ({
      ...(current ?? applicationForm),
      status: value,
    }));
  }

  function setDueDate(value: string) {
    setApplicationDraft((current) => ({
      ...(current ?? applicationForm),
      dueDate: value,
    }));
  }

  function setOutcome(value: ApplicationOutcome | '') {
    setApplicationDraft((current) => ({
      ...(current ?? applicationForm),
      outcome: value,
    }));
  }

  function setOutcomeDate(value: string) {
    setApplicationDraft((current) => ({
      ...(current ?? applicationForm),
      outcomeDate: value,
    }));
  }

  function setRejectionStage(value: RejectionStage | '') {
    setApplicationDraft((current) => ({
      ...(current ?? applicationForm),
      rejectionStage: value,
    }));
  }

  function setOfferStatus(value: OfferStatus) {
    setOfferDraft((current) => ({
      ...(current ?? offerForm),
      status: value,
    }));
  }

  function setOfferMin(value: string) {
    setOfferDraft((current) => ({
      ...(current ?? offerForm),
      min: value,
    }));
  }

  function setOfferMax(value: string) {
    setOfferDraft((current) => ({
      ...(current ?? offerForm),
      max: value,
    }));
  }

  function setOfferCurrency(value: string) {
    setOfferDraft((current) => ({
      ...(current ?? offerForm),
      currency: value,
    }));
  }

  function setOfferStartsAt(value: string) {
    setOfferDraft((current) => ({
      ...(current ?? offerForm),
      startsAt: value,
    }));
  }

  function setOfferNotes(value: string) {
    setOfferDraft((current) => ({
      ...(current ?? offerForm),
      notes: value,
    }));
  }

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

  function setNewContactField<K extends keyof ContactInput>(field: K, value: ContactInput[K]) {
    setNewContact((current) => ({ ...current, [field]: value }));
  }

  return {
    detailQuery,
    contactsQuery,
    detail,
    availableContacts,
    compensationLabel: detail ? formatCompensation(detail) : null,
    applicationForm: {
      applicationStatus: applicationForm.status,
      dueDate: applicationForm.dueDate,
      outcome: applicationForm.outcome,
      outcomeDate: applicationForm.outcomeDate,
      rejectionStage: applicationForm.rejectionStage,
      hasApplicationChanges,
      isPending: applicationMutation.isPending,
      setApplicationStatus,
      setDueDate,
      clearDueDate: () => setDueDate(''),
      setOutcome,
      setOutcomeDate,
      setRejectionStage,
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
      setExistingContactId: setSelectedExistingContactId,
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
      offerStatus: offerForm.status,
      offerMin: offerForm.min,
      offerMax: offerForm.max,
      offerCurrency: offerForm.currency,
      offerStartsAt: offerForm.startsAt,
      offerNotes: offerForm.notes,
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
