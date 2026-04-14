export function StatusBadge({ status }: { status: string }) {
  return <span className={`statusPill status-${status}`}>{status}</span>;
}
