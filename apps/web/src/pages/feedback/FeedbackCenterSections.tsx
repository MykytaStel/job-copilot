/* eslint-disable react-refresh/only-export-components */

import type { CompanyFeedbackRecord } from '@job-copilot/shared/feedback';
import type { JobPosting } from '@job-copilot/shared';
import type { LucideIcon } from 'lucide-react';
import { Bookmark, EyeOff, ThumbsDown } from 'lucide-react';
import { Link } from 'react-router-dom';

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
          {jobs.map((job) => (
            <JobRow
              key={job.id}
              job={job}
              tone={tone}
              actionLabel={actionLabel}
              onAction={onAction}
              isPending={isPending}
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
}: {
  jobs: JobPosting[];
  searchQuery: string;
  onUnsave: (jobId: string) => void;
  isPending: boolean;
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
    />
  );
}

export function HiddenJobsSection({
  jobs,
  searchQuery,
  onUnhide,
  isPending,
}: {
  jobs: JobPosting[];
  searchQuery: string;
  onUnhide: (jobId: string) => void;
  isPending: boolean;
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
    />
  );
}

export function BadFitJobsSection({
  jobs,
  searchQuery,
  onUnmark,
  isPending,
}: {
  jobs: JobPosting[];
  searchQuery: string;
  onUnmark: (jobId: string) => void;
  isPending: boolean;
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
    />
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
  isMovePending,
  isRemovePending,
  isBulkHidePending,
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
  isMovePending: boolean;
  isRemovePending: boolean;
  isBulkHidePending: boolean;
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
          accent={accent}
          badgeLabel={badgeLabel}
          description={rowDescription}
          moveTitle={moveTitle}
          onMove={() => onMove(company.companyName, moveToStatus)}
          onRemove={() => onRemove(company.companyName)}
          onBulkHide={() => onBulkHide(company.companyName)}
          isMovePending={isMovePending}
          isRemovePending={isRemovePending}
          isBulkHidePending={isBulkHidePending}
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
  isAddWhitelistPending,
  isAddBlacklistPending,
  isMovePending,
  isRemoveWhitelistPending,
  isRemoveBlacklistPending,
  isBulkHidePending,
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
  isAddWhitelistPending: boolean;
  isAddBlacklistPending: boolean;
  isMovePending: boolean;
  isRemoveWhitelistPending: boolean;
  isRemoveBlacklistPending: boolean;
  isBulkHidePending: boolean;
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
        isMovePending={isMovePending}
        isRemovePending={isRemoveWhitelistPending}
        isBulkHidePending={isBulkHidePending}
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
        isMovePending={isMovePending}
        isRemovePending={isRemoveBlacklistPending}
        isBulkHidePending={isBulkHidePending}
      />
    </div>
  );
}
