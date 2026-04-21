import type { ChangeEventHandler, RefObject } from 'react';
import { Upload } from 'lucide-react';

import type { LlmContext } from '../../api/analytics';
import type { RankedJobResult, SearchRunResult } from '../../api/jobs';
import type {
  RoleCatalogItem,
  SearchProfileBuildResult,
  SearchTargetRegion,
  SearchWorkMode,
  SourceCatalogItem,
} from '../../api/profiles';

import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { OptionCardGroup } from '../../components/ui/OptionCardGroup';
import { PillList } from '../../components/ui/PillList';
import { formatFallbackLabel } from '../../lib/format';
import {
  PROFILE_LANGUAGE_OPTIONS,
  PROFILE_SALARY_CURRENCY_OPTIONS,
  TARGET_REGION_OPTIONS,
  WORK_MODE_OPTIONS,
} from './profile.constants';
import { resolveRoleLabel, resolveSourceLabel } from './profile.utils';
import { SearchProfilePillSection, SearchResultsSection } from './SearchResultCard';

function formatSeniorityLabel(value: string) {
  return value.trim() ? formatFallbackLabel(value) : 'Not specified';
}

function renderErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

export function ProfileFormSection({
  name,
  email,
  location,
  rawText,
  yearsOfExperience,
  salaryMin,
  salaryMax,
  salaryCurrency,
  languages,
  profileExists,
  fileInputRef,
  isSaving,
  isAnalyzing,
  onSave,
  onAnalyze,
  onOpenFilePicker,
  onFileChange,
  setName,
  setEmail,
  setLocation,
  setRawText,
  setYearsOfExperience,
  setSalaryMin,
  setSalaryMax,
  setSalaryCurrency,
  onToggleLanguage,
}: {
  name: string;
  email: string;
  location: string;
  rawText: string;
  yearsOfExperience: string;
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: string[];
  profileExists: boolean;
  fileInputRef: RefObject<HTMLInputElement | null>;
  isSaving: boolean;
  isAnalyzing: boolean;
  onSave: () => void;
  onAnalyze: () => void;
  onOpenFilePicker: () => void;
  onFileChange: ChangeEventHandler<HTMLInputElement>;
  setName: (value: string) => void;
  setEmail: (value: string) => void;
  setLocation: (value: string) => void;
  setRawText: (value: string) => void;
  setYearsOfExperience: (value: string) => void;
  setSalaryMin: (value: string) => void;
  setSalaryMax: (value: string) => void;
  setSalaryCurrency: (value: string) => void;
  onToggleLanguage: (value: string) => void;
}) {
  return (
    <>
      <div className="flex flex-col gap-5 rounded-[28px] border border-border bg-card/85 p-7 shadow-[var(--shadow-hero)] md:flex-row md:items-end md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Persisted profile
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              {profileExists ? 'Ready for analysis' : 'Create profile first'}
            </span>
          </div>
          <h2 className="m-0 text-2xl font-bold text-card-foreground">Candidate intake</h2>
          <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
            Persisted in `engine-api` and used for analysis/search-profile flows.
          </p>
        </div>
        <Button
          type="button"
          onClick={onAnalyze}
          disabled={!profileExists || isAnalyzing}
          className="w-full md:w-auto"
        >
          {isAnalyzing ? 'Analyzing…' : 'Analyze'}
        </Button>
      </div>

      <form
        className="flex flex-col gap-5 rounded-[24px] border border-border bg-card/85 p-7"
        onSubmit={(event) => {
          event.preventDefault();
          onSave();
        }}
      >
        <label>
          Name
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Your name"
            required
          />
        </label>
        <label>
          Email
          <input
            type="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            placeholder="you@email.com"
            required
          />
        </label>
        <label>
          Location <span className="text-muted-foreground">(optional)</span>
          <input
            value={location}
            onChange={(event) => setLocation(event.target.value)}
            placeholder="Kyiv / Remote"
          />
        </label>
        <label>
          Years of experience <span className="text-muted-foreground">(optional)</span>
          <input
            type="number"
            min={0}
            max={80}
            value={yearsOfExperience}
            onChange={(event) => setYearsOfExperience(event.target.value)}
            placeholder="5"
          />
        </label>
        <div className="fieldGroup">
          <span className="fieldLabel">Expected salary</span>
          <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_140px]">
            <label>
              Min <span className="text-muted-foreground">(optional)</span>
              <input
                type="number"
                min={0}
                value={salaryMin}
                onChange={(event) => setSalaryMin(event.target.value)}
                placeholder="2500"
              />
            </label>
            <label>
              Max <span className="text-muted-foreground">(optional)</span>
              <input
                type="number"
                min={0}
                value={salaryMax}
                onChange={(event) => setSalaryMax(event.target.value)}
                placeholder="4000"
              />
            </label>
            <label>
              Currency
              <select
                value={salaryCurrency}
                onChange={(event) => setSalaryCurrency(event.target.value)}
              >
                {PROFILE_SALARY_CURRENCY_OPTIONS.map((option) => (
                  <option key={option.id} value={option.id}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
        </div>
        <div className="fieldGroup">
          <span className="fieldLabel">Languages</span>
          <OptionCardGroup
            options={PROFILE_LANGUAGE_OPTIONS.map((option) => ({
              id: option.id,
              label: option.label,
            }))}
            value={languages}
            onToggle={onToggleLanguage}
          />
        </div>
        <label>
          <span className="flex items-center justify-between gap-3">
            CV / текст профілю
            <Button type="button" variant="ghost" size="sm" onClick={onOpenFilePicker}>
              <Upload size={13} />
              Завантажити .pdf / .txt / .md
            </Button>
          </span>
          <input
            ref={fileInputRef}
            type="file"
            accept=".pdf,.txt,.md,.text"
            className="hidden"
            onChange={onFileChange}
          />
          <textarea
            value={rawText}
            onChange={(event) => setRawText(event.target.value)}
            rows={12}
            placeholder="Вставте ваше CV, досвід, навички та цільові ролі. Або натисніть «Завантажити» для .txt / .md файлу."
            required
          />
        </label>

        <Button
          type="submit"
          disabled={isSaving || !name || !email || !rawText.trim()}
          className="w-full md:w-auto"
        >
          {isSaving ? 'Saving…' : profileExists ? 'Update Profile' : 'Create Profile'}
        </Button>
      </form>
    </>
  );
}

export function SearchProfileBuilderSection({
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
    <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
      <div className="mb-5 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Structured search profile
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Roles, regions, work mode, sources
            </span>
          </div>
          <h2 className="m-0 text-xl font-semibold text-card-foreground">
            Build from current raw text
          </h2>
          <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
            Uses the CV text above plus explicit preferences. No persistence required.
          </p>
        </div>
        <Button type="button" onClick={onBuild} disabled={isBuilding || !canBuild}>
          {isBuilding ? 'Building…' : 'Build search profile'}
        </Button>
      </div>

      <div className="flex flex-col gap-5">
        <div className="fieldGroup">
          <span className="fieldLabel">Target regions</span>
          <OptionCardGroup
            options={TARGET_REGION_OPTIONS}
            value={targetRegions}
            onToggle={onToggleTargetRegion}
          />
        </div>

        <div className="fieldGroup">
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
    </section>
  );
}

export function SearchProfileResultSection({
  result,
  roles,
  sources,
}: {
  result: SearchProfileBuildResult;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
}) {
  return (
    <div className="grid gap-6 xl:grid-cols-2">
      <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
        <div className="space-y-2">
          <p className="eyebrow">Analyzed profile</p>
          <h3 className="m-0 text-lg font-semibold text-card-foreground">
            Candidate summary from current text
          </h3>
        </div>
        <p className="m-0 leading-7 text-card-foreground">{result.analyzedProfile.summary}</p>

        <div className="detailGrid resultSection">
          <div>
            <span className="detailLabel">Primary role</span>
            <strong>{resolveRoleLabel(roles, result.analyzedProfile.primaryRole)}</strong>
          </div>
          <div>
            <span className="detailLabel">Seniority</span>
            <strong>{formatSeniorityLabel(result.analyzedProfile.seniority)}</strong>
          </div>
        </div>

        <div className="resultSection">
          <span className="detailLabel">Skills</span>
          <PillList items={result.analyzedProfile.skills} emptyLabel="No skills detected yet." />
        </div>

        <div className="resultSection">
          <span className="detailLabel">Suggested search terms</span>
          <PillList
            items={result.analyzedProfile.suggestedSearchTerms}
            emptyLabel="No suggested search terms returned."
          />
        </div>

        <div className="resultSection">
          <span className="detailLabel">Role candidates</span>
          {result.analyzedProfile.roleCandidates.length > 0 ? (
            <div className="stackList">
              {result.analyzedProfile.roleCandidates.map((candidate) => (
                <div key={candidate.role} className="stackListItem">
                  <strong>{resolveRoleLabel(roles, candidate.role)}</strong>
                  <span className="muted">
                    score {candidate.score} · confidence {candidate.confidence}%
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState message="No role candidates returned." />
          )}
        </div>
      </section>

      <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
        <div className="space-y-2">
          <p className="eyebrow">Search profile</p>
          <h3 className="m-0 text-lg font-semibold text-card-foreground">
            Structured preferences sent to ranking
          </h3>
        </div>

        <div className="detailGrid">
          <div>
            <span className="detailLabel">Primary role</span>
            <strong>{resolveRoleLabel(roles, result.searchProfile.primaryRole)}</strong>
          </div>
          <div>
            <span className="detailLabel">Seniority</span>
            <strong>{formatSeniorityLabel(result.searchProfile.seniority)}</strong>
          </div>
        </div>

        <SearchProfilePillSection
          label="Target roles"
          items={result.searchProfile.targetRoles.map((role) => resolveRoleLabel(roles, role))}
          emptyLabel="No target roles returned."
        />
        <SearchProfilePillSection
          label="Target regions"
          items={result.searchProfile.targetRegions.map((region) => formatFallbackLabel(region))}
          emptyLabel="No target regions selected."
        />
        <SearchProfilePillSection
          label="Work modes"
          items={result.searchProfile.workModes.map((mode) => formatFallbackLabel(mode))}
          emptyLabel="No work modes selected."
        />
        <SearchProfilePillSection
          label="Allowed sources"
          items={result.searchProfile.allowedSources.map((source) =>
            resolveSourceLabel(sources, source),
          )}
          emptyLabel="No source restrictions selected."
        />
        <SearchProfilePillSection
          label="Search terms"
          items={result.searchProfile.searchTerms}
          emptyLabel="No search terms returned."
        />
        <SearchProfilePillSection
          label="Exclude terms"
          items={result.searchProfile.excludeTerms}
          emptyLabel="No exclude terms selected."
        />
      </section>
    </div>
  );
}

export function RankedResultsSection({
  searchResult,
  searchError,
  buildResult,
  profileId,
  rawProfileText,
  llmContext,
  llmContextError,
  llmContextLoading,
  roles,
  sources,
  isRunning,
  onRunSearch,
}: {
  searchResult: SearchRunResult | null;
  searchError: string | null;
  buildResult: SearchProfileBuildResult;
  profileId: string | null;
  rawProfileText: string;
  llmContext: LlmContext | null;
  llmContextError: unknown;
  llmContextLoading: boolean;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
  isRunning: boolean;
  onRunSearch: () => void;
}) {
  return (
    <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
      <div className="mb-5 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Deterministic ranking
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Explainable fit reasons
            </span>
          </div>
          <h2 className="m-0 text-xl font-semibold text-card-foreground">
            Run deterministic search
          </h2>
          <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
            Uses the built search profile above and returns explainable ranked jobs.
          </p>
        </div>
        <Button type="button" onClick={onRunSearch} disabled={isRunning}>
          {isRunning ? 'Running…' : 'Run search'}
        </Button>
      </div>

      {searchError && <p className="error">{searchError}</p>}

      {isRunning ? (
        <p className="m-0 text-sm leading-6 text-muted-foreground">
          Running ranked search against active jobs…
        </p>
      ) : searchResult ? (
        <SearchResultsSection
          result={searchResult}
          buildResult={buildResult}
          profileId={profileId}
          rawProfileText={rawProfileText}
          llmContext={llmContext}
          llmContextError={llmContextError}
          llmContextLoading={llmContextLoading}
          roles={roles}
          sources={sources}
        />
      ) : (
        <p className="m-0 text-sm leading-6 text-muted-foreground">
          Build a search profile, then run search to inspect ranked jobs and fit reasons.
        </p>
      )}
    </section>
  );
}

export function LatestAnalysisSection({ summary, skills }: { summary?: string; skills: string[] }) {
  return (
    <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
      <div className="space-y-2">
        <p className="eyebrow">Latest analysis</p>
        <h3 className="m-0 text-lg font-semibold text-card-foreground">
          Persisted summary and extracted skills
        </h3>
      </div>
      {summary ? (
        <>
          <p className="m-0 leading-7 text-card-foreground">{summary}</p>
          <div className="resultSection">
            <PillList items={skills} emptyLabel="No skills returned." />
          </div>
        </>
      ) : (
        <p className="m-0 text-sm leading-6 text-muted-foreground">
          No persisted analysis yet. Save the profile, then run Analyze.
        </p>
      )}
    </section>
  );
}
