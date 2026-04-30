/* eslint-disable react-refresh/only-export-components */

import type { CompanyFeedbackRecord } from '@job-copilot/shared/feedback';
import type { JobPosting } from '@job-copilot/shared';
import type { LucideIcon } from 'lucide-react';
import { Bookmark, Clock3, EyeOff, ThumbsDown } from 'lucide-react';
import { Link } from 'react-router-dom';

import type { FeedbackTimelineItem } from '../../api/feedback';
import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import {
  CompanyPanel,
  CompanyRow,
  JobRow,
  Section,
  type FeedbackListTone,
} from './FeedbackCenterComponents';

export function filterJobsBySearch(jobs: JobPosting[], normalizedSearch: string) {
  return jobs.filter((job) => {
    if (!normalizedSearch) return true;

    const title = job.presentation?.title ?? job.title;
    const company = job.presentation?.company ?? job.company;

    return (
      title.toLowerCase().includes(normalizedSearch) ||
      company.toLowerCase().includes(normalizedSearch)
    );
  });
}

function JobFeedbackSection({
  title,
  description,
  icon: Icon,
  jobs,
  tone,
  emptyMessage,
  actionLabel,
  onAction,
  isPending,
  selectedJobIds,
  selectedJobsCount,
  allVisibleJobsSelected,
  onSelectAllVisible,
  onToggleSelected,
  onClearSelection,
  onBulkHide,
  onBulkBadFit,
  onBulkSave,
  isBulkPending,
}: {
  title: string;
  description: string;
  icon: LucideIcon;
  jobs: JobPosting[];
  tone: FeedbackListTone;
  emptyMessage: string;
  actionLabel: string;
  onAction: (jobId: string) => void;
  isPending: boolean;
  selectedJobIds: Set<string>;
  selectedJobsCount: number;
  allVisibleJobsSelected: boolean;
  onSelectAllVisible: (isSelected: boolean) => void;
  onToggleSelected: (jobId: string) => void;
  onClearSelection: () => void;
  onBulkHide: () => void;
  onBulkBadFit: () => void;
  onBulkSave: () => void;
  isBulkPending: boolean;
}) {
  return (
    <Section title={title} icon={<Icon size={16} />} description={description} count={jobs.length}>
      {jobs.length === 0 ? (
        <EmptyState
          icon={<Icon className="h-10 w-10" />}
          message={emptyMessage}
          description="Use the job feed to save, hide, or mark roles as bad fit."
          action={
            <Link to="/" className="inline-flex no-underline">
              <Button size="sm" variant="outline">
                Find jobs
              </Button>
            </Link>
          }
          className="px-4 py-5"
        />
      ) : (
        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-3 rounded-lg border border-border bg-surface-muted px-3 py-3 md:flex-row md:items-center md:justify-between">
            <label className="flex items-center gap-2 text-sm font-medium text-card-foreground">
              <input
                type="checkbox"
                checked={allVisibleJobsSelected}
                onChange={(event) => onSelectAllVisible(event.target.checked)}
                className="h-4 w-4 rounded border-border accent-primary"
              />
              Select all visible
            </label>
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-xs text-muted-foreground">
                {selectedJobsCount} selected
              </span>
              {selectedJobsCount > 0 ? (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-8 rounded-lg"
                  onClick={onClearSelection}
                  disabled={isBulkPending}
                >
                  Clear
                </Button>
              ) : null}
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-8 rounded-lg"
                onClick={onBulkHide}
                disabled={selectedJobsCount === 0 || isBulkPending}
              >
                <EyeOff className="h-3.5 w-3.5" />
                Hide selected
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-8 rounded-lg"
                onClick={onBulkBadFit}
                disabled={selectedJobsCount === 0 || isBulkPending}
              >
                <ThumbsDown className="h-3.5 w-3.5" />
                Mark selected bad-fit
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-8 rounded-lg"
                onClick={onBulkSave}
                disabled={selectedJobsCount === 0 || isBulkPending}
              >
                <Bookmark className="h-3.5 w-3.5" />
                Save selected
              </Button>
            </div>
          </div>
          {jobs.map((job) => (
            <JobRow
              key={job.id}
              job={job}
              tone={tone}
              actionLabel={actionLabel}
              onAction={onAction}
              isPending={isPending}
              isSelected={selectedJobIds.has(job.id)}
              onSelectedChange={onToggleSelected}
            />
          ))}
        </div>
      )}
    </Section>
  );
}

export function SavedJobsSection({
  jobs,
  searchQuery,
  onUnsave,
  isPending,
  selectedJobIds,
  selectedJobsCount,
  allVisibleJobsSelected,
  onSelectAllVisible,
  onToggleSelected,
  onClearSelection,
  onBulkHide,
  onBulkBadFit,
  onBulkSave,
  isBulkPending,
}: {
  jobs: JobPosting[];
  searchQuery: string;
  onUnsave: (jobId: string) => void;
  isPending: boolean;
  selectedJobIds: Set<string>;
  selectedJobsCount: number;
  allVisibleJobsSelected: boolean;
  onSelectAllVisible: (isSelected: boolean) => void;
  onToggleSelected: (jobId: string) => void;
  onClearSelection: () => void;
  onBulkHide: () => void;
  onBulkBadFit: () => void;
  onBulkSave: () => void;
  isBulkPending: boolean;
}) {
  return (
    <JobFeedbackSection
      title="Saved Jobs"
      description="High-confidence jobs you kept for follow-up, tailoring, or application."
      icon={Bookmark}
      jobs={jobs}
      tone="saved"
      emptyMessage={searchQuery ? 'No saved jobs match this query.' : 'No saved jobs.'}
      actionLabel="Unsave"
      onAction={onUnsave}
      isPending={isPending}
      selectedJobIds={selectedJobIds}
      selectedJobsCount={selectedJobsCount}
      allVisibleJobsSelected={allVisibleJobsSelected}
      onSelectAllVisible={onSelectAllVisible}
      onToggleSelected={onToggleSelected}
      onClearSelection={onClearSelection}
      onBulkHide={onBulkHide}
      onBulkBadFit={onBulkBadFit}
      onBulkSave={onBulkSave}
      isBulkPending={isBulkPending}
    />
  );
}

export function HiddenJobsSection({
  jobs,
  searchQuery,
  onUnhide,
  isPending,
  selectedJobIds,
  selectedJobsCount,
  allVisibleJobsSelected,
  onSelectAllVisible,
  onToggleSelected,
  onClearSelection,
  onBulkHide,
  onBulkBadFit,
  onBulkSave,
  isBulkPending,
}: {
  jobs: JobPosting[];
  searchQuery: string;
  onUnhide: (jobId: string) => void;
  isPending: boolean;
  selectedJobIds: Set<string>;
  selectedJobsCount: number;
  allVisibleJobsSelected: boolean;
  onSelectAllVisible: (isSelected: boolean) => void;
  onToggleSelected: (jobId: string) => void;
  onClearSelection: () => void;
  onBulkHide: () => void;
  onBulkBadFit: () => void;
  onBulkSave: () => void;
  isBulkPending: boolean;
}) {
  return (
    <JobFeedbackSection
      title="Hidden Jobs"
      description="Suppressed jobs stay out of the main feed until you restore them."
      icon={EyeOff}
      jobs={jobs}
      tone="hidden"
      emptyMessage={searchQuery ? 'No hidden jobs match this query.' : 'No hidden jobs.'}
      actionLabel="Unhide"
      onAction={onUnhide}
      isPending={isPending}
      selectedJobIds={selectedJobIds}
      selectedJobsCount={selectedJobsCount}
      allVisibleJobsSelected={allVisibleJobsSelected}
      onSelectAllVisible={onSelectAllVisible}
      onToggleSelected={onToggleSelected}
      onClearSelection={onClearSelection}
      onBulkHide={onBulkHide}
      onBulkBadFit={onBulkBadFit}
      onBulkSave={onBulkSave}
      isBulkPending={isBulkPending}
    />
  );
}

export function BadFitJobsSection({
  jobs,
  searchQuery,
  onUnmark,
  isPending,
  selectedJobIds,
  selectedJobsCount,
  allVisibleJobsSelected,
  onSelectAllVisible,
  onToggleSelected,
  onClearSelection,
  onBulkHide,
  onBulkBadFit,
  onBulkSave,
  isBulkPending,
}: {
  jobs: JobPosting[];
  searchQuery: string;
  onUnmark: (jobId: string) => void;
  isPending: boolean;
  selectedJobIds: Set<string>;
  selectedJobsCount: number;
  allVisibleJobsSelected: boolean;
  onSelectAllVisible: (isSelected: boolean) => void;
  onToggleSelected: (jobId: string) => void;
  onClearSelection: () => void;
  onBulkHide: () => void;
  onBulkBadFit: () => void;
  onBulkSave: () => void;
  isBulkPending: boolean;
}) {
  return (
    <JobFeedbackSection
      title="Bad Fit"
      description="Negative examples influence future ranking and reduce similar recommendations."
      icon={ThumbsDown}
      jobs={jobs}
      tone="bad-fit"
      emptyMessage={searchQuery ? 'No bad-fit jobs match this query.' : 'No jobs marked as bad fit.'}
      actionLabel="Unmark"
      onAction={onUnmark}
      isPending={isPending}
      selectedJobIds={selectedJobIds}
      selectedJobsCount={selectedJobsCount}
      allVisibleJobsSelected={allVisibleJobsSelected}
      onSelectAllVisible={onSelectAllVisible}
      onToggleSelected={onToggleSelected}
      onClearSelection={onClearSelection}
      onBulkHide={onBulkHide}
      onBulkBadFit={onBulkBadFit}
      onBulkSave={onBulkSave}
      isBulkPending={isBulkPending}
    />
  );
}

const TIMELINE_ACTION_LABELS: Record<string, string> = {
  job_saved: 'Saved',
  job_unsaved: 'Unsaved',
  job_hidden: 'Hidden',
  job_unhidden: 'Unhidden',
  job_bad_fit: 'Marked bad fit',
  job_bad_fit_removed: 'Removed bad fit',
};

function formatTimelineDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
  }).format(date);
}

export function TimelineSection({
  items,
  totalCount,
  hasMore,
  isLoading,
  isLoadingMore,
  onLoadMore,
}: {
  items: FeedbackTimelineItem[];
  totalCount: number;
  hasMore: boolean;
  isLoading: boolean;
  isLoadingMore: boolean;
  onLoadMore: () => void;
}) {
  return (
    <Section
      title="Timeline"
      icon={<Clock3 size={16} />}
      description="Feedback actions in reverse chronological order."
      count={totalCount}
    >
      {isLoading ? (
        <EmptyState message="Loading timeline..." className="px-4 py-5" />
      ) : items.length === 0 ? (
        <EmptyState
          icon={<Clock3 className="h-10 w-10" />}
          message="No feedback history yet."
          description="Save, hide, or mark jobs as bad fit to build a timeline."
          className="px-4 py-5"
        />
      ) : (
        <div className="space-y-4">
          <div className="flex flex-col gap-3">
            {items.map((item) => {
              const action = TIMELINE_ACTION_LABELS[item.eventType] ?? 'Updated';

              return (
                <div
                  key={item.id}
                  className="rounded-lg border border-border bg-card px-5 py-4"
                >
                  <div className="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
                    <div className="min-w-0">
                      <p className="m-0 text-sm font-semibold text-card-foreground">
                        {action} {item.jobTitle} at {item.companyName}
                      </p>
                      {item.reason ? (
                        <p className="m-0 mt-1 text-sm text-muted-foreground">
                          reason: {item.reason}
                        </p>
                      ) : null}
                    </div>
                    <time
                      dateTime={item.createdAt}
                      className="shrink-0 text-xs font-medium uppercase text-muted-foreground"
                    >
                      {formatTimelineDate(item.createdAt)}
                    </time>
                  </div>
                </div>
              );
            })}
          </div>
          {hasMore ? (
            <div className="flex justify-center">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-10 rounded-xl"
                onClick={onLoadMore}
                disabled={isLoadingMore}
              >
                {isLoadingMore ? 'Loading...' : 'Load more'}
              </Button>
            </div>
          ) : null}
        </div>
      )}
    </Section>
  );
}

function CompanyFeedbackPanel({
  title,
  description,
  count,
  value,
  placeholder,
  accent,
  onChange,
  onSubmit,
  isSubmitting,
  emptyMessage,
  companies,
  badgeLabel,
  rowDescription,
  moveTitle,
  moveToStatus,
  onMove,
  onRemove,
  onBulkHide,
  onUpdateNotes,
  isMovePending,
  isRemovePending,
  isBulkHidePending,
  isUpdateNotesPending,
}: {
  title: string;
  description: string;
  count: number;
  value: string;
  placeholder: string;
  accent: 'success' | 'danger';
  onChange: (value: string) => void;
  onSubmit: () => void;
  isSubmitting: boolean;
  emptyMessage: string;
  companies: CompanyFeedbackRecord[];
  badgeLabel: string;
  rowDescription: string;
  moveTitle: string;
  moveToStatus: 'whitelist' | 'blacklist';
  onMove: (companyName: string, nextStatus: 'whitelist' | 'blacklist') => void;
  onRemove: (companyName: string) => void;
  onBulkHide: (companyName: string) => void;
  onUpdateNotes: (companySlug: string, notes: string) => void;
  isMovePending: boolean;
  isRemovePending: boolean;
  isBulkHidePending: boolean;
  isUpdateNotesPending: boolean;
}) {
  return (
    <CompanyPanel
      title={title}
      description={description}
      count={count}
      value={value}
      placeholder={placeholder}
      accent={accent}
      onChange={onChange}
      onSubmit={onSubmit}
      isSubmitting={isSubmitting}
      emptyMessage={emptyMessage}
    >
      {companies.map((company) => (
        <CompanyRow
          key={company.normalizedCompanyName}
          companyName={company.companyName}
          notes={company.notes}
          accent={accent}
          badgeLabel={badgeLabel}
          description={rowDescription}
          moveTitle={moveTitle}
          onMove={() => onMove(company.companyName, moveToStatus)}
          onRemove={() => onRemove(company.companyName)}
          onBulkHide={() => onBulkHide(company.companyName)}
          onNotesBlur={(notes) => onUpdateNotes(company.normalizedCompanyName, notes)}
          isMovePending={isMovePending}
          isRemovePending={isRemovePending}
          isBulkHidePending={isBulkHidePending}
          isUpdateNotesPending={isUpdateNotesPending}
        />
      ))}
    </CompanyPanel>
  );
}

export function CompaniesSection({
  whitelistedCompanies,
  blacklistedCompanies,
  whitelistInput,
  blacklistInput,
  onWhitelistInputChange,
  onBlacklistInputChange,
  onSubmitCompany,
  onMoveCompany,
  onRemoveWhitelist,
  onRemoveBlacklist,
  onBulkHideCompany,
  onUpdateCompanyNotes,
  isAddWhitelistPending,
  isAddBlacklistPending,
  isMovePending,
  isRemoveWhitelistPending,
  isRemoveBlacklistPending,
  isBulkHidePending,
  isUpdateNotesPending,
}: {
  whitelistedCompanies: CompanyFeedbackRecord[];
  blacklistedCompanies: CompanyFeedbackRecord[];
  whitelistInput: string;
  blacklistInput: string;
  onWhitelistInputChange: (value: string) => void;
  onBlacklistInputChange: (value: string) => void;
  onSubmitCompany: (status: 'whitelist' | 'blacklist') => void;
  onMoveCompany: (companyName: string, nextStatus: 'whitelist' | 'blacklist') => void;
  onRemoveWhitelist: (companyName: string) => void;
  onRemoveBlacklist: (companyName: string) => void;
  onBulkHideCompany: (companyName: string) => void;
  onUpdateCompanyNotes: (companySlug: string, notes: string) => void;
  isAddWhitelistPending: boolean;
  isAddBlacklistPending: boolean;
  isMovePending: boolean;
  isRemoveWhitelistPending: boolean;
  isRemoveBlacklistPending: boolean;
  isBulkHidePending: boolean;
  isUpdateNotesPending: boolean;
}) {
  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <CompanyFeedbackPanel
        title="Whitelisted Companies"
        description="Jobs from these companies should be prioritized in the feed."
        count={whitelistedCompanies.length}
        value={whitelistInput}
        placeholder="Add company to priority list..."
        accent="success"
        onChange={onWhitelistInputChange}
        onSubmit={() => onSubmitCompany('whitelist')}
        isSubmitting={isAddWhitelistPending}
        emptyMessage="No whitelisted companies."
        companies={whitelistedCompanies}
        badgeLabel="Priority"
        rowDescription="Prioritized for future ranking and shortlist views."
        moveTitle="Move to blacklist"
        moveToStatus="blacklist"
        onMove={onMoveCompany}
        onRemove={onRemoveWhitelist}
        onBulkHide={onBulkHideCompany}
        onUpdateNotes={onUpdateCompanyNotes}
        isMovePending={isMovePending}
        isRemovePending={isRemoveWhitelistPending}
        isBulkHidePending={isBulkHidePending}
        isUpdateNotesPending={isUpdateNotesPending}
      />

      <CompanyFeedbackPanel
        title="Blacklisted Companies"
        description="Jobs from these companies should be hidden from the main feed."
        count={blacklistedCompanies.length}
        value={blacklistInput}
        placeholder="Add company to block list..."
        accent="danger"
        onChange={onBlacklistInputChange}
        onSubmit={() => onSubmitCompany('blacklist')}
        isSubmitting={isAddBlacklistPending}
        emptyMessage="No blacklisted companies."
        companies={blacklistedCompanies}
        badgeLabel="Blocked"
        rowDescription="Suppressed from ranking and hidden in future feeds."
        moveTitle="Move to whitelist"
        moveToStatus="whitelist"
        onMove={onMoveCompany}
        onRemove={onRemoveBlacklist}
        onBulkHide={onBulkHideCompany}
        onUpdateNotes={onUpdateCompanyNotes}
        isMovePending={isMovePending}
        isRemovePending={isRemoveBlacklistPending}
        isBulkHidePending={isBulkHidePending}
        isUpdateNotesPending={isUpdateNotesPending}
      />
    </div>
  );
}
