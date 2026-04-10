interface SkeletonBlockProps {
  height?: number;
  width?: string;
  borderRadius?: number;
}

function SkeletonBlock({ height = 16, width = '100%', borderRadius = 8 }: SkeletonBlockProps) {
  return <div className="skeleton" style={{ height, width, borderRadius }} />;
}

/** Single shimmer line. Use for inline/short replacements. */
export function Skeleton({ height = 20, width = '60%' }: { height?: number; width?: string }) {
  return <SkeletonBlock height={height} width={width} />;
}

/** Full page skeleton: simulates a page header + a few content blocks. */
export function SkeletonPage() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <SkeletonBlock height={32} width="40%" borderRadius={10} />
      <SkeletonBlock height={100} borderRadius={16} />
      <SkeletonBlock height={100} borderRadius={16} />
      <SkeletonBlock height={80} borderRadius={16} />
    </div>
  );
}

/** List skeleton: simulates N card rows. */
export function SkeletonList({ rows = 3 }: { rows?: number }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      {Array.from({ length: rows }, (_, i) => (
        <SkeletonBlock key={i} height={56} borderRadius={14} />
      ))}
    </div>
  );
}
