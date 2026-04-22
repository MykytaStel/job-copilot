import type { ReactNode } from 'react';

export const insightPanelBaseClass =
  'flex flex-col gap-3 rounded-[var(--radius-lg)] border px-4 py-3.5';

export function FitExplanationList({
  label,
  items,
  emptyLabel,
}: {
  label: string;
  items: string[];
  emptyLabel: string;
}) {
  return (
    <div>
      <span className="detailLabel">{label}</span>
      {items.length > 0 ? (
        <ul className="textList fitReasonsList" style={{ marginTop: 8 }}>
          {items.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      ) : (
        <p className="m-0 text-sm leading-6 text-muted-foreground">{emptyLabel}</p>
      )}
    </div>
  );
}

export function PanelTextBlock({
  label,
  text,
  emptyLabel,
}: {
  label: string;
  text: string;
  emptyLabel: string;
}) {
  return (
    <div>
      <span className="detailLabel">{label}</span>
      <p className="sectionText" style={{ marginBottom: 0 }}>
        {text || emptyLabel}
      </p>
    </div>
  );
}

export function ParagraphStack({
  items,
  emptyLabel,
}: {
  items: string[];
  emptyLabel: string;
}) {
  if (items.length === 0) {
    return <p className="m-0 text-sm leading-6 text-muted-foreground">{emptyLabel}</p>;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10, marginTop: 8 }}>
      {items.map((paragraph, index) => (
        <p key={`${index}-${paragraph}`} className="sectionText" style={{ marginBottom: 0 }}>
          {paragraph}
        </p>
      ))}
    </div>
  );
}

export function InsightPanel({
  toneClassName,
  children,
}: {
  toneClassName: string;
  children: ReactNode;
}) {
  return <div className={`${insightPanelBaseClass} border-edge-accent ${toneClassName}`}>{children}</div>;
}
