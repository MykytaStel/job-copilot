import {
  BarChart3,
  BriefcaseBusiness,
  CalendarClock,
  SearchCheck,
} from 'lucide-react';
import type { Application } from '@job-copilot/shared';
import { Badge } from '../../components/ui/Badge';
import { Card, CardContent } from '../../components/ui/Card';
import { HeroMetric } from '../../components/ui/HeroMetric';
import { formatOptionalDate } from '../../lib/format';

interface ApplicationBoardHeroProps {
  applications: Application[];
  activeCount: number;
  latestUpdatedAt?: string | null | undefined;
}

export function ApplicationBoardHero({
  applications,
  activeCount,
  latestUpdatedAt,
}: ApplicationBoardHeroProps) {
  const offerCount = applications.filter((a) => a.status === 'offer').length;

  return (
    <Card className="overflow-hidden border-border bg-card">
      <CardContent className="p-0">
        <div className="relative">
          <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/10 via-accent/6 to-transparent" />
          <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-end lg:justify-between">
            <div className="max-w-3xl space-y-3">
              <div className="flex flex-wrap gap-2">
                <Badge
                  variant="default"
                  className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                >
                  Pipeline board
                </Badge>
                <Badge variant="muted" className="px-2 py-0.5 text-[10px] uppercase tracking-wide">
                  Saved, applied, interview, offer, rejected
                </Badge>
              </div>
              <h2 className="m-0 text-2xl font-bold text-card-foreground lg:text-3xl">
                Move opportunities through one consistent application workflow
              </h2>
              <p className="m-0 text-sm leading-7 text-muted-foreground lg:text-base">
                This board is the action layer on top of saved jobs. Keep statuses current, watch
                stalled columns, and open any application record for notes, contacts, and offer
                tracking.
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[440px]">
              <HeroMetric label="Active pipeline" value={`${activeCount} roles`} icon={SearchCheck} />
              <HeroMetric label="Offers" value={offerCount} icon={BriefcaseBusiness} />
              <HeroMetric
                label="Last update"
                value={formatOptionalDate(latestUpdatedAt ?? undefined) ?? 'n/a'}
                icon={CalendarClock}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
