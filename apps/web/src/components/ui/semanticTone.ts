export type SemanticTone = 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'muted';

export const semanticBadgeClass: Record<SemanticTone, string> = {
  primary: 'border-primary/25 bg-primary/12 text-primary',
  info: 'border-fit-good/25 bg-fit-good/12 text-fit-good',
  success: 'border-fit-excellent/25 bg-fit-excellent/12 text-fit-excellent',
  warning: 'border-fit-fair/25 bg-fit-fair/12 text-fit-fair',
  danger: 'border-destructive/25 bg-destructive/12 text-destructive',
  muted: 'border-border bg-white-a04 text-muted-foreground',
};

export const semanticPanelClass: Record<SemanticTone, string> = {
  primary: 'border-primary/20 bg-primary/5',
  info: 'border-fit-good/20 bg-fit-good/5',
  success: 'border-fit-excellent/20 bg-fit-excellent/5',
  warning: 'border-fit-fair/20 bg-fit-fair/5',
  danger: 'border-destructive/20 bg-destructive/5',
  muted: 'border-border/70 bg-surface-muted',
};

export const semanticIconFrameClass: Record<SemanticTone, string> = {
  primary: 'border-primary/15 bg-primary/10 text-primary',
  info: 'border-fit-good/20 bg-fit-good/10 text-fit-good',
  success: 'border-fit-excellent/20 bg-fit-excellent/10 text-fit-excellent',
  warning: 'border-fit-fair/20 bg-fit-fair/10 text-fit-fair',
  danger: 'border-destructive/20 bg-destructive/10 text-destructive',
  muted: 'border-border bg-white-a04 text-muted-foreground',
};

export const semanticTextClass: Record<SemanticTone, string> = {
  primary: 'text-primary',
  info: 'text-fit-good',
  success: 'text-fit-excellent',
  warning: 'text-fit-fair',
  danger: 'text-destructive',
  muted: 'text-muted-foreground',
};

export const semanticDotClass: Record<SemanticTone, string> = {
  primary: 'bg-primary',
  info: 'bg-fit-good',
  success: 'bg-fit-excellent',
  warning: 'bg-fit-fair',
  danger: 'bg-destructive',
  muted: 'bg-muted-foreground',
};

export const semanticFillClass: Record<SemanticTone, string> = {
  primary: 'bg-primary',
  info: 'bg-fit-good',
  success: 'bg-fit-excellent',
  warning: 'bg-fit-fair',
  danger: 'bg-destructive',
  muted: 'bg-muted-foreground',
};
