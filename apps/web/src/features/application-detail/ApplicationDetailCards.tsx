import { useState } from 'react';
import { FileText } from 'lucide-react';
import type { ApplicationContact } from '@job-copilot/shared';

import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { formatEnumLabel } from '../../lib/format';

export function DescriptionBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const limit = 1200;
  const shouldTruncate = text.length > limit;
  const displayed = expanded || !shouldTruncate ? text : `${text.slice(0, limit)}...`;

  return (
    <div className="space-y-3 rounded-2xl border border-border/70 bg-surface-muted p-4">
      <div className="flex items-center gap-2">
        <FileText className="h-4 w-4 text-primary" />
        <p className="m-0 text-sm font-semibold text-card-foreground">Job description</p>
      </div>
      <div className="whitespace-pre-wrap text-sm leading-7 text-muted-foreground">{displayed}</div>
      {shouldTruncate ? (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="px-0 text-primary hover:text-primary"
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? 'Show less' : 'Show more'}
        </Button>
      ) : null}
    </div>
  );
}

export function ContactCard({ item }: { item: ApplicationContact }) {
  const contact = item.contact;

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-4">
      <div className="flex flex-wrap items-center gap-2">
        <p className="m-0 text-sm font-semibold text-card-foreground">{contact.name}</p>
        <Badge variant="muted" className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]">
          {formatEnumLabel(item.relationship)}
        </Badge>
      </div>
      {contact.role || contact.company ? (
        <p className="m-0 mt-2 text-sm text-muted-foreground">
          {[contact.role, contact.company].filter(Boolean).join(' at ')}
        </p>
      ) : null}
      <div className="mt-3 flex flex-wrap gap-3 text-xs text-muted-foreground">
        {contact.email ? (
          <a href={`mailto:${contact.email}`} className="text-primary no-underline hover:underline">
            {contact.email}
          </a>
        ) : null}
        {contact.phone ? <span>{contact.phone}</span> : null}
        {contact.linkedinUrl ? (
          <a
            href={contact.linkedinUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary no-underline hover:underline"
          >
            LinkedIn
          </a>
        ) : null}
      </div>
    </div>
  );
}
