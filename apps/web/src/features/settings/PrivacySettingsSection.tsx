import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { AlertTriangle, Ban, Building2, Download, Trash2 } from 'lucide-react';

import { clearAllHiddenJobs, getFeedback, removeCompanyBlacklistBySlug } from '../../api/feedback';
import { exportUserData } from '../../api/export';
import { resetProfileData } from '../../api/dataManagement';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { invalidateFeedbackQueries } from '../../lib/queryInvalidation';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';
import { SettingsSection } from './settingsShared';

export function PrivacySettingsSection() {
  const queryClient = useQueryClient();
  const profileId = readProfileId();
  const [showResetConfirmation, setShowResetConfirmation] = useState(false);
  const [resetConfirmationInput, setResetConfirmationInput] = useState('');

  const { data: feedbackOverview } = useQuery({
    queryKey: queryKeys.feedback.profile(profileId ?? 'none'),
    queryFn: () => getFeedback(profileId!),
    enabled: !!profileId,
  });

  const hiddenJobsCount = feedbackOverview?.summary.hiddenJobsCount ?? 0;
  const blacklistedCompanies = (feedbackOverview?.companies ?? []).filter(
    (company) => company.status === 'blacklist',
  );

  const clearHiddenJobsMutation = useMutation({
    mutationFn: () => {
      if (!profileId) throw new Error('Create a profile first');
      return clearAllHiddenJobs(profileId);
    },
    onSuccess: () => {
      if (!profileId) return;
      void invalidateFeedbackQueries(queryClient, profileId);
    },
  });

  const removeBlockedCompanyMutation = useMutation({
    mutationFn: ({
      normalizedCompanyName,
    }: {
      companyName: string;
      normalizedCompanyName: string;
    }) => {
      if (!profileId) throw new Error('Create a profile first');
      return removeCompanyBlacklistBySlug(profileId, normalizedCompanyName);
    },
    onSuccess: () => {
      if (!profileId) return;
      void invalidateFeedbackQueries(queryClient, profileId);
    },
  });

  const exportDataMutation = useMutation({
    mutationFn: exportUserData,
    onSuccess: (data) => {
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `job-copilot-export-${formatDateForFilename(new Date())}.json`;
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
    },
  });

  const resetDataMutation = useMutation({
    mutationFn: () => {
      if (!profileId) throw new Error('Create a profile first');
      return resetProfileData(profileId);
    },
    onSuccess: () => {
      setResetConfirmationInput('');
      setShowResetConfirmation(false);
      void queryClient.invalidateQueries({ queryKey: queryKeys.profile.root() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
      void invalidateFeedbackQueries(queryClient, profileId);
    },
  });

  function confirmClearHiddenJobs() {
    const confirmed = window.confirm(
      `This will delete ${hiddenJobsCount} hidden feedback entries. Continue?`,
    );
    if (confirmed) clearHiddenJobsMutation.mutate();
  }

  function confirmRemoveBlockedCompany(company: {
    companyName: string;
    normalizedCompanyName: string;
  }) {
    const confirmed = window.confirm(
      `Are you sure? Remove ${company.companyName} from blocked companies?`,
    );
    if (confirmed) removeBlockedCompanyMutation.mutate(company);
  }

  function beginResetDataConfirmation() {
    const confirmed = window.confirm(
      'This will delete all feedback and applications for this profile and reset search preferences. Continue?',
    );
    if (confirmed) {
      setShowResetConfirmation(true);
      setResetConfirmationInput('');
    }
  }

  return (
    <SettingsSection title="Data & Privacy">
      <div className="space-y-4">
        <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p className="text-sm font-semibold text-foreground">Hidden jobs</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Clear hidden feedback entries for this profile so those jobs can appear again.
              </p>
              <p className="mt-3 text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
                {hiddenJobsCount} hidden feedback entries
              </p>
            </div>

            <Button
              type="button"
              variant="outline"
              onClick={confirmClearHiddenJobs}
              disabled={hiddenJobsCount === 0 || clearHiddenJobsMutation.isPending || !profileId}
              className="shrink-0"
            >
              <Trash2 className="h-4 w-4" />
              {clearHiddenJobsMutation.isPending ? 'Clearing hidden jobs' : 'Clear all hidden jobs'}
            </Button>
          </div>

          {clearHiddenJobsMutation.isError && (
            <p className="mt-3 text-sm text-danger">
              Could not clear hidden jobs. Please try again.
            </p>
          )}
        </div>

        <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p className="text-sm font-semibold text-foreground">Export my data</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Download profile, feedback, company lists, and applications as JSON.
              </p>
            </div>

            <Button
              type="button"
              variant="outline"
              onClick={() => exportDataMutation.mutate()}
              disabled={exportDataMutation.isPending}
              className="shrink-0"
            >
              <Download className="h-4 w-4" />
              {exportDataMutation.isPending ? 'Exporting data' : 'Export my data'}
            </Button>
          </div>

          {exportDataMutation.isError && (
            <p className="mt-3 text-sm text-danger">
              Could not export your data. Please sign in and try again.
            </p>
          )}
        </div>

        <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
          <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p className="text-sm font-semibold text-foreground">Blocked companies</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Manage companies currently blacklisted for this profile.
              </p>
            </div>

            <Badge
              variant="danger"
              className="w-fit px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
            >
              {blacklistedCompanies.length}
            </Badge>
          </div>

          {blacklistedCompanies.length === 0 ? (
            <EmptyState
              icon={<Ban className="h-5 w-5" />}
              message="No blocked companies"
              description="Companies blocked from Feedback Center will appear here."
              className="bg-surface"
            />
          ) : (
            <div className="space-y-3">
              {blacklistedCompanies.map((company) => {
                const isRemoving =
                  removeBlockedCompanyMutation.isPending &&
                  removeBlockedCompanyMutation.variables?.normalizedCompanyName ===
                    company.normalizedCompanyName;

                return (
                  <div
                    key={company.normalizedCompanyName}
                    className="flex flex-col gap-3 rounded-[var(--radius-md)] border border-border bg-surface p-3 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <div className="flex min-w-0 items-center gap-3">
                      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-md)] border border-danger/25 bg-danger/10 text-danger">
                        <Building2 className="h-4 w-4" />
                      </div>
                      <div className="min-w-0">
                        <p className="truncate text-sm font-medium text-foreground">
                          {company.companyName}
                        </p>
                        <p className="mt-1 truncate text-xs text-muted-foreground">
                          {company.normalizedCompanyName}
                        </p>
                      </div>
                    </div>

                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => confirmRemoveBlockedCompany(company)}
                      disabled={isRemoving || !profileId}
                      className="shrink-0"
                    >
                      {isRemoving ? 'Removing' : 'Remove'}
                    </Button>
                  </div>
                );
              })}
            </div>
          )}

          {removeBlockedCompanyMutation.isError && (
            <p className="mt-3 text-sm text-danger">
              Could not remove blocked company. Please try again.
            </p>
          )}
        </div>

        <div className="rounded-[var(--radius-lg)] border border-danger/35 bg-danger/5 p-4">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
            <div className="flex gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-md)] border border-danger/25 bg-danger/10 text-danger">
                <AlertTriangle className="h-4 w-4" />
              </div>
              <div>
                <p className="text-sm font-semibold text-foreground">Danger Zone</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  Clear all feedback, applications, and saved search preferences for this profile.
                  Profile details and CVs are preserved.
                </p>
              </div>
            </div>

            <Button
              type="button"
              variant="outline"
              onClick={beginResetDataConfirmation}
              disabled={!profileId || resetDataMutation.isPending}
              className="shrink-0 border-danger/40 text-danger hover:bg-danger/10 hover:text-danger"
            >
              <Trash2 className="h-4 w-4" />
              Clear all data
            </Button>
          </div>

          {showResetConfirmation && (
            <div className="mt-4 rounded-[var(--radius-md)] border border-danger/30 bg-surface p-3">
              <label className="block text-xs font-medium uppercase tracking-[0.14em] text-muted-foreground">
                Type RESET to confirm
              </label>
              <div className="mt-3 flex flex-col gap-3 sm:flex-row">
                <input
                  value={resetConfirmationInput}
                  onChange={(event) => setResetConfirmationInput(event.currentTarget.value)}
                  disabled={resetDataMutation.isPending}
                  className="min-h-10 flex-1 rounded-[var(--radius-md)] border border-border bg-background px-3 text-sm text-foreground outline-none focus:border-danger/60 focus:ring-2 focus:ring-danger/20"
                  autoComplete="off"
                />
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => resetDataMutation.mutate()}
                  disabled={
                    resetConfirmationInput !== 'RESET' ||
                    resetDataMutation.isPending ||
                    !profileId
                  }
                  className="shrink-0 border-danger/40 text-danger hover:bg-danger/10 hover:text-danger"
                >
                  {resetDataMutation.isPending ? 'Clearing data' : 'Confirm reset'}
                </Button>
              </div>
            </div>
          )}

          {resetDataMutation.isError && (
            <p className="mt-3 text-sm text-danger">
              Could not clear profile data. Please try again.
            </p>
          )}
        </div>
      </div>
    </SettingsSection>
  );
}

function formatDateForFilename(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}
