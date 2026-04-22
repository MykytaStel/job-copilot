import {
  BookmarkCheck,
  BriefcaseBusiness,
  CalendarClock,
  CircleX,
  SearchCheck,
  Send,
  Users,
} from 'lucide-react';
import type { ApplicationStatus } from '@job-copilot/shared';

export const COLUMNS: ApplicationStatus[] = ['saved', 'applied', 'interview', 'offer', 'rejected'];

export const NEXT_STATUS: Partial<Record<ApplicationStatus, ApplicationStatus>> = {
  saved: 'applied',
  applied: 'interview',
  interview: 'offer',
};

export const COLUMN_META: Record<
  ApplicationStatus,
  { description: string; icon: typeof SearchCheck }
> = {
  saved: {
    description: 'Jobs kept in the pipeline but not yet submitted.',
    icon: BookmarkCheck,
  },
  applied: {
    description: 'Submitted roles waiting for recruiter or hiring-team response.',
    icon: Send,
  },
  interview: {
    description: 'Active conversations and evaluation loops in progress.',
    icon: Users,
  },
  offer: {
    description: 'Late-stage opportunities with concrete package discussion.',
    icon: BriefcaseBusiness,
  },
  rejected: {
    description: 'Closed opportunities that should still inform future choices.',
    icon: CircleX,
  },
};

export const PIPELINE_STATS = [
  { key: 'activeCount', label: 'Active pipeline', suffix: 'roles', icon: SearchCheck },
  { key: 'offerCount', label: 'Offers', icon: BriefcaseBusiness },
  { key: 'lastUpdate', label: 'Last update', icon: CalendarClock },
] as const;
