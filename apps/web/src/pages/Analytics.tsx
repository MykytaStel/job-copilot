import { useQuery } from '@tanstack/react-query';
import {
  BarChart2,
  Bookmark,
  Brain,
  Building2,
  Eye,
  EyeOff,
  Hash,
  Layers,
  ShieldCheck,
  ShieldOff,
  ThumbsDown,
  Zap,
} from 'lucide-react';

import { getAnalyticsSummary, getFunnelSummary, getLlmContext, getProfileInsights } from '../api';
import type { AnalyticsSummary, FunnelSummary, LlmContext, ProfileInsights } from '../api';
import { queryKeys } from '../queryKeys';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

// ─── Primitives ───────────────────────────────────────────────────────────────

function SummaryCard({
  label,
  count,
  icon,
  color,
}: {
  label: string;
  count: number;
  icon: React.ReactNode;
  color: string;
}) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        padding: '14px 18px',
        borderRadius: 16,
        border: '1px solid var(--color-border-soft)',
        background: 'rgba(18, 25, 39, 0.85)',
        minWidth: 140,
      }}
    >
      <span style={{ color, opacity: 0.85 }}>{icon}</span>
      <div>
        <div style={{ fontSize: 24, fontWeight: 700, color: 'var(--color-text-strong)', lineHeight: 1 }}>
          {count}
        </div>
        <div
          style={{
            fontSize: 11,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            color: 'var(--color-text-secondary)',
            marginTop: 3,
          }}
        >
          {label}
        </div>
      </div>
    </div>
  );
}

function Section({
  title,
  icon,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section
      style={{
        background: 'var(--color-bg-elevated)',
        border: '1px solid var(--color-border-soft)',
        borderRadius: 20,
        padding: '20px 24px',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 16 }}>
        <span style={{ color: 'var(--color-text-secondary)' }}>{icon}</span>
        <h2
          style={{
            margin: 0,
            fontSize: 15,
            fontWeight: 600,
            color: 'var(--color-text-primary)',
          }}
        >
          {title}
        </h2>
      </div>
      {children}
    </section>
  );
}

// ─── Bar chart (CSS-only, no library) ────────────────────────────────────────

function BarRow({
  label,
  value,
  maxValue,
  color,
}: {
  label: string;
  value: number;
  maxValue: number;
  color: string;
}) {
  const pct = maxValue > 0 ? Math.round((value / maxValue) * 100) : 0;

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8 }}>
      <div
        style={{
          width: 110,
          fontSize: 12,
          color: 'var(--color-text-secondary)',
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          flexShrink: 0,
          textAlign: 'right',
        }}
      >
        {label}
      </div>
      <div
        style={{
          flex: 1,
          height: 8,
          background: 'var(--color-bg-hover)',
          borderRadius: 4,
          overflow: 'hidden',
        }}
      >
        <div
          style={{
            width: `${pct}%`,
            height: '100%',
            background: color,
            borderRadius: 4,
            transition: 'width 0.3s ease',
          }}
        />
      </div>
      <div
        style={{
          width: 32,
          fontSize: 12,
          fontWeight: 600,
          color: 'var(--color-text-primary)',
          textAlign: 'right',
          flexShrink: 0,
        }}
      >
        {value}
      </div>
    </div>
  );
}

function TagList({ items, color }: { items: string[]; color: string }) {
  if (items.length === 0) {
    return <p className="emptyState" style={{ margin: 0 }}>None yet.</p>;
  }

  return (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
      {items.map((item) => (
        <span
          key={item}
          style={{
            padding: '3px 10px',
            borderRadius: 20,
            fontSize: 12,
            fontWeight: 500,
            background: 'var(--color-bg-hover)',
            border: `1px solid ${color}33`,
            color,
          }}
        >
          {item}
        </span>
      ))}
    </div>
  );
}

function SimpleList({
  items,
  empty,
  color,
}: {
  items: string[];
  empty: string;
  color: string;
}) {
  if (items.length === 0) {
    return <p className="emptyState" style={{ margin: 0 }}>{empty}</p>;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {items.map((item) => (
        <div
          key={item}
          style={{
            display: 'flex',
            alignItems: 'flex-start',
            gap: 8,
            fontSize: 13,
            color: 'var(--color-text-primary)',
            lineHeight: 1.45,
          }}
        >
          <span style={{ color, marginTop: 1 }}>-</span>
          <span>{item}</span>
        </div>
      ))}
    </div>
  );
}

function formatPercent(rate: number) {
  return `${Math.round(rate * 100)}%`;
}

function ConversionCard({
  label,
  rate,
  numerator,
  denominator,
  color,
}: {
  label: string;
  rate: number;
  numerator: number;
  denominator: number;
  color: string;
}) {
  const width = `${Math.max(0, Math.min(rate, 1)) * 100}%`;

  return (
    <div
      style={{
        padding: '14px 16px',
        borderRadius: 16,
        border: '1px solid var(--color-border-soft)',
        background: 'rgba(18, 25, 39, 0.65)',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 12,
          marginBottom: 10,
        }}
      >
        <div style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>{label}</div>
        <div style={{ fontSize: 18, fontWeight: 700, color }}>{formatPercent(rate)}</div>
      </div>
      <div
        style={{
          height: 8,
          borderRadius: 999,
          background: 'var(--color-bg-hover)',
          overflow: 'hidden',
          marginBottom: 8,
        }}
      >
        <div
          style={{
            width,
            height: '100%',
            background: color,
            borderRadius: 999,
            transition: 'width 0.3s ease',
          }}
        />
      </div>
      <div style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>
        {numerator} / {denominator}
      </div>
    </div>
  );
}

// ─── Source distribution ──────────────────────────────────────────────────────

function SourceDistribution({ summary }: { summary: AnalyticsSummary }) {
  const max = Math.max(...summary.jobsBySource.map((s) => s.count), 1);
  const colors = ['#95a7ff', '#c9fff8', '#ffd6a5', '#ffb4b4', '#b9fbc0'];

  if (summary.jobsBySource.length === 0) {
    return <p className="emptyState" style={{ margin: 0 }}>No source data yet.</p>;
  }

  return (
    <div>
      {summary.jobsBySource.map((entry, i) => (
        <BarRow
          key={entry.source}
          label={entry.source}
          value={entry.count}
          maxValue={max}
          color={colors[i % colors.length]}
        />
      ))}
    </div>
  );
}

// ─── Lifecycle distribution ───────────────────────────────────────────────────

function LifecycleDistribution({ summary }: { summary: AnalyticsSummary }) {
  const lc = summary.jobsByLifecycle;
  const max = Math.max(lc.active, lc.inactive, lc.reactivated, 1);

  return (
    <div>
      <BarRow label="Active" value={lc.active} maxValue={max} color="#b9fbc0" />
      <BarRow label="Inactive" value={lc.inactive} maxValue={max} color="#9aa8bc" />
      <BarRow label="Reactivated" value={lc.reactivated} maxValue={max} color="#95a7ff" />
      <div
        style={{
          marginTop: 10,
          fontSize: 12,
          color: 'var(--color-text-secondary)',
        }}
      >
        Total indexed: <strong style={{ color: 'var(--color-text-primary)' }}>{lc.total}</strong>
      </div>
    </div>
  );
}

function FunnelSourceBreakdown({ summary }: { summary: FunnelSummary }) {
  const rows = Array.from(
    new Set([
      ...summary.impressionsBySource.map((entry) => entry.source),
      ...summary.opensBySource.map((entry) => entry.source),
      ...summary.savesBySource.map((entry) => entry.source),
      ...summary.applicationsBySource.map((entry) => entry.source),
    ]),
  ).map((source) => ({
    source,
    impressions:
      summary.impressionsBySource.find((entry) => entry.source === source)?.count ?? 0,
    opens: summary.opensBySource.find((entry) => entry.source === source)?.count ?? 0,
    saves: summary.savesBySource.find((entry) => entry.source === source)?.count ?? 0,
    applications:
      summary.applicationsBySource.find((entry) => entry.source === source)?.count ?? 0,
  }));

  if (rows.length === 0) {
    return <p className="emptyState" style={{ margin: 0 }}>No source funnel data yet.</p>;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      {rows.map((row) => (
        <div
          key={row.source}
          style={{
            display: 'grid',
            gridTemplateColumns: 'minmax(120px, 1.3fr) repeat(4, minmax(0, 1fr))',
            gap: 12,
            padding: '12px 14px',
            borderRadius: 14,
            border: '1px solid var(--color-border-soft)',
            background: 'rgba(18, 25, 39, 0.55)',
            fontSize: 12,
          }}
        >
          <div style={{ color: 'var(--color-text-primary)', fontWeight: 600 }}>{row.source}</div>
          <div style={{ color: 'var(--color-text-secondary)' }}>Impr. {row.impressions}</div>
          <div style={{ color: 'var(--color-text-secondary)' }}>Open {row.opens}</div>
          <div style={{ color: 'var(--color-text-secondary)' }}>Save {row.saves}</div>
          <div style={{ color: 'var(--color-text-secondary)' }}>Apply {row.applications}</div>
        </div>
      ))}
    </div>
  );
}

// ─── LLM context panel ────────────────────────────────────────────────────────

function LlmContextPanel({ ctx }: { ctx: LlmContext }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {ctx.analyzedProfile && (
        <div>
          <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 4 }}>
            Primary role
          </div>
          <span
            style={{
              padding: '3px 12px',
              borderRadius: 20,
              fontSize: 13,
              fontWeight: 600,
              background: 'var(--color-bg-hover)',
              color: '#95a7ff',
              border: '1px solid #95a7ff33',
            }}
          >
            {ctx.analyzedProfile.primaryRole}
          </span>
          <span
            style={{
              marginLeft: 8,
              padding: '3px 10px',
              borderRadius: 20,
              fontSize: 12,
              background: 'var(--color-bg-hover)',
              color: 'var(--color-text-secondary)',
            }}
          >
            {ctx.analyzedProfile.seniority}
          </span>
        </div>
      )}

      <div>
        <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 6 }}>
          Positive signals
        </div>
        {ctx.topPositiveEvidence.length === 0 ? (
          <p className="emptyState" style={{ margin: 0 }}>No positive signals yet.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {ctx.topPositiveEvidence.slice(0, 8).map((entry, i) => (
              <div
                key={i}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  fontSize: 12,
                  color: 'var(--color-text-primary)',
                }}
              >
                <span
                  style={{
                    padding: '1px 7px',
                    borderRadius: 8,
                    fontSize: 10,
                    background: '#b9fbc022',
                    color: '#b9fbc0',
                    flexShrink: 0,
                  }}
                >
                  {entry.type === 'saved_job' ? 'saved' : 'whitelist'}
                </span>
                {entry.label}
              </div>
            ))}
          </div>
        )}
      </div>

      <div>
        <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 6 }}>
          Negative signals
        </div>
        {ctx.topNegativeEvidence.length === 0 ? (
          <p className="emptyState" style={{ margin: 0 }}>No negative signals yet.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {ctx.topNegativeEvidence.slice(0, 8).map((entry, i) => (
              <div
                key={i}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  fontSize: 12,
                  color: 'var(--color-text-primary)',
                }}
              >
                <span
                  style={{
                    padding: '1px 7px',
                    borderRadius: 8,
                    fontSize: 10,
                    background: '#ffb4b422',
                    color: '#ffb4b4',
                    flexShrink: 0,
                  }}
                >
                  {entry.type === 'bad_fit_job' ? 'bad fit' : 'blacklist'}
                </span>
                {entry.label}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function ProfileInsightsPanel({ insights }: { insights: ProfileInsights }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        <div style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>Profile summary</div>
        <p style={{ margin: 0, fontSize: 14, lineHeight: 1.6, color: 'var(--color-text-primary)' }}>
          {insights.profileSummary || 'No summary generated yet.'}
        </p>
      </div>

      {insights.searchStrategySummary && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ fontSize: 12, color: 'var(--color-text-secondary)' }}>Search strategy</div>
          <p style={{ margin: 0, fontSize: 13, lineHeight: 1.55, color: 'var(--color-text-secondary)' }}>
            {insights.searchStrategySummary}
          </p>
        </div>
      )}

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
          gap: 16,
        }}
      >
        <div>
          <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 8 }}>Strengths</div>
          <SimpleList items={insights.strengths} empty="No strengths highlighted yet." color="#b9fbc0" />
        </div>

        <div>
          <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 8 }}>Risks</div>
          <SimpleList items={insights.risks} empty="No risks highlighted yet." color="#ffb4b4" />
        </div>

        <div>
          <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 8 }}>
            Recommended actions
          </div>
          <SimpleList
            items={insights.recommendedActions}
            empty="No actions suggested yet."
            color="#95a7ff"
          />
        </div>
      </div>

      <div>
        <div style={{ fontSize: 12, color: 'var(--color-text-secondary)', marginBottom: 8 }}>
          Search term suggestions
        </div>
        <TagList items={insights.searchTermSuggestions} color="#ffd6a5" />
      </div>
    </div>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function Analytics() {
  const profileId = readProfileId();

  const { data: summary, isLoading: summaryLoading } = useQuery({
    queryKey: queryKeys.analytics.summary(profileId ?? ''),
    queryFn: () => getAnalyticsSummary(profileId!),
    enabled: !!profileId,
  });

  const { data: funnel, isLoading: funnelLoading } = useQuery({
    queryKey: queryKeys.analytics.funnel(profileId ?? ''),
    queryFn: () => getFunnelSummary(profileId!),
    enabled: !!profileId,
  });

  const { data: llmCtx, isLoading: ctxLoading } = useQuery({
    queryKey: queryKeys.analytics.llmContext(profileId ?? ''),
    queryFn: () => getLlmContext(profileId!),
    enabled: !!profileId,
  });

  const enrichmentContextVersion = llmCtx ? JSON.stringify(llmCtx) : '';

  const {
    data: profileInsights,
    isLoading: insightsLoading,
    error: insightsError,
  } = useQuery({
    queryKey: queryKeys.analytics.profileInsights(profileId ?? '', enrichmentContextVersion),
    queryFn: () => getProfileInsights(llmCtx!),
    enabled: !!profileId && !!llmCtx,
    retry: 0,
  });

  if (!profileId) {
    return (
      <div className="jobDetails">
        <p className="emptyState">Create a profile to view analytics.</p>
      </div>
    );
  }

  const isLoading = summaryLoading || funnelLoading || ctxLoading;

  return (
    <div className="jobDetails">
      <div className="pageHeader">
        <div>
          <p className="eyebrow">Insights</p>
          <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, color: 'var(--color-text-strong)' }}>
            Analytics
          </h1>
        </div>
      </div>

      {isLoading ? (
        <p className="emptyState">Loading analytics…</p>
      ) : summary ? (
        <>
          {/* Feedback summary cards */}
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
            <SummaryCard
              label="Saved"
              count={summary.feedback.savedJobsCount}
              icon={<Bookmark size={18} />}
              color="#95a7ff"
            />
            <SummaryCard
              label="Hidden"
              count={summary.feedback.hiddenJobsCount}
              icon={<EyeOff size={18} />}
              color="#9aa8bc"
            />
            <SummaryCard
              label="Bad Fit"
              count={summary.feedback.badFitJobsCount}
              icon={<ThumbsDown size={18} />}
              color="#ffb4b4"
            />
            <SummaryCard
              label="Whitelisted"
              count={summary.feedback.whitelistedCompaniesCount}
              icon={<ShieldCheck size={18} />}
              color="#c9fff8"
            />
            <SummaryCard
              label="Blacklisted"
              count={summary.feedback.blacklistedCompaniesCount}
              icon={<ShieldOff size={18} />}
              color="#ffb4b4"
            />
            <SummaryCard
              label="Total Jobs"
              count={summary.jobsByLifecycle.total}
              icon={<Layers size={18} />}
              color="#ffd6a5"
            />
          </div>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
              gap: 16,
            }}
          >
            {funnel && (
              <Section title="Job Funnel" icon={<Eye size={16} />}>
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
                    gap: 10,
                    marginBottom: 16,
                  }}
                >
                  <SummaryCard
                    label="Impressions"
                    count={funnel.impressionCount}
                    icon={<Eye size={18} />}
                    color="#95a7ff"
                  />
                  <SummaryCard
                    label="Opens"
                    count={funnel.openCount}
                    icon={<BarChart2 size={18} />}
                    color="#c9fff8"
                  />
                  <SummaryCard
                    label="Saves"
                    count={funnel.saveCount}
                    icon={<Bookmark size={18} />}
                    color="#ffd6a5"
                  />
                  <SummaryCard
                    label="Applications"
                    count={funnel.applicationCreatedCount}
                    icon={<Zap size={18} />}
                    color="#b9fbc0"
                  />
                </div>

                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
                    gap: 12,
                    marginBottom: 16,
                  }}
                >
                  <ConversionCard
                    label="Open from impressions"
                    rate={funnel.conversionRates.openRateFromImpressions}
                    numerator={funnel.openCount}
                    denominator={funnel.impressionCount}
                    color="#95a7ff"
                  />
                  <ConversionCard
                    label="Save from opens"
                    rate={funnel.conversionRates.saveRateFromOpens}
                    numerator={funnel.saveCount}
                    denominator={funnel.openCount}
                    color="#ffd6a5"
                  />
                  <ConversionCard
                    label="Apply from saves"
                    rate={funnel.conversionRates.applicationRateFromSaves}
                    numerator={funnel.applicationCreatedCount}
                    denominator={funnel.saveCount}
                    color="#b9fbc0"
                  />
                </div>

                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
                    gap: 10,
                  }}
                >
                  <SummaryCard
                    label="Hidden"
                    count={funnel.hideCount}
                    icon={<EyeOff size={18} />}
                    color="#9aa8bc"
                  />
                  <SummaryCard
                    label="Bad Fit"
                    count={funnel.badFitCount}
                    icon={<ThumbsDown size={18} />}
                    color="#ffb4b4"
                  />
                  <SummaryCard
                    label="Fit Explainers"
                    count={funnel.fitExplanationRequestedCount}
                    icon={<Brain size={18} />}
                    color="#95a7ff"
                  />
                  <SummaryCard
                    label="Coach"
                    count={funnel.applicationCoachRequestedCount}
                    icon={<Brain size={18} />}
                    color="#c9fff8"
                  />
                  <SummaryCard
                    label="Cover Letters"
                    count={funnel.coverLetterDraftRequestedCount}
                    icon={<Layers size={18} />}
                    color="#ffd6a5"
                  />
                  <SummaryCard
                    label="Interview Prep"
                    count={funnel.interviewPrepRequestedCount}
                    icon={<Zap size={18} />}
                    color="#b9fbc0"
                  />
                </div>
              </Section>
            )}

            <Section title="Jobs by Source" icon={<BarChart2 size={16} />}>
              <SourceDistribution summary={summary} />
            </Section>

            <Section title="Jobs by Lifecycle" icon={<Zap size={16} />}>
              <LifecycleDistribution summary={summary} />
            </Section>
          </div>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))',
              gap: 16,
            }}
          >
            {funnel && (
              <Section title="Funnel by Source" icon={<BarChart2 size={16} />}>
                <FunnelSourceBreakdown summary={funnel} />
              </Section>
            )}

            <Section title="Top Matched Roles" icon={<Building2 size={16} />}>
              <TagList items={summary.topMatchedRoles} color="#95a7ff" />
            </Section>

            <Section title="Top Matched Skills" icon={<Zap size={16} />}>
              <TagList items={summary.topMatchedSkills} color="#c9fff8" />
            </Section>

            <Section title="Top Keywords" icon={<Hash size={16} />}>
              <TagList items={summary.topMatchedKeywords} color="#ffd6a5" />
            </Section>
          </div>

          {llmCtx && (
            <Section title="LLM Context Preview" icon={<Brain size={16} />}>
              <div
                style={{
                  fontSize: 11,
                  color: 'var(--color-text-secondary)',
                  marginBottom: 14,
                  letterSpacing: '0.04em',
                  textTransform: 'uppercase',
                }}
              >
                Deterministic payload — ready for Python enrichment
              </div>
              <LlmContextPanel ctx={llmCtx} />
            </Section>
          )}

          {llmCtx && (
            <Section title="LLM Enrichment" icon={<Brain size={16} />}>
              {insightsLoading ? (
                <p className="emptyState" style={{ margin: 0 }}>Generating enrichment…</p>
              ) : insightsError ? (
                <p className="emptyState" style={{ margin: 0 }}>
                  {(insightsError as Error).message || 'ML enrichment is unavailable right now.'}
                </p>
              ) : profileInsights ? (
                <ProfileInsightsPanel insights={profileInsights} />
              ) : (
                <p className="emptyState" style={{ margin: 0 }}>No enrichment available yet.</p>
              )}
            </Section>
          )}
        </>
      ) : (
        <p className="emptyState">No analytics data available.</p>
      )}
    </div>
  );
}
