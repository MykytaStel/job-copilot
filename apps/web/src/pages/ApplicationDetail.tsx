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
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { useApplicationDetail } from '../features/application-detail/useApplicationDetail';

export default function ApplicationDetail() {
  const { id } = useParams<{ id: string }>();

  if (!id)
    return (
      <Page>
        <EmptyState message="Application not found" />
      </Page>
    );

  return <ApplicationDetailContent key={id} id={id} />;
}

function ApplicationDetailContent({ id }: { id: string }) {
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

  if (detailQuery.isLoading)
    return (
      <Page>
        <EmptyState message="Loading..." />
      </Page>
    );
  if (detailQuery.error || !detail) {
    return (
      <Page>
        <EmptyState
          message={
            detailQuery.error instanceof Error ? detailQuery.error.message : 'Application not found'
          }
        />
      </Page>
    );
  }

  return (
    <Page>
      <PageHeader
        title="Application Detail"
        description="Track pipeline status, linked contacts, offer state, notes, and task follow-up for a single opportunity."
        breadcrumb={[
          { label: 'Dashboard', href: '/' },
          { label: 'Applications', href: '/applications' },
          { label: detail.job.company },
        ]}
      />

      <ApplicationHeader detail={detail} />

      <div className="grid gap-6 lg:grid-cols-2">
        <ApplicationFormSection
          status={applicationForm.applicationStatus}
          dueDate={applicationForm.dueDate}
          outcome={applicationForm.outcome}
          outcomeDate={applicationForm.outcomeDate}
          rejectionStage={applicationForm.rejectionStage}
          isPending={applicationForm.isPending}
          hasChanges={applicationForm.hasApplicationChanges}
          setStatus={applicationForm.setApplicationStatus}
          setDueDate={applicationForm.setDueDate}
          clearDueDate={applicationForm.clearDueDate}
          setOutcome={applicationForm.setOutcome}
          setOutcomeDate={applicationForm.setOutcomeDate}
          setRejectionStage={applicationForm.setRejectionStage}
          onSubmit={applicationForm.saveApplication}
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
      </div>

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(360px,0.9fr)]">
        <JobDetailsSection detail={detail} />

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
      </div>

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <NotesSection
          notes={detail.notes}
          noteContent={noteForm.noteContent}
          isPending={noteForm.isPending}
          setNoteContent={noteForm.setNoteContent}
          onSubmit={noteForm.addApplicationNote}
        />

        <div className="space-y-6">
          <TasksSection tasks={detail.tasks} />
          <ActivitiesSection activities={detail.activities} />
        </div>
      </div>
    </Page>
  );
}
