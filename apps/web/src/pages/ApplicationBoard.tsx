import { useMemo, useState } from 'react';
import { Download } from 'lucide-react';
import type { ApplicationStatus } from '@job-copilot/shared';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { useApplicationBoard } from '../features/application-board/useApplicationBoard';
import { ApplicationsHeader } from './application-board/ApplicationsHeader';
import { ApplicationsTable } from './application-board/ApplicationsTable';
import { ApplicationQuickPanel } from './application-board/ApplicationQuickPanel';

export default function ApplicationBoard() {
  const { applications, jobsById, error, moveMutation, exportCsv } = useApplicationBoard();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState<ApplicationStatus | 'all'>('all');

  const filtered = useMemo(() => {
    let result = applications;
    if (filterStatus !== 'all') result = result.filter((a) => a.status === filterStatus);
    if (search.trim()) {
      const q = search.toLowerCase();
      result = result.filter((a) => {
        const job = jobsById.get(a.jobId);
        return (
          job?.title?.toLowerCase().includes(q) || job?.company?.toLowerCase().includes(q)
        );
      });
    }
    return result;
  }, [applications, jobsById, filterStatus, search]);

  const selectedApplication = selectedId ? applications.find((a) => a.id === selectedId) : null;

  function handleSelect(id: string) {
    setSelectedId((prev) => (prev === id ? null : id));
  }

  return (
    <Page>
      <PageHeader
        title="Applications"
        description="Track saved roles, submitted applications, interview loops, offers, and outcomes."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Applications' }]}
        actions={
          applications.length > 0 ? (
            <Button onClick={exportCsv}>
              <Download size={14} />
              Export CSV
            </Button>
          ) : undefined
        }
      />

      {error && (
        <EmptyState
          message={error instanceof Error ? error.message : 'Не вдалося завантажити заявки'}
        />
      )}

      {applications.length === 0 ? (
        <EmptyState
          message="Заявок поки немає"
          description="Збережіть першу вакансію на дашборді або на сторінці вакансії."
        />
      ) : (
        <div className="space-y-4">
          <ApplicationsHeader
            applications={applications}
            search={search}
            filterStatus={filterStatus}
            onSearch={setSearch}
            onFilter={setFilterStatus}
          />

          <div
            className={`grid gap-4 transition-all ${selectedApplication ? 'xl:grid-cols-[1fr_384px]' : ''}`}
          >
            <div className="min-w-0">
              {filtered.length === 0 ? (
                <EmptyState message="No applications match the current filter." />
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
            </div>

            {selectedApplication && (
              <div className="xl:sticky xl:top-4 xl:h-[calc(100vh-8rem)]">
                <ApplicationQuickPanel
                  application={selectedApplication}
                  job={jobsById.get(selectedApplication.jobId)}
                  onClose={() => setSelectedId(null)}
                />
              </div>
            )}
          </div>
        </div>
      )}
    </Page>
  );
}
