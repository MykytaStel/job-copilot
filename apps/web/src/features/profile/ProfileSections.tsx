import type { ChangeEventHandler, RefObject } from 'react';
import { useEffect, useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { ExternalLink, Sparkles, Upload } from 'lucide-react';

import type {
  ApplicationCoach,
  CoverLetterDraft,
  InterviewPrep,
  JobFitExplanation,
  LlmContext,
  RankedJobResult,
  RoleCatalogItem,
  SearchProfileBuildResult,
  SearchRunResult,
  SearchTargetRegion,
  SearchWorkMode,
  SourceCatalogItem,
} from '../../api';
import {
  getApplicationCoach,
  getCoverLetterDraft,
  getInterviewPrep,
  getJobFitExplanation,
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
  buildResult,
  profileId,
  rawProfileText,
  llmContext,
  llmContextError,
  llmContextLoading,
  roles,
  sources,
}: {
  result: SearchRunResult;
  buildResult: SearchProfileBuildResult;
  profileId: string | null;
  rawProfileText: string;
  llmContext: LlmContext | null;
  llmContextError: unknown;
  llmContextLoading: boolean;
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
              analyzedProfile={buildResult.analyzedProfile}
              searchProfile={buildResult.searchProfile}
              profileId={profileId}
              rawProfileText={rawProfileText}
              llmContext={llmContext}
              llmContextError={llmContextError}
              llmContextLoading={llmContextLoading}
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
  analyzedProfile,
  searchProfile,
  profileId,
  rawProfileText,
  llmContext,
  llmContextError,
  llmContextLoading,
  roles,
  sources,
}: {
  result: RankedJobResult;
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'];
  searchProfile: SearchProfileBuildResult['searchProfile'];
  profileId: string | null;
  rawProfileText: string;
  llmContext: LlmContext | null;
  llmContextError: unknown;
  llmContextLoading: boolean;
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

        <SearchResultFitExplanation
          analyzedProfile={analyzedProfile}
          searchProfile={searchProfile}
          result={result}
          profileId={profileId}
          rawProfileText={rawProfileText}
          llmContext={llmContext}
          llmContextError={llmContextError}
          llmContextLoading={llmContextLoading}
        />
      </div>

      <div className="fitScoreBadge">
        <span className={`badge fitScorePill fitScorePill-${scoreTone}`}>
          {result.fit.score}% fit
        </span>
      </div>
    </article>
  );
}

function SearchResultFitExplanation({
  analyzedProfile,
  searchProfile,
  result,
  profileId,
  rawProfileText,
  llmContext,
  llmContextError,
  llmContextLoading,
}: {
  analyzedProfile: SearchProfileBuildResult['analyzedProfile'];
  searchProfile: SearchProfileBuildResult['searchProfile'];
  result: RankedJobResult;
  profileId: string | null;
  rawProfileText: string;
  llmContext: LlmContext | null;
  llmContextError: unknown;
  llmContextLoading: boolean;
}) {
  const [explanation, setExplanation] = useState<JobFitExplanation | null>(null);
  const [coaching, setCoaching] = useState<ApplicationCoach | null>(null);
  const [coverLetterDraft, setCoverLetterDraft] = useState<CoverLetterDraft | null>(null);
  const [interviewPrep, setInterviewPrep] = useState<InterviewPrep | null>(null);

  useEffect(() => {
    setExplanation(null);
    setCoaching(null);
    setCoverLetterDraft(null);
    setInterviewPrep(null);
  }, [
    result.job.id,
    result.fit.score,
    searchProfile.primaryRole,
    searchProfile.searchTerms.join('|'),
    searchProfile.excludeTerms.join('|'),
    searchProfile.allowedSources.join('|'),
  ]);

  const explainMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before requesting fit explanation.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getJobFitExplanation({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
      });
    },
    onSuccess: (payload) => {
      setExplanation(payload);
      setCoverLetterDraft(null);
      setInterviewPrep(null);
    },
  });

  const coachMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before requesting application coaching.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getApplicationCoach({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
      });
    },
    onSuccess: (payload) => {
      setCoaching(payload);
      setCoverLetterDraft(null);
      setInterviewPrep(null);
    },
  });

  const coverLetterMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before drafting a cover letter.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getCoverLetterDraft({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        applicationCoach: coaching,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
        rawProfileText: rawProfileText.trim() ? rawProfileText : null,
      });
    },
    onSuccess: (payload) => {
      setCoverLetterDraft(payload);
      setInterviewPrep(null);
    },
  });

  const interviewPrepMutation = useMutation({
    mutationFn: async () => {
      if (!profileId) {
        throw new Error('Profile is required before preparing interview guidance.');
      }
      if (!llmContext) {
        throw new Error('Feedback-aware context is not ready yet.');
      }

      return getInterviewPrep({
        profileId,
        analyzedProfile,
        searchProfile,
        rankedJob: result.job,
        deterministicFit: result.fit,
        jobFitExplanation: explanation,
        applicationCoach: coaching,
        coverLetterDraft,
        feedbackState: {
          feedbackSummary: llmContext.feedbackSummary,
          topPositiveEvidence: llmContext.topPositiveEvidence,
          topNegativeEvidence: llmContext.topNegativeEvidence,
          currentJobFeedback: result.job.feedback,
        },
        rawProfileText: rawProfileText.trim() ? rawProfileText : null,
      });
    },
    onSuccess: (payload) => {
      setInterviewPrep(payload);
    },
  });

  return (
    <div className="resultSection" style={{ marginTop: 12 }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 12,
          marginBottom: explanation || explainMutation.isPending || explainMutation.error ? 10 : 0,
        }}
      >
        <span className="detailLabel">LLM fit explanation</span>
        <button
          type="button"
          className="ghostBtn ghostBtnCompact"
          disabled={
            explainMutation.isPending ||
            !profileId ||
            llmContextLoading ||
            !!llmContextError ||
            !llmContext
          }
          onClick={() => explainMutation.mutate()}
        >
          <Sparkles size={13} />
          {explainMutation.isPending
            ? 'Explaining…'
            : explanation
              ? 'Refresh explanation'
              : 'Explain fit'}
        </button>
      </div>

      {llmContextLoading && (
        <p className="muted sectionText" style={{ margin: 0 }}>
          Feedback-aware context is loading. Fit explanation will be available once it is ready.
        </p>
      )}

      {!llmContextLoading && Boolean(llmContextError) && (
        <p className="error" style={{ marginBottom: 0 }}>
          {renderErrorMessage(llmContextError, 'Feedback-aware context is unavailable right now.')}
        </p>
      )}

      {explainMutation.error && (
        <p className="error" style={{ marginBottom: 0 }}>
          {renderErrorMessage(explainMutation.error, 'Fit explanation is unavailable right now.')}
        </p>
      )}

      {explanation && <FitExplanationPanel explanation={explanation} />}

      {!llmContextLoading && !llmContextError && llmContext && (
        <div style={{ marginTop: explanation ? 12 : 0 }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              gap: 12,
              marginBottom: coaching || coachMutation.isPending || coachMutation.error ? 10 : 0,
            }}
          >
            <span className="detailLabel">Application coaching</span>
            <button
              type="button"
              className="ghostBtn ghostBtnCompact"
              disabled={coachMutation.isPending || !profileId}
              onClick={() => coachMutation.mutate()}
            >
              <Sparkles size={13} />
              {coachMutation.isPending
                ? 'Coaching…'
                : coaching
                  ? 'Refresh coaching'
                  : 'Coach application'}
            </button>
          </div>

          {coachMutation.error && (
            <p className="error" style={{ marginBottom: 0 }}>
              {renderErrorMessage(coachMutation.error, 'Application coaching is unavailable right now.')}
            </p>
          )}

          {coaching && <ApplicationCoachPanel coaching={coaching} />}

          <div style={{ marginTop: coaching ? 12 : 0 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 12,
                marginBottom:
                  coverLetterDraft || coverLetterMutation.isPending || coverLetterMutation.error
                    ? 10
                    : 0,
              }}
            >
              <span className="detailLabel">Cover letter draft</span>
              <button
                type="button"
                className="ghostBtn ghostBtnCompact"
                disabled={coverLetterMutation.isPending || !profileId}
                onClick={() => coverLetterMutation.mutate()}
              >
                <Sparkles size={13} />
                {coverLetterMutation.isPending
                  ? 'Drafting…'
                  : coverLetterDraft
                    ? 'Refresh draft'
                    : 'Draft cover letter'}
              </button>
            </div>

            {coverLetterMutation.error && (
              <p className="error" style={{ marginBottom: 0 }}>
                {renderErrorMessage(
                  coverLetterMutation.error,
                  'Cover letter drafting is unavailable right now.',
                )}
              </p>
            )}

            {coverLetterDraft && <CoverLetterDraftPanel draft={coverLetterDraft} />}
          </div>

          <div style={{ marginTop: coverLetterDraft ? 12 : 0 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 12,
                marginBottom:
                  interviewPrep || interviewPrepMutation.isPending || interviewPrepMutation.error
                    ? 10
                    : 0,
              }}
            >
              <span className="detailLabel">Interview prep pack</span>
              <button
                type="button"
                className="ghostBtn ghostBtnCompact"
                disabled={interviewPrepMutation.isPending || !profileId}
                onClick={() => interviewPrepMutation.mutate()}
              >
                <Sparkles size={13} />
                {interviewPrepMutation.isPending
                  ? 'Preparing…'
                  : interviewPrep
                    ? 'Refresh prep'
                    : 'Prepare interview'}
              </button>
            </div>

            {interviewPrepMutation.error && (
              <p className="error" style={{ marginBottom: 0 }}>
                {renderErrorMessage(
                  interviewPrepMutation.error,
                  'Interview prep is unavailable right now.',
                )}
              </p>
            )}

            {interviewPrep && <InterviewPrepPanel prep={interviewPrep} />}
          </div>
        </div>
      )}
    </div>
  );
}

function FitExplanationPanel({ explanation }: { explanation: JobFitExplanation }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
        padding: '14px 16px',
        borderRadius: 14,
        border: '1px solid rgba(149,167,255,0.18)',
        background:
          'linear-gradient(180deg, rgba(18,26,40,0.92) 0%, rgba(13,19,31,0.88) 100%)',
      }}
    >
      <p
        style={{
          margin: 0,
          fontSize: 14,
          lineHeight: 1.6,
          color: 'var(--color-text-primary)',
        }}
      >
        {explanation.fitSummary || 'No summary returned.'}
      </p>

      <FitExplanationList
        label="Why it matches"
        items={explanation.whyItMatches}
        emptyLabel="No supporting signals returned."
      />
      <FitExplanationList
        label="Risks"
        items={explanation.risks}
        emptyLabel="No explicit risks returned."
      />
      <FitExplanationList
        label="Missing signals"
        items={explanation.missingSignals}
        emptyLabel="No missing signals returned."
      />

      <div className="detailGrid">
        <div>
          <span className="detailLabel">Recommended next step</span>
          <p className="sectionText" style={{ marginBottom: 0 }}>
            {explanation.recommendedNextStep || 'No next step returned.'}
          </p>
        </div>
        <div>
          <span className="detailLabel">Application angle</span>
          <p className="sectionText" style={{ marginBottom: 0 }}>
            {explanation.applicationAngle || 'No application angle returned.'}
          </p>
        </div>
      </div>
    </div>
  );
}

function FitExplanationList({
  label,
  items,
  emptyLabel,
}: {
  label: string;
  items: string[];
  emptyLabel: string;
}) {
  return (
    <div>
      <span className="detailLabel">{label}</span>
      {items.length > 0 ? (
        <ul className="textList fitReasonsList" style={{ marginTop: 8 }}>
          {items.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      ) : (
        <p className="muted sectionText" style={{ marginBottom: 0 }}>
          {emptyLabel}
        </p>
      )}
    </div>
  );
}

function ApplicationCoachPanel({ coaching }: { coaching: ApplicationCoach }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
        padding: '14px 16px',
        borderRadius: 14,
        border: '1px solid rgba(91,180,255,0.18)',
        background:
          'linear-gradient(180deg, rgba(12,22,35,0.94) 0%, rgba(10,16,28,0.9) 100%)',
      }}
    >
      <p
        style={{
          margin: 0,
          fontSize: 14,
          lineHeight: 1.6,
          color: 'var(--color-text-primary)',
        }}
      >
        {coaching.applicationSummary || 'No application summary returned.'}
      </p>

      <FitExplanationList
        label="Resume focus points"
        items={coaching.resumeFocusPoints}
        emptyLabel="No resume focus points returned."
      />
      <FitExplanationList
        label="Suggested bullets"
        items={coaching.suggestedBullets}
        emptyLabel="No suggested bullets returned."
      />
      <FitExplanationList
        label="Cover letter angles"
        items={coaching.coverLetterAngles}
        emptyLabel="No cover letter angles returned."
      />
      <FitExplanationList
        label="Interview focus"
        items={coaching.interviewFocus}
        emptyLabel="No interview focus returned."
      />
      <FitExplanationList
        label="Gaps to address"
        items={coaching.gapsToAddress}
        emptyLabel="No explicit gaps returned."
      />
      <FitExplanationList
        label="Red flags"
        items={coaching.redFlags}
        emptyLabel="No explicit red flags returned."
      />
    </div>
  );
}

function CoverLetterDraftPanel({ draft }: { draft: CoverLetterDraft }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
        padding: '14px 16px',
        borderRadius: 14,
        border: '1px solid rgba(255,197,113,0.2)',
        background:
          'linear-gradient(180deg, rgba(36,26,15,0.92) 0%, rgba(24,18,12,0.9) 100%)',
      }}
    >
      <div>
        <span className="detailLabel">Draft summary</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {draft.draftSummary || 'No draft summary returned.'}
        </p>
      </div>

      <div>
        <span className="detailLabel">Opening paragraph</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {draft.openingParagraph || 'No opening paragraph returned.'}
        </p>
      </div>

      <div>
        <span className="detailLabel">Body paragraphs</span>
        {draft.bodyParagraphs.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10, marginTop: 8 }}>
            {draft.bodyParagraphs.map((paragraph, index) => (
              <p key={`${index}-${paragraph}`} className="sectionText" style={{ marginBottom: 0 }}>
                {paragraph}
              </p>
            ))}
          </div>
        ) : (
          <p className="muted sectionText" style={{ marginBottom: 0 }}>
            No body paragraphs returned.
          </p>
        )}
      </div>

      <div>
        <span className="detailLabel">Closing paragraph</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {draft.closingParagraph || 'No closing paragraph returned.'}
        </p>
      </div>

      <FitExplanationList
        label="Key claims used"
        items={draft.keyClaimsUsed}
        emptyLabel="No grounded claims returned."
      />
      <FitExplanationList
        label="Evidence gaps"
        items={draft.evidenceGaps}
        emptyLabel="No evidence gaps returned."
      />
      <FitExplanationList
        label="Tone notes"
        items={draft.toneNotes}
        emptyLabel="No tone notes returned."
      />
    </div>
  );
}

function InterviewPrepPanel({ prep }: { prep: InterviewPrep }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 12,
        padding: '14px 16px',
        borderRadius: 14,
        border: '1px solid rgba(120,223,165,0.18)',
        background:
          'linear-gradient(180deg, rgba(13,31,24,0.94) 0%, rgba(10,21,17,0.9) 100%)',
      }}
    >
      <div>
        <span className="detailLabel">Prep summary</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {prep.prepSummary || 'No prep summary returned.'}
        </p>
      </div>

      <FitExplanationList
        label="Likely topics"
        items={prep.likelyTopics}
        emptyLabel="No likely topics returned."
      />
      <FitExplanationList
        label="Technical focus"
        items={prep.technicalFocus}
        emptyLabel="No technical focus returned."
      />
      <FitExplanationList
        label="Behavioral focus"
        items={prep.behavioralFocus}
        emptyLabel="No behavioral focus returned."
      />
      <FitExplanationList
        label="Stories to prepare"
        items={prep.storiesToPrepare}
        emptyLabel="No story prompts returned."
      />
      <FitExplanationList
        label="Questions to ask"
        items={prep.questionsToAsk}
        emptyLabel="No questions returned."
      />
      <FitExplanationList
        label="Risk areas"
        items={prep.riskAreas}
        emptyLabel="No risk areas returned."
      />
      <FitExplanationList
        label="Follow-up plan"
        items={prep.followUpPlan}
        emptyLabel="No follow-up plan returned."
      />
    </div>
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
