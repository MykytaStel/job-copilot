import {
  Bookmark,
  Building2,
  EyeOff,
  ShieldCheck,
  ShieldOff,
  ThumbsDown,
  type LucideIcon,
} from 'lucide-react';

export type FeedbackTab = 'saved' | 'hidden' | 'bad-fit' | 'companies';
export type FeedbackListTone = Exclude<FeedbackTab, 'companies'>;

export const FEEDBACK_TAB_META: Array<{
  id: FeedbackTab;
  label: string;
  description: string;
  icon: LucideIcon;
}> = [
  {
    id: 'saved',
    label: 'Saved',
    description: 'High-intent roles you want to revisit and potentially act on.',
    icon: Bookmark,
  },
  {
    id: 'hidden',
    label: 'Hidden',
    description: 'Suppressed roles that should stay out of the main ranking feed.',
    icon: EyeOff,
  },
  {
    id: 'bad-fit',
    label: 'Bad Fit',
    description: 'Explicit mismatches used as negative ranking evidence.',
    icon: ThumbsDown,
  },
  {
    id: 'companies',
    label: 'Companies',
    description: 'Allow and block lists that steer future ranking toward preferred employers.',
    icon: Building2,
  },
];

export const JOB_ROW_TONE_STYLES: Record<
  FeedbackListTone,
  {
    badge: 'default' | 'warning' | 'danger';
    badgeLabel: string;
    iconClass: string;
    actionClass: string;
  }
> = {
  saved: {
    badge: 'default',
    badgeLabel: 'Positive signal',
    iconClass: 'border-primary/20 bg-primary/10 text-primary',
    actionClass: 'text-primary hover:text-primary',
  },
  hidden: {
    badge: 'warning',
    badgeLabel: 'Suppressed',
    iconClass: 'border-border bg-white-a04 text-muted-foreground',
    actionClass: 'text-muted-foreground hover:text-foreground',
  },
  'bad-fit': {
    badge: 'danger',
    badgeLabel: 'Negative signal',
    iconClass: 'border-destructive/20 bg-destructive/10 text-destructive',
    actionClass: 'text-destructive hover:text-destructive',
  },
};

export const FEEDBACK_SUMMARY_CARDS = [
  { key: 'savedJobsCount', title: 'Saved', icon: Bookmark },
  { key: 'hiddenJobsCount', title: 'Hidden', icon: EyeOff },
  { key: 'badFitJobsCount', title: 'Bad Fit', icon: ThumbsDown },
  { key: 'whitelistedCompaniesCount', title: 'Whitelisted', icon: ShieldCheck },
  { key: 'blacklistedCompaniesCount', title: 'Blacklisted', icon: ShieldOff },
] as const;
