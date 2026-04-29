import { Bookmark, EyeOff, ShieldCheck, ShieldOff, ThumbsDown } from 'lucide-react';

import { Page } from '../../components/ui/Page';
import { PageHeader } from '../../components/ui/SectionHeader';
import { StatCard } from '../../components/ui/StatCard';

import { FEEDBACK_SUMMARY_CARDS } from './FeedbackCenterComponents';
import { FeedbackCenterHero } from './FeedbackCenterHero';
import {
  BadFitJobsSection,
  CompaniesSection,
  HiddenJobsSection,
  SavedJobsSection,
  TimelineSection,
} from './FeedbackCenterSections';
import { FeedbackCenterTabs } from './FeedbackCenterTabs';
import type { FeedbackCenterPageState } from './useFeedbackCenterPage';

export function FeedbackCenterContent({ state }: { state: FeedbackCenterPageState }) {
  const summary = state.summary;
  const stats = state.feedbackStats;
  const statValue = (value: number | undefined) => value ?? '--';

  return (
    <Page>
      <PageHeader
        title="Feedback Center"
        description="Manage saved jobs, hidden roles, bad fits, and company preferences."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Feedback' }]}
      />

      <div className="grid grid-cols-2 gap-4 lg:grid-cols-5">
        <StatCard
          title="Saved this week"
          value={statValue(stats?.savedThisWeekCount)}
          icon={Bookmark}
        />
        <StatCard
          title="Hidden this week"
          value={statValue(stats?.hiddenThisWeekCount)}
          icon={EyeOff}
        />
        <StatCard
          title="Bad fit this week"
          value={statValue(stats?.badFitThisWeekCount)}
          icon={ThumbsDown}
        />
        <StatCard
          title="Whitelisted companies"
          value={statValue(stats?.whitelistedCompaniesCount)}
          icon={ShieldCheck}
        />
        <StatCard
          title="Blacklisted companies"
          value={statValue(stats?.blacklistedCompaniesCount)}
          icon={ShieldOff}
        />
      </div>

      <FeedbackCenterHero
        savedJobs={state.savedJobs}
        badFitJobs={state.badFitJobs}
        tabCounts={state.tabCounts}
      />

      {summary ? (
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-5">
          {FEEDBACK_SUMMARY_CARDS.map((item) => (
            <StatCard
              key={item.key}
              title={item.title}
              value={summary[item.key]}
              icon={item.icon}
            />
          ))}
        </div>
      ) : null}

      <FeedbackCenterTabs
        activeTab={state.activeTab}
        setActiveTab={state.setActiveTab}
        tabCounts={state.tabCounts}
        activeTabMeta={state.activeTabMeta}
        searchQuery={state.searchQuery}
        setSearchQuery={state.setSearchQuery}
        onExport={() => state.exportMutation.mutate()}
        isExporting={state.exportMutation.isPending}
      />

      {state.activeTab === 'saved' ? (
        <SavedJobsSection
          jobs={state.filteredSavedJobs}
          searchQuery={state.searchQuery}
          onUnsave={(jobId) => state.unsaveMutation.mutate(jobId)}
          isPending={state.unsaveMutation.isPending}
        />
      ) : null}

      {state.activeTab === 'hidden' ? (
        <HiddenJobsSection
          jobs={state.filteredHiddenJobs}
          searchQuery={state.searchQuery}
          onUnhide={(jobId) => state.unhideMutation.mutate(jobId)}
          isPending={state.unhideMutation.isPending}
        />
      ) : null}

      {state.activeTab === 'bad-fit' ? (
        <BadFitJobsSection
          jobs={state.filteredBadFitJobs}
          searchQuery={state.searchQuery}
          onUnmark={(jobId) => state.unmarkBadFitMutation.mutate(jobId)}
          isPending={state.unmarkBadFitMutation.isPending}
        />
      ) : null}

      {state.activeTab === 'companies' ? (
        <CompaniesSection
          whitelistedCompanies={state.whitelistedCompanies}
          blacklistedCompanies={state.blacklistedCompanies}
          whitelistInput={state.whitelistInput}
          blacklistInput={state.blacklistInput}
          onWhitelistInputChange={state.setWhitelistInput}
          onBlacklistInputChange={state.setBlacklistInput}
          onSubmitCompany={state.submitCompany}
          onMoveCompany={(companyName, nextStatus) =>
            state.moveCompanyMutation.mutate({ companyName, nextStatus })
          }
          onRemoveWhitelist={(companyName) => state.removeWhitelistMutation.mutate(companyName)}
          onRemoveBlacklist={(companyName) => state.removeBlacklistMutation.mutate(companyName)}
          onBulkHideCompany={(companyName) => state.bulkHideCompanyMutation.mutate(companyName)}
          onUpdateCompanyNotes={(companySlug, notes) =>
            state.updateCompanyNotesMutation.mutate({ companySlug, notes })
          }
          isAddWhitelistPending={state.addWhitelistMutation.isPending}
          isAddBlacklistPending={state.addBlacklistMutation.isPending}
          isMovePending={state.moveCompanyMutation.isPending}
          isRemoveWhitelistPending={state.removeWhitelistMutation.isPending}
          isRemoveBlacklistPending={state.removeBlacklistMutation.isPending}
          isBulkHidePending={state.bulkHideCompanyMutation.isPending}
          isUpdateNotesPending={state.updateCompanyNotesMutation.isPending}
        />
      ) : null}

      {state.activeTab === 'timeline' ? (
        <TimelineSection
          items={state.timelineItems}
          totalCount={state.timelineTotalCount}
          hasMore={state.hasMoreTimeline}
          isLoading={state.isTimelineLoading}
          isLoadingMore={state.isTimelineLoadingMore}
          onLoadMore={state.loadMoreTimeline}
        />
      ) : null}
    </Page>
  );
}
