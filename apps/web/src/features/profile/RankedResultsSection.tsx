import type { LlmContext } from '../../api/analytics';
import type { SearchRunResult } from '../../api/jobs';
import type {
  RoleCatalogItem,
  SearchProfileBuildResult,
  SourceCatalogItem,
} from '../../api/profiles';

import { Button } from '../../components/ui/Button';
import { SurfaceSection } from '../../components/ui/Surface';
import { SearchResultsSection } from './SearchResultCard';

export function RankedResultsSection({
  searchResult,
  searchError,
  buildResult,
  buildIsCurrent,
  buildRestoredFromStorage,
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
  buildIsCurrent: boolean;
  buildRestoredFromStorage: boolean;
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
    <SurfaceSection>
      <div className="mb-5 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Deterministic ranking
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
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
        <Button type="button" onClick={onRunSearch} disabled={isRunning || !buildIsCurrent}>
          {isRunning ? 'Running…' : 'Run search'}
        </Button>
      </div>

      {searchError && <p className="error">{searchError}</p>}

      {!buildIsCurrent ? (
        <p className="m-0 text-sm leading-6 text-muted-foreground">
          Raw text or filters changed after the last build. Rebuild the search profile before
          running ranking again.
        </p>
      ) : null}

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
          {buildRestoredFromStorage
            ? 'The last built search profile was restored for these inputs. Run search to refresh ranked jobs and fit reasons.'
            : 'Build a search profile, then run search to inspect ranked jobs and fit reasons.'}
          </p>
      )}
    </SurfaceSection>
  );
}
