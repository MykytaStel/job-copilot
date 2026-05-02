import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Bell, Settings as SettingsIcon, ShieldCheck, SlidersHorizontal, Target, UserRound } from 'lucide-react';

import { getProfile } from '../api/profiles';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { cn } from '../lib/cn';
import { queryKeys } from '../queryKeys';
import { DisplaySettingsSection } from '../features/settings/DisplaySettingsSection';
import { NotificationsSettingsSection } from '../features/settings/NotificationsSettingsSection';
import { PrivacySettingsSection } from '../features/settings/PrivacySettingsSection';
import { ProfileSettingsSection } from '../features/settings/ProfileSettingsSection';
import { SearchSettingsSection } from '../features/settings/SearchSettingsSection';

type SectionId = 'profile' | 'search' | 'notifications' | 'display' | 'privacy';

const SECTIONS: {
  id: SectionId;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}[] = [
  { id: 'profile', label: 'Profile & Account', icon: UserRound },
  { id: 'search', label: 'Search Preferences', icon: Target },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'display', label: 'Display', icon: SlidersHorizontal },
  { id: 'privacy', label: 'Data & Privacy', icon: ShieldCheck },
];

export default function Settings() {
  const [activeSection, setActiveSection] = useState<SectionId>('profile');

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });

  if (!profile) {
    return (
      <Page>
        <PageHeader
          title="Settings"
          description="Profile-scoped defaults and notification routing for the current operator context."
          breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Settings' }]}
        />
        <EmptyState
          icon={<SettingsIcon className="h-5 w-5" />}
          message="Settings need an active profile"
          description="Create or load a profile first so Job Copilot can scope preferences to the right candidate context."
        />
      </Page>
    );
  }

  return (
    <Page>
      <PageHeader
        title="Settings"
        description="Profile-scoped preferences and operator defaults."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Settings' }]}
      />

      <div className="flex flex-col gap-6 sm:flex-row sm:gap-0">
        <nav className="flex shrink-0 flex-row gap-1 overflow-x-auto pb-1 sm:w-52 sm:flex-col sm:overflow-x-visible sm:border-r sm:border-border sm:pb-0 sm:pr-4">
          {SECTIONS.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveSection(id)}
              className={cn(
                'flex items-center gap-2.5 whitespace-nowrap rounded-[var(--radius-md)] px-3 py-2 text-sm font-medium transition-colors',
                activeSection === id
                  ? 'bg-surface-muted text-foreground'
                  : 'text-muted-foreground hover:bg-surface-muted/60 hover:text-foreground',
              )}
            >
              <Icon className="h-4 w-4 shrink-0" />
              {label}
            </button>
          ))}
        </nav>

        <div className="min-w-0 flex-1 sm:pl-8">
          {activeSection === 'profile' && <ProfileSettingsSection />}
          {activeSection === 'search' && <SearchSettingsSection />}
          {activeSection === 'notifications' && <NotificationsSettingsSection />}
          {activeSection === 'display' && <DisplaySettingsSection />}
          {activeSection === 'privacy' && <PrivacySettingsSection />}
        </div>
      </div>
    </Page>
  );
}
