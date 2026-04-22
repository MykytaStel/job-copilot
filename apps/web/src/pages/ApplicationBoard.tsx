import { Download } from 'lucide-react';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Page, PageGrid } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { useApplicationBoard } from '../features/application-board/useApplicationBoard';
import { ApplicationBoardHero } from './application-board/ApplicationBoardHero';
import { ApplicationBoardKanban } from './application-board/ApplicationBoardKanban';
import { ApplicationBoardSidebar } from './application-board/ApplicationBoardSidebar';

export default function ApplicationBoard() {
  const { applications, jobsById, activeCount, latestUpdatedAt, error, moveMutation, exportCsv } =
    useApplicationBoard();

  return (
    <Page>
      <PageHeader
        title="Applications"
        description="Track saved roles, submitted applications, interview loops, offers, and closed outcomes in one operator board."
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
          message={error instanceof Error ? error.message : 'Не вдалося завантажити pipeline'}
        />
      )}

      {applications.length === 0 ? (
        <EmptyState
          message="Заявок поки немає"
          description="Збережіть першу вакансію на дашборді або на сторінці вакансії."
        />
      ) : (
        <>
          <ApplicationBoardHero
            applications={applications}
            activeCount={activeCount}
            latestUpdatedAt={latestUpdatedAt}
          />

          <PageGrid aside={<ApplicationBoardSidebar applications={applications} />}>
            <ApplicationBoardKanban
              applications={applications}
              jobsById={jobsById}
              isPending={moveMutation.isPending}
              onMove={(id, status) => moveMutation.mutate({ id, status })}
            />
          </PageGrid>
        </>
      )}
    </Page>
  );
}
