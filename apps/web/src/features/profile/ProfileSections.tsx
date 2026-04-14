import type { ChangeEventHandler, RefObject } from 'react';
import { ExternalLink, Upload } from 'lucide-react';

import type {
  RankedJobResult,
  RoleCatalogItem,
  SearchProfileBuildResult,
  SearchRunResult,
  SearchTargetRegion,
  SearchWorkMode,
  SourceCatalogItem,
} from '../../api';
import { EmptyState } from '../../components/ui/EmptyState';
import { OptionCardGroup } from '../../components/ui/OptionCardGroup';
import { PillList } from '../../components/ui/PillList';
import { formatFallbackLabel } from '../../lib/format';
import { TARGET_REGION_OPTIONS, WORK_MODE_OPTIONS } from './profile.constants';
import { getFitScoreTone, resolveRoleLabel, resolveSourceLabel } from './profile.utils';

function renderErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

export function ProfileFormSection({
  name,
  email,
  location,
  rawText,
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
}: {
  name: string;
  email: string;
  location: string;
  rawText: string;
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
}) {
  return (
    <>
      <div className="pageHeader">
        <div>
          <h1>Profile</h1>
          <p className="muted">
            Persisted in `engine-api` and used for analysis/search-profile flows.
          </p>
        </div>
        <button type="button" onClick={onAnalyze} disabled={!profileExists || isAnalyzing}>
          {isAnalyzing ? 'Analyzing…' : 'Analyze'}
        </button>
      </div>

      <form
        className="card form"
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
          Location <span className="muted">(optional)</span>
          <input
            value={location}
            onChange={(event) => setLocation(event.target.value)}
            placeholder="Kyiv / Remote"
          />
        </label>
        <label>
          <span className="profileUploadLabel">
            CV / текст профілю
            <button
              type="button"
              className="ghostBtn ghostBtnCompact"
              onClick={onOpenFilePicker}
            >
              <Upload size={13} />
              Завантажити .pdf / .txt / .md
            </button>
          </span>
          <input
            ref={fileInputRef}
            type="file"
            accept=".pdf,.txt,.md,.text"
            className="visuallyHidden"
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

        <button
          type="submit"
          disabled={isSaving || !name || !email || !rawText.trim()}
        >
          {isSaving ? 'Saving…' : profileExists ? 'Update Profile' : 'Create Profile'}
        </button>
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
    <section className="card">
      <div className="sectionCardHeader">
        <div>
          <p className="eyebrow">Search profile builder</p>
          <h2>Build from current raw text</h2>
          <p className="muted sectionText">
            Uses the CV text above plus explicit preferences. No persistence required.
          </p>
        </div>
        <button type="button" onClick={onBuild} disabled={isBuilding || !canBuild}>
          {isBuilding ? 'Building…' : 'Build search profile'}
        </button>
      </div>

      <div className="form">
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
            <p className="error">
              {renderErrorMessage(sourcesError, 'Failed to load sources')}
            </p>
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
    <div className="grid">
      <section className="card">
        <p className="eyebrow">Analyzed profile</p>
        <p className="sectionText">{result.analyzedProfile.summary}</p>

        <div className="detailGrid resultSection">
          <div>
            <span className="detailLabel">Primary role</span>
            <strong>{resolveRoleLabel(roles, result.analyzedProfile.primaryRole)}</strong>
          </div>
          <div>
            <span className="detailLabel">Seniority</span>
            <strong>{formatFallbackLabel(result.analyzedProfile.seniority)}</strong>
          </div>
        </div>

        <div className="resultSection">
          <span className="detailLabel">Skills</span>
          <PillList
            items={result.analyzedProfile.skills}
            emptyLabel="No skills detected yet."
          />
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

      <section className="card">
        <p className="eyebrow">Search profile</p>

        <div className="detailGrid">
          <div>
            <span className="detailLabel">Primary role</span>
            <strong>{resolveRoleLabel(roles, result.searchProfile.primaryRole)}</strong>
          </div>
          <div>
            <span className="detailLabel">Seniority</span>
            <strong>{formatFallbackLabel(result.searchProfile.seniority)}</strong>
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
  roles,
  sources,
  isRunning,
  onRunSearch,
}: {
  searchResult: SearchRunResult | null;
  searchError: string | null;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
  isRunning: boolean;
  onRunSearch: () => void;
}) {
  return (
    <section className="card">
      <div className="sectionCardHeader sectionCardHeader-tight">
        <div>
          <p className="eyebrow">Ranked results</p>
          <h2>Run deterministic search</h2>
          <p className="muted sectionText">
            Uses the built search profile above and returns explainable ranked jobs.
          </p>
        </div>
        <button type="button" onClick={onRunSearch} disabled={isRunning}>
          {isRunning ? 'Running…' : 'Run search'}
        </button>
      </div>

      {searchError && <p className="error">{searchError}</p>}

      {isRunning ? (
        <p className="muted sectionText">Running ranked search against active jobs…</p>
      ) : searchResult ? (
        <SearchResultsSection result={searchResult} roles={roles} sources={sources} />
      ) : (
        <p className="muted sectionText">
          Build a search profile, then run search to inspect ranked jobs and fit reasons.
        </p>
      )}
    </section>
  );
}

export function LatestAnalysisSection({
  summary,
  skills,
}: {
  summary?: string;
  skills: string[];
}) {
  return (
    <section className="card">
      <p className="eyebrow">Latest analysis</p>
      {summary ? (
        <>
          <p className="sectionText">{summary}</p>
          <div className="resultSection">
            <PillList items={skills} emptyLabel="No skills returned." />
          </div>
        </>
      ) : (
        <p className="muted sectionText">
          No persisted analysis yet. Save the profile, then run Analyze.
        </p>
      )}
    </section>
  );
}

function SearchResultsSection({
  result,
  roles,
  sources,
}: {
  result: SearchRunResult;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
}) {
  const summary = `${result.meta.returnedJobs} ranked job${result.meta.returnedJobs === 1 ? '' : 's'} from ${result.meta.scoredJobs} scored candidate${result.meta.scoredJobs === 1 ? '' : 's'}.`;
  const filteredSummary =
    result.meta.filteredOutBySource > 0
      ? ` ${result.meta.filteredOutBySource} filtered out by source.`
      : '';

  return (
    <div className="resultSection">
      <p className="muted sectionText">
        {summary}
        {filteredSummary}
      </p>

      {result.results.length === 0 ? (
        <EmptyState message="No active jobs matched this search profile." />
      ) : (
        <div className="stackList">
          {result.results.map((item) => (
            <SearchResultCard
              key={item.job.id}
              result={item}
              roles={roles}
              sources={sources}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function SearchResultCard({
  result,
  roles,
  sources,
}: {
  result: RankedJobResult;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
}) {
  const sourceLabel = resolveSourceLabel(
    sources,
    result.job.primaryVariant?.source ?? result.source,
  );
  const scoreTone = getFitScoreTone(result.fit.score);
  const presentation = result.job.presentation;
  const displayTitle = presentation?.title || result.job.title;
  const displayCompany = presentation?.company || result.job.company;
  const displaySource = presentation?.sourceLabel || sourceLabel;
  const outboundUrl = presentation?.outboundUrl || result.job.url;
  const metaItems = [
    presentation?.locationLabel,
    presentation?.workModeLabel,
    presentation?.salaryLabel,
    presentation?.freshnessLabel,
  ].filter(Boolean) as string[];

  return (
    <article className="stackListItem searchResultCard">
      <div className="searchResultMain">
        <div className="searchResultHeader">
          <strong className="searchResultTitle">{displayTitle}</strong>
          <span className="badge badge-secondary">{displaySource}</span>
          {presentation?.badges.map((badge) => (
            <span key={badge} className="badge badge-secondary searchResultBadge">
              {badge}
            </span>
          ))}
        </div>

        <p className="muted sectionText">{displayCompany}</p>

        {presentation?.summary && (
          <p className="sectionText searchResultSummary">{presentation.summary}</p>
        )}

        {(metaItems.length > 0 || outboundUrl) && (
          <div className="searchResultMetaRow">
            {metaItems.map((item) => (
              <span key={item} className="jobMetaChip">
                {item}
              </span>
            ))}
            {outboundUrl && (
              <a
                href={outboundUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="linkBtn searchResultLink"
              >
                <ExternalLink size={13} />
                Open source
              </a>
            )}
          </div>
        )}

        <div className="resultSection">
          <span className="detailLabel">Fit reasons</span>
          {result.fit.reasons.length > 0 ? (
            <ul className="textList fitReasonsList">
              {result.fit.reasons.map((reason) => (
                <li key={reason}>{reason}</li>
              ))}
            </ul>
          ) : (
            <EmptyState message="No fit reasons returned." />
          )}
        </div>

        <SearchProfilePillSection
          label="Matched roles"
          items={result.fit.matchedRoles.map((role) => resolveRoleLabel(roles, role))}
          emptyLabel="No matched roles returned."
        />

        {(result.fit.matchedSkills.length > 0 || result.fit.matchedKeywords.length > 0) && (
          <div className="detailGrid resultSection">
            <div>
              <span className="detailLabel">Matched skills</span>
              <PillList
                items={result.fit.matchedSkills}
                emptyLabel="No matched skills returned."
                tone="success"
              />
            </div>
            <div>
              <span className="detailLabel">Matched keywords</span>
              <PillList
                items={result.fit.matchedKeywords}
                emptyLabel="No matched keywords returned."
                tone="success"
              />
            </div>
          </div>
        )}
      </div>

      <div className="fitScoreBadge">
        <span className={`badge fitScorePill fitScorePill-${scoreTone}`}>
          {result.fit.score}% fit
        </span>
      </div>
    </article>
  );
}

function SearchProfilePillSection({
  label,
  items,
  emptyLabel,
}: {
  label: string;
  items: string[];
  emptyLabel: string;
}) {
  return (
    <div className="resultSection">
      <span className="detailLabel">{label}</span>
      <PillList items={items} emptyLabel={emptyLabel} />
    </div>
  );
}
