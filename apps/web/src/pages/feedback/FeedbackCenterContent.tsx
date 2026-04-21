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
} from './FeedbackCenterSections';
import { FeedbackCenterTabs } from './FeedbackCenterTabs';
import type { FeedbackCenterPageState } from './useFeedbackCenterPage';

export function FeedbackCenterContent({ state }: { state: FeedbackCenterPageState }) {
  const summary = state.summary;

  return (
    <Page>
      <PageHeader
        title="Feedback Center"
        description="Manage saved jobs, hidden roles, bad fits, and company preferences."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Feedback' }]}
      />

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
          isAddWhitelistPending={state.addWhitelistMutation.isPending}
          isAddBlacklistPending={state.addBlacklistMutation.isPending}
          isMovePending={state.moveCompanyMutation.isPending}
          isRemoveWhitelistPending={state.removeWhitelistMutation.isPending}
          isRemoveBlacklistPending={state.removeBlacklistMutation.isPending}
        />
      ) : null}
    </Page>
  );
}
