/** Single shimmer line with dynamic dimensions. */
export function Skeleton({ height = 20, width = '60%' }: { height?: number; width?: string }) {
  return <div className="skeleton" style={{ height, width, borderRadius: 8 }} />;
}

/** Full page skeleton: simulates a page header + a few content blocks. */
export function SkeletonPage() {
  return (
    <div className="flex flex-col gap-5">
      <div className="skeleton h-8 w-2/5 rounded-[10px]" />
      <div className="skeleton h-[100px] rounded-2xl" />
      <div className="skeleton h-[100px] rounded-2xl" />
      <div className="skeleton h-20 rounded-2xl" />
    </div>
  );
}

/** List skeleton: simulates N card rows. */
export function SkeletonList({ rows = 3 }: { rows?: number }) {
  return (
    <div className="flex flex-col gap-3">
      {Array.from({ length: rows }, (_, i) => (
        <div key={i} className="skeleton h-14 rounded-[14px]" />
      ))}
    </div>
  );
}
