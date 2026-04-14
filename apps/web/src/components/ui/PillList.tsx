export function PillList({
  items,
  emptyLabel,
  tone = 'default',
}: {
  items: string[];
  emptyLabel: string;
  tone?: 'default' | 'success' | 'danger';
}) {
  if (items.length === 0) {
    return <p className="muted sectionText">{emptyLabel}</p>;
  }

  return (
    <div className="pillWrap">
      {items.map((item) => (
        <span key={item} className={`pill ${tone !== 'default' ? `pill-${tone}` : ''}`}>
          {item}
        </span>
      ))}
    </div>
  );
}
