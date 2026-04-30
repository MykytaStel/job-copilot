import { Page } from '../../components/ui/Page';
import { PageHeader } from '../../components/ui/SectionHeader';
import type { MarketPageState } from '../../features/market/useMarketPage';

import { MarketCompanyVelocitySection } from './MarketCompanyVelocitySection';
import { MarketCompaniesSection } from './MarketCompaniesSection';
import { MarketFreezeSignalsSection } from './MarketFreezeSignalsSection';
import { MarketHero } from './MarketHero';
import { MarketRegionBreakdownSection } from './MarketRegionBreakdownSection';
import { MarketRoleDemandSection } from './MarketRoleDemandSection';
import { MarketSalarySection } from './MarketSalarySection';
import { MarketTechDemandSection } from './MarketTechDemandSection';

export function MarketContent({ state }: { state: MarketPageState }) {
  return (
    <Page>
      <PageHeader
        title="Market Intelligence"
        description="Currently live from the active jobs feed: market overview, hiring companies, freeze signals, salary ranges, role-demand buckets, and technology demand."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Market' }]}
      />

      <MarketHero state={state} />
      <MarketCompanyVelocitySection state={state} />
      <MarketFreezeSignalsSection state={state} />
      <MarketCompaniesSection state={state} />

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
        <MarketSalarySection state={state} />
        <MarketRoleDemandSection state={state} />
      </div>
      <MarketRegionBreakdownSection state={state} />
      <MarketTechDemandSection state={state} />
    </Page>
  );
}
