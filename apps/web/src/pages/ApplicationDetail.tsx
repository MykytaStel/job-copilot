import { useParams } from 'react-router-dom';

import {
  ActivitiesSection,
  ApplicationFormSection,
  ApplicationHeader,
  ContactsSection,
  JobDetailsSection,
  NotesSection,
  OfferSection,
  TasksSection,
} from '../features/application-detail/ApplicationDetailSections';
import { useApplicationDetail } from '../features/application-detail/useApplicationDetail';

export default function ApplicationDetail() {
  const { id } = useParams<{ id: string }>();
  const {
    detailQuery,
    contactsQuery,
    detail,
    availableContacts,
    compensationLabel,
    applicationForm,
    noteForm,
    existingContactForm,
    newContactForm,
    offerForm,
  } = useApplicationDetail(id);

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

  return (
    <div className="jobDetails">
      <ApplicationHeader detail={detail} />

      <ApplicationFormSection
        status={applicationForm.applicationStatus}
        dueDate={applicationForm.dueDate}
        isPending={applicationForm.isPending}
        hasChanges={applicationForm.hasApplicationChanges}
        setStatus={applicationForm.setApplicationStatus}
        setDueDate={applicationForm.setDueDate}
        clearDueDate={applicationForm.clearDueDate}
        onSubmit={applicationForm.saveApplication}
      />

      <JobDetailsSection detail={detail} />

      <NotesSection
        notes={detail.notes}
        noteContent={noteForm.noteContent}
        isPending={noteForm.isPending}
        setNoteContent={noteForm.setNoteContent}
        onSubmit={noteForm.addApplicationNote}
      />

      <ContactsSection
        detail={detail}
        contactsLoading={contactsQuery.isLoading}
        availableContacts={availableContacts}
        existingContactId={existingContactForm.existingContactId}
        existingRelationship={existingContactForm.existingRelationship}
        linkPending={existingContactForm.isPending}
        setExistingContactId={existingContactForm.setExistingContactId}
        setExistingRelationship={existingContactForm.setExistingRelationship}
        onLinkExisting={existingContactForm.linkExistingContact}
        newContact={newContactForm.newContact}
        newContactRelationship={newContactForm.newContactRelationship}
        createPending={newContactForm.isPending}
        setNewContactField={newContactForm.setNewContactField}
        setNewContactRelationship={newContactForm.setNewContactRelationship}
        onCreateAndLink={newContactForm.createAndLinkContact}
      />

      <OfferSection
        detail={detail}
        compensationLabel={compensationLabel}
        status={offerForm.offerStatus}
        min={offerForm.offerMin}
        max={offerForm.offerMax}
        currency={offerForm.offerCurrency}
        startsAt={offerForm.offerStartsAt}
        notes={offerForm.offerNotes}
        isPending={offerForm.isPending}
        setStatus={offerForm.setOfferStatus}
        setMin={offerForm.setOfferMin}
        setMax={offerForm.setOfferMax}
        setCurrency={offerForm.setOfferCurrency}
        setStartsAt={offerForm.setOfferStartsAt}
        setNotes={offerForm.setOfferNotes}
        onSubmit={offerForm.saveOffer}
      />

      <ActivitiesSection activities={detail.activities} />
      <TasksSection tasks={detail.tasks} />
    </div>
  );
}
