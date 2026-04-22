import { useEffect } from 'react';
import { ExternalLink } from 'lucide-react';

import type { LlmContext } from '../../api/analytics';
import type { RankedJobResult, SearchProfileBuildResult, SearchRunResult } from '../../api/jobs';
import type { RoleCatalogItem, SourceCatalogItem } from '../../api/profiles';
import { EmptyState } from '../../components/ui/EmptyState';
import { PillList } from '../../components/ui/PillList';
import { ACTIVE_ONLY_EMPTY_STATE_MESSAGE, getJobMetaLabels } from '../../lib/jobPresentation';
import { logJobImpressionsOnce } from '../events/jobImpressions';
import { getFitScoreTone, resolveRoleLabel, resolveSourceLabel } from './profile.utils';
import { SearchResultFitExplanation } from './SearchResultFitExplanation';

export function SearchProfilePillSection({
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

export function SearchResultsSection({
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
  const rerankerModeRequested = result.meta.rerankerModeRequested?.trim() || null;
  const rerankerModeActive = result.meta.rerankerModeActive?.trim() || null;
  const rerankerFallbackReason = result.meta.rerankerFallbackReason?.trim() || null;
  const showRerankerRuntime =
    import.meta.env.DEV ||
    Boolean(rerankerFallbackReason) ||
    (Boolean(rerankerModeRequested) &&
      Boolean(rerankerModeActive) &&
      rerankerModeRequested !== rerankerModeActive);

  useEffect(() => {
    void logJobImpressionsOnce({
      profileId,
      jobs: result.results.map((item) => item.job),
      surface: 'ranked_search_results',
    });
  }, [profileId, result.results]);

  return (
    <div className="resultSection">
      <p className="m-0 text-sm leading-6 text-muted-foreground">
        {summary}
        {filteredSummary}
      </p>

      {showRerankerRuntime && (
        <div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="inline-flex items-center rounded-full border border-border bg-white/[0.04] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Debug
            </span>
            <span className="text-xs font-medium text-card-foreground">Reranker runtime</span>
          </div>
          <div className="mt-2 flex flex-wrap gap-3 text-xs text-muted-foreground">
            {rerankerModeRequested && (
              <span>
                Requested <code className="font-mono text-[11px] text-card-foreground">{rerankerModeRequested}</code>
              </span>
            )}
            {rerankerModeActive && (
              <span>
                Active <code className="font-mono text-[11px] text-card-foreground">{rerankerModeActive}</code>
              </span>
            )}
            {(rerankerFallbackReason || import.meta.env.DEV) && (
              <span>
                Fallback{' '}
                <code className="font-mono text-[11px] text-card-foreground">
                  {rerankerFallbackReason ?? 'none'}
                </code>
              </span>
            )}
          </div>
        </div>
      )}

      {result.results.length === 0 ? (
        <EmptyState message={ACTIVE_ONLY_EMPTY_STATE_MESSAGE} />
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
  const fitExplanationScopeKey = [
    result.job.id,
    result.fit.score,
    searchProfile.primaryRole,
    searchProfile.searchTerms.join('|'),
    searchProfile.excludeTerms.join('|'),
    searchProfile.allowedSources.join('|'),
  ].join('::');
  const metaItems = getJobMetaLabels(result.job);

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

        <p className="m-0 text-sm text-muted-foreground">{displayCompany}</p>

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
          key={fitExplanationScopeKey}
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
