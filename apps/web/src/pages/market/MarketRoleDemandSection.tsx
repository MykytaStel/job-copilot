import { RadioTower } from 'lucide-react';

import type { MarketRoleDemand } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';
import { MarketTrendBadge } from './MarketTrendBadge';

function RoleRow({ role }: { role: MarketRoleDemand }) {
  const delta = role.thisPeriod - role.prevPeriod;
  const deltaLabel = delta > 0 ? `+${delta}` : `${delta}`;

  return (
    <div className="flex items-center justify-between gap-4 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4">
      <div className="min-w-0">
        <p className="m-0 text-sm font-semibold text-card-foreground">{role.roleGroup}</p>
        <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
          {formatCount(role.thisPeriod)} this period
          {' • '}
          {formatCount(role.prevPeriod)} previous period
        </p>
      </div>
      <div className="text-right">
        <MarketTrendBadge trend={role.trend} />
        <p className="m-0 mt-2 text-xs text-muted-foreground">{deltaLabel} net change</p>
      </div>
    </div>
  );
}

export function MarketRoleDemandSection({ state }: { state: MarketPageState }) {
  return (
    <MarketSection
      title="Role Demand"
      description="Current-period volume compared with the previous matching window, grouped by major role families."
      icon={RadioTower}
    >
      {state.rolesQuery.isPending ? (
        <ListSkeleton rows={6} />
      ) : state.rolesQuery.isError ? (
        <EmptyState
          message="Unable to load role demand."
          description="The role demand endpoint did not return trend data."
        />
      ) : state.roleDemand.length > 0 ? (
        <div className="space-y-3">
          {state.roleDemand.map((role) => (
            <RoleRow key={role.roleGroup} role={role} />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No role demand signals yet."
          description="Role demand needs enough classified active jobs in the selected period."
        />
      )}
    </MarketSection>
  );
}
