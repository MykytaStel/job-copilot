import type { ReactNode } from 'react';
import { Ban, Building2, Plus, Star } from 'lucide-react';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { Card, CardContent, CardHeader } from '../../components/ui/Card';
import { EmptyState } from '../../components/ui/EmptyState';
import { semanticIconFrameClass, semanticPanelClass } from '../../components/ui/semanticTone';
import { cn } from '../../lib/cn';

interface CompanyPanelProps {
  title: string;
  description: string;
  count: number;
  value: string;
  placeholder: string;
  accent: 'success' | 'danger';
  onChange: (value: string) => void;
  onSubmit: () => void;
  isSubmitting: boolean;
  emptyMessage: string;
  children: ReactNode;
}

export function CompanyPanel({
  title,
  description,
  count,
  value,
  placeholder,
  accent,
  onChange,
  onSubmit,
  isSubmitting,
  emptyMessage,
  children,
}: CompanyPanelProps) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="gap-3">
        <div className="flex items-center gap-2">
          <h2 className="m-0 text-base font-semibold text-card-foreground">{title}</h2>
          <Badge
            variant={accent}
            className="ml-auto px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
          >
            {count}
          </Badge>
        </div>
        <p className="m-0 text-sm leading-6 text-muted-foreground">{description}</p>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-2xl border border-border/70 bg-surface-muted p-3">
          <div className="flex gap-2">
            <input
              type="text"
              value={value}
              onChange={(event) => onChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') onSubmit();
              }}
              placeholder={placeholder}
              className="h-11 flex-1 rounded-xl border border-border bg-background/70 px-3"
            />
            <Button
              type="button"
              variant="outline"
              size="icon"
              className="h-11 w-11 rounded-xl"
              onClick={onSubmit}
              disabled={isSubmitting || !value.trim()}
            >
              <Plus className="h-4 w-4" />
            </Button>
          </div>
        </div>
        {count === 0 ? (
          <EmptyState message={emptyMessage} className="px-4 py-5 text-left" />
        ) : (
          <div className="flex flex-col gap-3">{children}</div>
        )}
      </CardContent>
    </Card>
  );
}

interface CompanyRowProps {
  companyName: string;
  accent: 'success' | 'danger';
  badgeLabel: string;
  description: string;
  moveTitle: string;
  onMove: () => void;
  onRemove: () => void;
  isMovePending: boolean;
  isRemovePending: boolean;
}

export function CompanyRow({
  companyName,
  accent,
  badgeLabel,
  description,
  moveTitle,
  onMove,
  onRemove,
  isMovePending,
  isRemovePending,
}: CompanyRowProps) {
  const iconClass =
    accent === 'success' ? semanticIconFrameClass.success : semanticIconFrameClass.danger;
  const rowClass =
    accent === 'success' ? semanticPanelClass.success : semanticPanelClass.danger;

  return (
    <Card className={cn('border', rowClass)}>
      <CardContent className="flex items-start justify-between gap-4 px-4 py-4">
        <div className="flex min-w-0 items-start gap-3">
          <div
            className={cn(
              'flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border',
              iconClass,
            )}
          >
            <Building2 className="h-4 w-4" />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <p className="m-0 text-sm font-semibold text-foreground">{companyName}</p>
              <Badge
                variant={accent}
                className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {badgeLabel}
              </Badge>
            </div>
            <p className="mt-1 mb-0 text-xs leading-6 text-muted-foreground">{description}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-muted-foreground"
            onClick={onMove}
            disabled={isMovePending}
            title={moveTitle}
          >
            {accent === 'success' ? <Ban className="h-4 w-4" /> : <Star className="h-4 w-4" />}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onRemove}
            disabled={isRemovePending}
          >
            Remove
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
