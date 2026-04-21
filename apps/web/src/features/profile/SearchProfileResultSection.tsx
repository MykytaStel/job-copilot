import type { RoleCatalogItem, SearchProfileBuildResult, SourceCatalogItem } from '../../api/profiles';

import { EmptyState } from '../../components/ui/EmptyState';
import { PillList } from '../../components/ui/PillList';
import { formatFallbackLabel } from '../../lib/format';
import { SearchProfilePillSection } from './SearchResultCard';
import { resolveRoleLabel, resolveSourceLabel } from './profile.utils';
import { formatSeniorityLabel } from './profileSection.utils';

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
          items={result.searchProfile.allowedSources.map((source) => resolveSourceLabel(sources, source))}
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
