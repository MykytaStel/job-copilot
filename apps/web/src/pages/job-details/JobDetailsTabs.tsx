import { cn } from '../../lib/cn';

import { JOB_DETAILS_TABS } from './jobDetails.constants';

export function JobDetailsTabs({
  activeTab,
  setActiveTab,
}: {
  activeTab: (typeof JOB_DETAILS_TABS)[number]['id'];
  setActiveTab: (value: (typeof JOB_DETAILS_TABS)[number]['id']) => void;
}) {
  return (
    <div className="flex flex-wrap gap-2">
      {JOB_DETAILS_TABS.map((tab) => (
        <button
          key={tab.id}
          type="button"
          onClick={() => setActiveTab(tab.id)}
          className={cn(
            'inline-flex items-center rounded-full border px-3 py-2 text-sm transition-colors',
            activeTab === tab.id
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-surface-elevated/50 text-muted-foreground hover:bg-surface-hover hover:text-foreground',
          )}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
