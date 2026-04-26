import { Skeleton } from './Skeleton';
import { Page, PageGrid } from './ui/Page';

export function JobDetailsSkeleton() {
  return (
    <Page>
      <div className="space-y-6">
        <div className="space-y-3">
          <Skeleton height={16} width="18%" />
          <Skeleton height={38} width="48%" />
          <Skeleton height={18} width="32%" />
        </div>

        <PageGrid
          aside={
            <aside className="space-y-4">
              <div className="rounded-2xl border border-border bg-card/70 p-5">
                <Skeleton height={20} width="52%" />
                <div className="mt-5 space-y-3">
                  <Skeleton height={14} width="72%" />
                  <Skeleton height={14} width="64%" />
                  <Skeleton height={14} width="58%" />
                </div>
              </div>

              <div className="rounded-2xl border border-border bg-card/70 p-5">
                <Skeleton height={20} width="46%" />
                <div className="mt-5 space-y-3">
                  <Skeleton height={34} width="100%" />
                  <Skeleton height={34} width="100%" />
                </div>
              </div>
            </aside>
          }
        >
          <div className="space-y-4">
            <div className="rounded-2xl border border-border bg-card/70 p-5">
              <Skeleton height={22} width="36%" />
              <div className="mt-5 grid gap-3 sm:grid-cols-3">
                <Skeleton height={72} width="100%" />
                <Skeleton height={72} width="100%" />
                <Skeleton height={72} width="100%" />
              </div>
            </div>

            <div className="rounded-2xl border border-border bg-card/70 p-5">
              <Skeleton height={22} width="28%" />
              <div className="mt-5 space-y-3">
                <Skeleton height={16} width="96%" />
                <Skeleton height={16} width="90%" />
                <Skeleton height={16} width="76%" />
                <Skeleton height={16} width="68%" />
              </div>
            </div>

            <div className="rounded-2xl border border-border bg-card/70 p-5">
              <Skeleton height={22} width="34%" />
              <div className="mt-5 flex flex-wrap gap-2">
                <Skeleton height={28} width="96px" />
                <Skeleton height={28} width="120px" />
                <Skeleton height={28} width="88px" />
                <Skeleton height={28} width="108px" />
              </div>
            </div>
          </div>
        </PageGrid>
      </div>
    </Page>
  );
}
