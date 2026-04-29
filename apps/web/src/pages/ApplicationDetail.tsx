import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useMutation } from '@tanstack/react-query';
import {
  AlertCircle,
  ChevronDown,
  ChevronUp,
  Loader2,
  MessageSquare,
  Sparkles,
} from 'lucide-react';
import type { ApplicationDetail as ApplicationDetailRecord } from '@job-copilot/shared';

import { getInterviewPrep, type InterviewPrep } from '../api/enrichment';
import { analyzeFit, type FitAnalysis } from '../api/jobs';
import { Button } from '../components/ui/Button';
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
import { InnerPanel, Panel } from '../features/application-detail/ApplicationDetailLayout';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { useApplicationDetail } from '../features/application-detail/useApplicationDetail';
import { readProfileId } from '../lib/profileSession';

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

        <div className="space-y-6">
          <InterviewPrepSection detail={detail} />

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

function buildDeterministicFit(fit: FitAnalysis) {
  return {
    jobId: fit.jobId,
    score: fit.score,
    scoreBreakdown: fit.scoreBreakdown,
    matchedRoles: fit.matchedRoles,
    matchedSkills: fit.matchedSkills,
    matchedKeywords: fit.matchedKeywords,
    missingSignals: fit.missingTerms,
    sourceMatch: false,
    workModeMatch: undefined,
    regionMatch: undefined,
    descriptionQuality: fit.descriptionQuality,
    positiveReasons: fit.positiveReasons,
    negativeReasons: fit.negativeReasons,
    reasons: [...fit.positiveReasons, ...fit.negativeReasons],
  };
}

function InterviewPrepSection({ detail }: { detail: ApplicationDetailRecord }) {
  const profileId = readProfileId();
  const [interviewPrep, setInterviewPrep] = useState<InterviewPrep | null>(null);
  const [expanded, setExpanded] = useState(false);

  const mutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Create a profile to generate interview prep.');
      }

      const fit = await analyzeFit(profileId, detail.job.id);
      return getInterviewPrep({
        profileId,
        analyzedProfile: null,
        searchProfile: null,
        rankedJob: detail.job,
        deterministicFit: buildDeterministicFit(fit),
      });
    },
    onSuccess: (prep) => {
      setInterviewPrep(prep);
      setExpanded(true);
    },
  });

  function generateInterviewPrep() {
    mutation.mutate();
  }

  const hasResults = Boolean(interviewPrep);

  return (
    <Panel
      title="Prepare for Interview"
      description="Ephemeral interview prep for this application. Generate again whenever you reopen the page."
      icon={MessageSquare}
    >
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <Button
          onClick={generateInterviewPrep}
          disabled={mutation.isPending || !profileId}
          variant="outline"
          className="gap-2"
        >
          {mutation.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Sparkles className="h-4 w-4" />
          )}
          {mutation.isPending ? 'Generating Interview Prep' : 'Generate Interview Prep'}
        </Button>

        {hasResults ? (
          <Button
            onClick={() => setExpanded((value) => !value)}
            variant="ghost"
            size="sm"
            className="gap-2"
          >
            {expanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
            {expanded ? 'Hide prep' : 'Show prep'}
          </Button>
        ) : null}
      </div>

      {!profileId ? (
        <div className="mt-4 rounded-2xl border border-border/70 bg-surface-muted p-4 text-sm leading-6 text-muted-foreground">
          Create a profile to generate interview prep.
        </div>
      ) : null}

      {mutation.isError ? (
        <div className="mt-4 rounded-2xl border border-danger/30 bg-danger/10 p-4">
          <div className="flex items-start gap-3">
            <AlertCircle className="mt-0.5 h-5 w-5 shrink-0 text-danger" />
            <div className="min-w-0 flex-1">
              <p className="m-0 text-sm font-medium text-card-foreground">
                Interview prep is unavailable right now.
              </p>
              <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">
                {mutation.error instanceof Error ? mutation.error.message : 'Try again.'}
              </p>
              <Button
                onClick={generateInterviewPrep}
                disabled={mutation.isPending}
                variant="outline"
                size="sm"
                className="mt-3"
              >
                Retry
              </Button>
            </div>
          </div>
        </div>
      ) : null}

      {interviewPrep && expanded ? <InterviewPrepResults prep={interviewPrep} /> : null}
    </Panel>
  );
}

function InterviewPrepResults({ prep }: { prep: InterviewPrep }) {
  return (
    <div className="mt-5 space-y-4">
      {prep.prepSummary ? (
        <div className="rounded-2xl border border-primary/20 bg-primary/10 p-4">
          <p className="m-0 text-sm leading-7 text-card-foreground">{prep.prepSummary}</p>
        </div>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-3">
        <ListPanel title="Common interview questions" items={prep.likelyTopics} />
        <ListPanel
          title="Suggested talking points"
          items={[...prep.storiesToPrepare, ...prep.followUpPlan]}
        />
        <ListPanel
          title="Skills to demonstrate"
          items={[...prep.technicalFocus, ...prep.behavioralFocus]}
        />
      </div>
    </div>
  );
}

function ListPanel({ title, items }: { title: string; items: string[] }) {
  return (
    <InnerPanel title={title}>
      {items.length === 0 ? (
        <p className="m-0 text-sm text-muted-foreground">No suggestions returned.</p>
      ) : (
        <div className="space-y-2">
          {items.map((item) => (
            <div key={item} className="flex items-start gap-3">
              <span className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-primary" />
              <p className="m-0 text-sm leading-6 text-muted-foreground">{item}</p>
            </div>
          ))}
        </div>
      )}
    </InnerPanel>
  );
}
