import type { ReactElement, ReactNode } from 'react';
import { useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import {
  Activity,
  ArchiveX,
  ArrowRight,
  Bookmark,
  Briefcase,
  CalendarDays,
  Clock3,
  RefreshCw,
  Search,
  Send,
  SortAsc,
  XCircle,
} from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type {
  Application,
  ApplicationStatus,
  JobFeedSummary,
  JobPosting,
} from '@job-copilot/shared';

import {
  type RankedJob,
  addCompanyBlacklist,
  addCompanyWhitelist,
  createApplication,
  getApplications,
  getDashboardStats,
  getJobsFeed,
  hideJobForProfile,
  markJobBadFit,
  markJobSaved,
  removeCompanyBlacklist,
  removeCompanyWhitelist,
  getSources,
  rerankJobs,
} from '../api';
import { queryKeys } from '../queryKeys';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

function ScoreBadge({ score }: { score: number }) {
  const color =
    score >= 60 ? '#22c55e' : score >= 35 ? '#f59e0b' : '#6b7280';
  return (
    <span
      title={`ML fit score: ${score}/100`}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 3,
        fontSize: 12,
        fontWeight: 600,
        color,
        background: `${color}1a`,
        border: `1px solid ${color}40`,
        borderRadius: 6,
        padding: '2px 7px',
        flexShrink: 0,
      }}
    >
      {score}%
    </span>
  );
}

const STATUS_COLUMNS: ApplicationStatus[] = [
  'saved',
  'applied',
  'interview',
  'offer',
  'rejected',
];

const STATUS_ICONS = {
  saved: <Bookmark size={14} />,
  applied: <Send size={14} />,
  interview: <CalendarDays size={14} />,
  offer: <Briefcase size={14} />,
  rejected: <XCircle size={14} />,
} satisfies Record<ApplicationStatus, ReactElement>;

function formatTimestamp(value?: string) {
  if (!value) return 'n/a';

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return new Intl.DateTimeFormat('uk-UA', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

function lifecycleLabel(job: JobPosting) {
  switch (job.lifecycleStage) {
    case 'reactivated':
      return 'reactivated';
    case 'inactive':
      return 'inactive';
    default:
      return job.isActive === false ? 'inactive' : 'active';
  }
}

function lifecycleClass(job: JobPosting) {
  switch (job.lifecycleStage) {
    case 'reactivated':
      return 'jobState-reactivated';
    case 'inactive':
      return 'jobState-inactive';
    default:
      return 'jobState-active';
  }
}

function SourceActivityCard({
  label,
  value,
  icon,
  tone,
}: {
  label: string;
  value: number;
  icon: ReactNode;
  tone: string;
}) {
  return (
    <div className={`sourceActivityCard ${tone}`}>
      <div className="sourceActivityIcon">{icon}</div>
      <div>
        <div className="sourceActivityValue">{value}</div>
        <div className="sourceActivityLabel">{label}</div>
      </div>
    </div>
  );
}

type LifecycleFilter = 'all' | 'active' | 'inactive' | 'reactivated';

const LIFECYCLE_TABS: { value: LifecycleFilter; label: string }[] = [
  { value: 'all', label: 'Всі' },
  { value: 'active', label: 'Активні' },
  { value: 'inactive', label: 'Зникли' },
  { value: 'reactivated', label: 'Повернулись' },
];

const DEFAULT_LIFECYCLE_FILTER: LifecycleFilter = 'all';

function readLifecycleFilter(searchParams: URLSearchParams): LifecycleFilter {
  const lifecycle = searchParams.get('lifecycle');

  if (
    lifecycle === 'active' ||
    lifecycle === 'inactive' ||
    lifecycle === 'reactivated'
  ) {
    return lifecycle;
  }

  return DEFAULT_LIFECYCLE_FILTER;
}

function readSourceFilter(searchParams: URLSearchParams): string | null {
  const source = searchParams.get('source')?.trim();
  return source ? source : null;
}

export default function Dashboard() {
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [search, setSearch] = useState('');
  const [sortByScore, setSortByScore] = useState(false);
  const lifecycleFilter = readLifecycleFilter(searchParams);
  const sourceFilter = readSourceFilter(searchParams);
  const profileId = readProfileId();

  const updateFilters = ({
    lifecycle,
    source,
  }: {
    lifecycle?: LifecycleFilter;
    source?: string | null;
  }) => {
    const nextSearchParams = new URLSearchParams(searchParams);

    if (lifecycle !== undefined) {
      if (lifecycle === DEFAULT_LIFECYCLE_FILTER) {
        nextSearchParams.delete('lifecycle');
      } else {
        nextSearchParams.set('lifecycle', lifecycle);
      }
    }

    if (source !== undefined) {
      if (source) {
        nextSearchParams.set('source', source);
      } else {
        nextSearchParams.delete('source');
      }
    }

    setSearchParams(nextSearchParams, { replace: true });
  };

  const { data: jobsFeed, error: jobsError, isLoading: jobsLoading } = useQuery<{
    jobs: JobPosting[];
    summary: JobFeedSummary;
  }>({
    queryKey: queryKeys.jobs.filtered(lifecycleFilter, sourceFilter, profileId),
    queryFn: () =>
      getJobsFeed({
        lifecycle: lifecycleFilter === 'all' ? undefined : lifecycleFilter,
        source: sourceFilter ?? undefined,
        limit: 200,
      }),
  });

  const {
    data: sources = [],
    error: sourcesError,
  } = useQuery({
    queryKey: queryKeys.sources.all(),
    queryFn: getSources,
    staleTime: 5 * 60_000,
  });

  const allJobs = jobsFeed?.jobs ?? [];
  const jobSummary = jobsFeed?.summary;

  // ML rerank — fires once jobs are loaded and profile exists.
  const { data: rankData } = useQuery<RankedJob[]>({
    queryKey: queryKeys.ml.rerank(profileId ?? ''),
    queryFn: () => rerankJobs(profileId!, allJobs.map((j) => j.id)),
    enabled: !!profileId && allJobs.length > 0,
    staleTime: 5 * 60_000,
    retry: false,
  });

  const scoreById = useMemo(() => {
    const map = new Map<string, number>();
    rankData?.forEach((r) => map.set(r.jobId, r.score));
    return map;
  }, [rankData]);

  const jobs = useMemo(() => {
    const q = search.trim().toLowerCase();
    let list = q
      ? allJobs.filter(
          (j) =>
            j.title.toLowerCase().includes(q) ||
            j.company.toLowerCase().includes(q) ||
            j.description.toLowerCase().includes(q),
        )
      : [...allJobs];

    if (sortByScore && scoreById.size > 0) {
      list.sort((a, b) => (scoreById.get(b.id) ?? 0) - (scoreById.get(a.id) ?? 0));
    }
    return list;
  }, [allJobs, search, sortByScore, scoreById]);

  const { data: applications = [] } = useQuery<Application[]>({
    queryKey: queryKeys.applications.all(),
    queryFn: getApplications,
  });

  const { data: stats } = useQuery({
    queryKey: queryKeys.dashboard.stats(),
    queryFn: getDashboardStats,
  });

  const saveMutation = useMutation({
    mutationFn: async ({
      jobId,
      hasApplication,
    }: {
      jobId: string;
      hasApplication: boolean;
    }) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobSaved(profileId, jobId);

      if (!hasApplication) {
        await createApplication({ jobId, status: 'saved' });
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.applications.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.stats() });
      toast.success('Збережено в pipeline');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const hideMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await hideJobForProfile(profileId, jobId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      toast.success('Вакансію приховано');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const badFitMutation = useMutation({
    mutationFn: async (jobId: string) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      await markJobBadFit(profileId, jobId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      toast.success('Позначено як bad fit');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const companyFeedbackMutation = useMutation({
    mutationFn: async ({
      company,
      currentStatus,
      nextStatus,
    }: {
      company: string;
      currentStatus?: 'whitelist' | 'blacklist';
      nextStatus: 'whitelist' | 'blacklist';
    }) => {
      if (!profileId) {
        throw new Error('Create a profile first');
      }

      if (nextStatus === 'whitelist') {
        if (currentStatus === 'whitelist') {
          await removeCompanyWhitelist(profileId, company);
        } else {
          await addCompanyWhitelist(profileId, company);
        }
        return;
      }

      if (currentStatus === 'blacklist') {
        await removeCompanyBlacklist(profileId, company);
      } else {
        await addCompanyBlacklist(profileId, company);
      }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobs.all() });
      toast.success('Оновлено список компанії');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Помилка'),
  });

  const applicationByJob = new Map(applications.map((item) => [item.jobId, item]));
  const error =
    jobsError instanceof Error ? jobsError.message : jobsError ? 'Error' : null;

  return (
    <div className="dashboardPage">
      <section className="dashboardHero">
        <div>
          <p className="eyebrow">Job Copilot UA</p>
          <h1>Відстежуйте вакансії та заявки в одному місці.</h1>
          <p className="muted dashboardHeroText">
            Вакансії автоматично збираються з Djinni та Work.ua. Налаштуйте профіль — і система підбере найкращі збіги.
          </p>
        </div>
        <div className="dashboardHeroActions">
          <Link to="/profile" className="btn">
            Update Profile
          </Link>
          <Link to="/applications" className="ghostBtn">
            Review Pipeline <ArrowRight size={14} />
          </Link>
        </div>
      </section>

      {error && <p className="error">{error}</p>}

      {jobSummary && (
        <section className="sourceActivityGrid">
          <SourceActivityCard
            label="active now"
            value={jobSummary.activeJobs}
            icon={<Activity size={16} />}
            tone="tone-active"
          />
          <SourceActivityCard
            label="inactive after refresh"
            value={jobSummary.inactiveJobs}
            icon={<ArchiveX size={16} />}
            tone="tone-inactive"
          />
          <SourceActivityCard
            label="reactivated"
            value={jobSummary.reactivatedJobs}
            icon={<RefreshCw size={16} />}
            tone="tone-reactivated"
          />
          <SourceActivityCard
            label="jobs tracked"
            value={jobSummary.totalJobs}
            icon={<Briefcase size={16} />}
            tone="tone-neutral"
          />
        </section>
      )}

      {stats && (
        <section className="statsGrid">
          {STATUS_COLUMNS.map((status) => (
            <div key={status} className="statCard">
              <div className="statNumber">{stats.byStatus[status] ?? 0}</div>
              <div className="statLabel">
                <span
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 4,
                    justifyContent: 'center',
                  }}
                >
                  {STATUS_ICONS[status]}
                  {status}
                </span>
              </div>
            </div>
          ))}
          <div className="statCard">
            <div className="statNumber">{applications.length}</div>
            <div className="statLabel">applications tracked</div>
          </div>
        </section>
      )}

      <div className="jobsFilters">
        <div className="lifecycleTabs">
          {LIFECYCLE_TABS.map((tab) => (
            <button
              key={tab.value}
              className={lifecycleFilter === tab.value ? 'btn' : 'ghostBtn'}
              style={{ padding: '4px 12px', fontSize: 13 }}
              onClick={() => updateFilters({ lifecycle: tab.value })}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="sourceTabs">
          <button
            className={sourceFilter === null ? 'btn' : 'ghostBtn'}
            style={{ padding: '4px 12px', fontSize: 13 }}
            onClick={() => updateFilters({ source: null })}
          >
            Всі джерела
          </button>
          {sources.map((source) => (
            <button
              key={source.id}
              className={sourceFilter === source.id ? 'btn' : 'ghostBtn'}
              style={{ padding: '4px 12px', fontSize: 13 }}
              onClick={() => updateFilters({ source: source.id })}
            >
              {source.displayName}
            </button>
          ))}
        </div>
      </div>

      {sourcesError && (
        <p className="muted" style={{ marginTop: -8, marginBottom: 16, fontSize: 13 }}>
          Не вдалося завантажити каталог джерел. Фільтр вакансій за джерелом тимчасово недоступний.
        </p>
      )}

      <div className="jobsHeader">
        <div className="searchBox">
          <Search size={14} className="searchIcon" />
          <input
            type="search"
            placeholder="Фільтр за назвою, компанією, описом…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        {rankData && rankData.length > 0 && (
          <button
            className={sortByScore ? 'btn' : 'ghostBtn'}
            style={{ display: 'inline-flex', alignItems: 'center', gap: 6, fontSize: 13, padding: '6px 12px', flexShrink: 0 }}
            onClick={() => setSortByScore((v) => !v)}
            title="Сортувати за ML-релевантністю"
          >
            <SortAsc size={14} />
            {sortByScore ? 'За score' : 'За score'}
          </button>
        )}

        {search && (
          <p className="muted" style={{ margin: 0, fontSize: 13, flexShrink: 0 }}>
            {jobs.length} з {allJobs.length}
          </p>
        )}
      </div>

      {jobsLoading ? (
        <p className="muted">Завантаження вакансій…</p>
      ) : jobs.length === 0 ? (
        <p className="muted">{search ? 'Нічого не знайдено.' : 'Вакансій поки немає — запустіть pnpm scrape:djinni'}</p>
      ) : (
        <section className="jobLifecycleGrid">
          {jobs.map((job) => {
            const application = applicationByJob.get(job.id);
            const isSaved = job.feedback?.saved || !!application;
            const isBadFit = job.feedback?.badFit;
            const companyStatus = job.feedback?.companyStatus;

            return (
              <article key={job.id} className="jobLifecycleCard">
                <div className="jobLifecycleHeader">
                  <div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
                      <div className={`jobStatePill ${lifecycleClass(job)}`}>
                        {lifecycleLabel(job)}
                      </div>
                      {scoreById.has(job.id) && (
                        <ScoreBadge score={scoreById.get(job.id)!} />
                      )}
                    </div>
                    <h2>{job.title}</h2>
                    <p className="muted jobLifecycleCompany">{job.company}</p>
                  </div>
                  <div className="jobLifecycleActions">
                    {application ? (
                      <span className={`statusPill status-${application.status}`}>
                        {application.status}
                      </span>
                    ) : isSaved ? (
                      <span className="statusPill status-saved">saved</span>
                    ) : (
                      <button
                        className="ghostBtn"
                        style={{ display: 'inline-flex', alignItems: 'center', gap: 5, padding: '4px 10px', fontSize: 13 }}
                        disabled={saveMutation.isPending}
                        onClick={() =>
                          saveMutation.mutate({
                            jobId: job.id,
                            hasApplication: !!application,
                          })
                        }
                      >
                        <Bookmark size={13} /> Зберегти
                      </button>
                    )}
                    <Link to={`/jobs/${job.id}`} className="linkBtn">
                      Деталі <ArrowRight size={13} />
                    </Link>
                  </div>
                </div>

                <p className="jobLifecycleDescription">
                  {job.description.slice(0, 200)}
                  {job.description.length > 200 ? '…' : ''}
                </p>

                {(isBadFit || companyStatus) && (
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 8,
                      flexWrap: 'wrap',
                      marginBottom: 10,
                    }}
                  >
                    {isBadFit && <span className="statusPill status-rejected">bad fit</span>}
                    {companyStatus === 'blacklist' && (
                      <span className="statusPill status-rejected">company blacklisted</span>
                    )}
                    {companyStatus === 'whitelist' && (
                      <span className="statusPill status-saved">company whitelisted</span>
                    )}
                  </div>
                )}

                <div className="jobMetaRow">
                  <span className="jobMetaChip">
                    <Clock3 size={13} />
                    seen {formatTimestamp(job.lastSeenAt)}
                  </span>
                  {job.primaryVariant && (
                    <span className="jobMetaChip">
                      <RefreshCw size={13} />
                      {job.primaryVariant.source}
                    </span>
                  )}
                  {job.primaryVariant?.sourceUrl && (
                    <a
                      href={job.primaryVariant.sourceUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="jobMetaChip jobMetaLink"
                    >
                      source page
                    </a>
                  )}
                </div>

                <div className="jobTimeline">
                  <div>
                    <span>first seen</span>
                    <strong>{formatTimestamp(job.firstSeenAt)}</strong>
                  </div>
                  <div>
                    <span>last refresh</span>
                    <strong>{formatTimestamp(job.primaryVariant?.fetchedAt)}</strong>
                  </div>
                  <div>
                    <span>inactive at</span>
                    <strong>{formatTimestamp(job.inactivatedAt)}</strong>
                  </div>
                  <div>
                    <span>reactivated at</span>
                    <strong>{formatTimestamp(job.reactivatedAt)}</strong>
                  </div>
                </div>

                <div
                  style={{
                    display: 'flex',
                    flexWrap: 'wrap',
                    gap: 8,
                    marginTop: 14,
                  }}
                >
                  <button
                    className="ghostBtn"
                    style={{ padding: '4px 10px', fontSize: 13 }}
                    disabled={hideMutation.isPending}
                    onClick={() => hideMutation.mutate(job.id)}
                  >
                    Hide
                  </button>
                  <button
                    className="ghostBtn"
                    style={{ padding: '4px 10px', fontSize: 13 }}
                    disabled={badFitMutation.isPending || !!isBadFit}
                    onClick={() => badFitMutation.mutate(job.id)}
                  >
                    {isBadFit ? 'Bad fit' : 'Mark bad fit'}
                  </button>
                  <button
                    className="ghostBtn"
                    style={{ padding: '4px 10px', fontSize: 13 }}
                    disabled={companyFeedbackMutation.isPending}
                    onClick={() =>
                      companyFeedbackMutation.mutate({
                        company: job.company,
                        currentStatus: companyStatus,
                        nextStatus: 'whitelist',
                      })
                    }
                  >
                    {companyStatus === 'whitelist'
                      ? 'Unwhitelist company'
                      : 'Whitelist company'}
                  </button>
                  <button
                    className="ghostBtn"
                    style={{ padding: '4px 10px', fontSize: 13 }}
                    disabled={companyFeedbackMutation.isPending}
                    onClick={() =>
                      companyFeedbackMutation.mutate({
                        company: job.company,
                        currentStatus: companyStatus,
                        nextStatus: 'blacklist',
                      })
                    }
                  >
                    {companyStatus === 'blacklist'
                      ? 'Unblacklist company'
                      : 'Blacklist company'}
                  </button>
                </div>
              </article>
            );
          })}
        </section>
      )}
    </div>
  );
}
