import { useMemo, useState } from 'react';
import { Download } from 'lucide-react';
import type { ApplicationStatus } from '@job-copilot/shared';

import { ApplicationBoardSkeleton } from '../components/ApplicationBoardSkeleton';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { useApplicationBoard } from '../features/application-board/useApplicationBoard';
import { ApplicationsHeader } from './application-board/ApplicationsHeader';
import { ApplicationsTable } from './application-board/ApplicationsTable';
import { ApplicationQuickPanel } from './application-board/ApplicationQuickPanel';

export default function ApplicationBoard() {
  const { applications, jobsById, error, isLoading, moveMutation, exportCsv } =
    useApplicationBoard();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState<ApplicationStatus | 'all'>('all');

  const filtered = useMemo(() => {
    let result = applications;

    if (filterStatus !== 'all') {
      result = result.filter((application) => application.status === filterStatus);
    }

    if (search.trim()) {
      const query = search.toLowerCase();

      result = result.filter((application) => {
        const job = jobsById.get(application.jobId);

        return (
          job?.title?.toLowerCase().includes(query) ||
          job?.company?.toLowerCase().includes(query)
        );
      });
    }

    return result;
  }, [applications, jobsById, filterStatus, search]);

  const selectedApplication = selectedId
    ? applications.find((application) => application.id === selectedId)
    : null;

  function handleSelect(id: string) {
    setSelectedId((prev) => (prev === id ? null : id));
  }

  return (
    <Page>
      <PageHeader
        title="Applications"
        description="Track saved jobs and move them through your application pipeline."
        actions={
          applications.length > 0 ? (
            <Button type="button" variant="outline" onClick={exportCsv}>
              <Download className="h-4 w-4" />
              Export CSV
            </Button>
          ) : undefined
        }
      />

      {isLoading ? (
        <ApplicationBoardSkeleton />
      ) : (
        <>
          {error && (
            <EmptyState
              message="Не вдалося завантажити заявки"
              description={
                error instanceof Error ? error.message : 'Спробуйте оновити сторінку.'
              }
            />
          )}

          {applications.length === 0 ? (
            <EmptyState
              message="No applications yet."
              description="Save your first job from the dashboard to start tracking it here."
            />
          ) : (
            <div className="space-y-5">
              <ApplicationsHeader
                applications={applications}
                search={search}
                filterStatus={filterStatus}
                onSearch={setSearch}
                onFilter={setFilterStatus}
              />

              {filtered.length === 0 ? (
                <EmptyState
                  message="No applications match the current filters."
                  description="Try another status or search query."
                />
              ) : (
                <ApplicationsTable
                  applications={filtered}
                  jobsById={jobsById}
                  selectedId={selectedId}
                  isPending={moveMutation.isPending}
                  onSelect={handleSelect}
                  onMove={(id, status) => moveMutation.mutate({ id, status })}
                />
              )}

              {selectedApplication && (
                <ApplicationQuickPanel
                  application={selectedApplication}
                  job={jobsById.get(selectedApplication.jobId)}
                  onClose={() => setSelectedId(null)}
                />
              )}
            </div>
          )}
        </>
      )}
    </Page>
  );
}
