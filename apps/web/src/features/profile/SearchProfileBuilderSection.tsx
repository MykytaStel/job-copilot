import type {
  RoleCatalogItem,
  SearchTargetRegion,
  SearchWorkMode,
  SourceCatalogItem,
} from '../../api/profiles';

import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { OptionCardGroup } from '../../components/ui/OptionCardGroup';
import { SurfaceSection } from '../../components/ui/Surface';
import { TARGET_REGION_OPTIONS, WORK_MODE_OPTIONS } from './profile.constants';
import { renderErrorMessage } from './profileSection.utils';

export function SearchProfileBuilderSection({
  profileExists,
  hasPersistedPreferences,
  targetRegions,
  workModes,
  preferredRoles,
  allowedSources,
  includeKeywordsInput,
  excludeKeywordsInput,
  roles,
  sources,
  rolesError,
  sourcesError,
  isBuilding,
  canBuild,
  onBuild,
  onToggleTargetRegion,
  onToggleWorkMode,
  onTogglePreferredRole,
  onToggleAllowedSource,
  setIncludeKeywordsInput,
  setExcludeKeywordsInput,
}: {
  profileExists: boolean;
  hasPersistedPreferences: boolean;
  targetRegions: SearchTargetRegion[];
  workModes: SearchWorkMode[];
  preferredRoles: string[];
  allowedSources: string[];
  includeKeywordsInput: string;
  excludeKeywordsInput: string;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
  rolesError: unknown;
  sourcesError: unknown;
  isBuilding: boolean;
  canBuild: boolean;
  onBuild: () => void;
  onToggleTargetRegion: (value: SearchTargetRegion) => void;
  onToggleWorkMode: (value: SearchWorkMode) => void;
  onTogglePreferredRole: (value: string) => void;
  onToggleAllowedSource: (value: string) => void;
  setIncludeKeywordsInput: (value: string) => void;
  setExcludeKeywordsInput: (value: string) => void;
}) {
  return (
    <SurfaceSection>
      <div className="mb-5 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Structured search profile
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Roles, regions, work mode, sources
            </span>
          </div>
          <h2 className="m-0 text-xl font-semibold text-card-foreground">
            Build from current raw text
          </h2>
          <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
            Uses the CV text above plus explicit preferences.{' '}
            {profileExists
              ? 'These filters persist on the active profile and can be restored on the next session.'
              : 'These filters stay local until you save the profile.'}
          </p>
          <p className="m-0 text-xs leading-6 text-muted-foreground">
            {hasPersistedPreferences
              ? 'Saved search preferences already exist for this profile.'
              : profileExists
                ? 'No persisted search preferences yet.'
                : 'Create a profile to persist regions, work modes, sources, and keywords.'}
          </p>
        </div>
        <Button type="button" onClick={onBuild} disabled={isBuilding || !canBuild}>
          {isBuilding ? 'Building…' : 'Build search profile'}
        </Button>
      </div>

      <div className="flex flex-col gap-5">
        <div id="profile-field-location-preference" className="fieldGroup">
          <span className="fieldLabel">Target regions</span>
          <OptionCardGroup
            options={TARGET_REGION_OPTIONS}
            value={targetRegions}
            onToggle={onToggleTargetRegion}
          />
        </div>

        <div id="profile-field-work-mode" className="fieldGroup">
          <span className="fieldLabel">Work modes</span>
          <OptionCardGroup
            options={WORK_MODE_OPTIONS}
            value={workModes}
            onToggle={onToggleWorkMode}
          />
        </div>

        <div className="fieldGroup">
          <span className="fieldLabel">Preferred roles</span>
          {roles.length > 0 ? (
            <OptionCardGroup
              options={roles.map((role) => ({ id: role.id, label: role.displayName }))}
              value={preferredRoles}
              onToggle={onTogglePreferredRole}
            />
          ) : (
            <EmptyState message="Role catalog unavailable." />
          )}
          {Boolean(rolesError) && (
            <p className="error">{renderErrorMessage(rolesError, 'Failed to load roles')}</p>
          )}
        </div>

        <div className="fieldGroup">
          <span className="fieldLabel">Allowed sources</span>
          {sources.length > 0 ? (
            <OptionCardGroup
              options={sources.map((source) => ({
                id: source.id,
                label: source.displayName,
              }))}
              value={allowedSources}
              onToggle={onToggleAllowedSource}
            />
          ) : (
            <EmptyState message="Source catalog unavailable." />
          )}
          {Boolean(sourcesError) && (
            <p className="error">{renderErrorMessage(sourcesError, 'Failed to load sources')}</p>
          )}
        </div>

        <div className="fieldGroup">
          <span className="fieldLabel">Include keywords</span>
          <textarea
            rows={3}
            value={includeKeywordsInput}
            onChange={(event) => setIncludeKeywordsInput(event.target.value)}
            placeholder="Comma or newline separated keywords"
          />
        </div>

        <div className="fieldGroup">
          <span className="fieldLabel">Exclude keywords</span>
          <textarea
            rows={3}
            value={excludeKeywordsInput}
            onChange={(event) => setExcludeKeywordsInput(event.target.value)}
            placeholder="Comma or newline separated keywords"
          />
        </div>
      </div>
    </SurfaceSection>
  );
}
