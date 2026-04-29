import { Page } from '../../components/ui/Page';
import { PageHeader } from '../../components/ui/SectionHeader';
import type { MarketPageState } from '../../features/market/useMarketPage';

import { MarketCompanyVelocitySection } from './MarketCompanyVelocitySection';
import { MarketCompaniesSection } from './MarketCompaniesSection';
import { MarketHero } from './MarketHero';
import { MarketRoleDemandSection } from './MarketRoleDemandSection';
import { MarketSalarySection } from './MarketSalarySection';

export function MarketContent({ state }: { state: MarketPageState }) {
  return (
    <Page>
      <PageHeader
        title="Market Intelligence"
        description="Currently live from the active jobs feed: market overview, hiring companies, salary ranges, and role-demand buckets. Freeze signals, skills demand, and snapshot-backed views are not implemented on this page yet."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Market' }]}
      />

      <MarketHero state={state} />
      <MarketCompanyVelocitySection state={state} />
      <MarketCompaniesSection state={state} />

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
        <MarketSalarySection state={state} />
        <MarketRoleDemandSection state={state} />
      </div>
    </Page>
  );
}
