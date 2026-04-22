import { Search } from 'lucide-react';

import { Card, CardContent } from '../../components/ui/Card';
import { cn } from '../../lib/cn';
import { FEEDBACK_TAB_META } from './FeedbackCenterComponents';
import type { FeedbackCenterPageState } from './useFeedbackCenterPage';

export function FeedbackCenterTabs({
  activeTab,
  setActiveTab,
  tabCounts,
  activeTabMeta,
  searchQuery,
  setSearchQuery,
}: Pick<
  FeedbackCenterPageState,
  'activeTab' | 'setActiveTab' | 'tabCounts' | 'activeTabMeta' | 'searchQuery' | 'setSearchQuery'
>) {
  return (
    <Card className="border-border bg-card">
      <CardContent className="space-y-6 px-6 py-6">
        <div className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_320px] lg:items-start">
          <div className="space-y-4">
            <div className="flex w-full flex-wrap gap-2">
              {FEEDBACK_TAB_META.map((tab) => (
                <button
                  key={tab.id}
                  type="button"
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    'inline-flex items-center gap-2 rounded-full border px-3 py-2 text-sm transition-colors',
                    activeTab === tab.id
                      ? 'border-primary bg-primary text-primary-foreground'
                      : 'border-border bg-surface-elevated/50 text-muted-foreground hover:bg-surface-hover hover:text-foreground',
                  )}
                >
                  <tab.icon className="h-4 w-4" />
                  <span>{tab.label}</span>
                  <span
                    className={cn(
                      'rounded-full px-1.5 py-0.5 text-[11px] leading-none',
                      activeTab === tab.id
                        ? 'bg-white/20 text-white'
                        : 'bg-black/20 text-muted-foreground',
                    )}
                  >
                    {tabCounts[tab.id]}
                  </span>
                </button>
              ))}
            </div>
            <p className="m-0 max-w-3xl text-sm leading-6 text-muted-foreground">
              {activeTabMeta.description}
            </p>
          </div>

          <div className="space-y-3 rounded-2xl border border-border/70 bg-white/[0.03] p-4">
            <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Active slice
            </p>
            <div className="flex items-center gap-3">
              <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
                <activeTabMeta.icon className="h-4 w-4" />
              </div>
              <div>
                <p className="m-0 text-sm font-semibold text-card-foreground">
                  {activeTabMeta.label}
                </p>
                <p className="m-0 mt-1 text-xs text-muted-foreground">
                  {activeTab === 'companies'
                    ? `${tabCounts.companies} companies tracked`
                    : `${searchQuery ? 'Filtered' : 'All'} jobs in this list`}
                </p>
              </div>
            </div>
            {activeTab !== 'companies' ? (
              <div className="relative">
                <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <input
                  type="search"
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                  placeholder="Search jobs..."
                  className="h-11 w-full rounded-xl border border-border bg-background/70 pl-9"
                />
              </div>
            ) : null}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
