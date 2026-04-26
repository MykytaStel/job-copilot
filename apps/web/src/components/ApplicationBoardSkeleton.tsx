import { Skeleton } from './Skeleton';

const COLUMN_WIDTHS = ['34%', '44%', '28%', '38%'];

export function ApplicationBoardSkeleton() {
  return (
    <div className="space-y-5">
      <div className="rounded-2xl border border-border bg-card/70 p-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="flex flex-wrap gap-2">
            <Skeleton height={28} width="88px" />
            <Skeleton height={28} width="104px" />
            <Skeleton height={28} width="96px" />
            <Skeleton height={28} width="112px" />
          </div>

          <Skeleton height={40} width="280px" />
        </div>
      </div>

      <div className="grid gap-4 lg:grid-cols-4">
        {COLUMN_WIDTHS.map((width, columnIndex) => (
          <div
            key={columnIndex}
            className="min-h-[360px] rounded-2xl border border-border bg-card/70 p-4"
          >
            <div className="mb-4 flex items-center justify-between">
              <Skeleton height={18} width={width} />
              <Skeleton height={24} width="32px" />
            </div>

            <div className="space-y-3">
              {Array.from({ length: 3 }, (_, rowIndex) => (
                <div
                  key={rowIndex}
                  className="rounded-xl border border-border bg-surface-muted/50 p-3"
                >
                  <Skeleton height={16} width="72%" />
                  <div className="mt-3 space-y-2">
                    <Skeleton height={12} width="58%" />
                    <Skeleton height={12} width="44%" />
                  </div>
                  <div className="mt-4 flex items-center justify-between">
                    <Skeleton height={22} width="78px" />
                    <Skeleton height={22} width="64px" />
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
