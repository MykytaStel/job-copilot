import { useQuery } from '@tanstack/react-query';
import {
  BarChart2,
  Bookmark,
  Brain,
  Building2,
  EyeOff,
  Hash,
  Layers,
  ShieldCheck,
  ShieldOff,
  ThumbsDown,
  Zap,
} from 'lucide-react';

import { getAnalyticsSummary, getLlmContext } from '../api';
import type { AnalyticsSummary, LlmContext } from '../api';
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

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function Analytics() {
  const profileId = readProfileId();

  const { data: summary, isLoading: summaryLoading } = useQuery({
    queryKey: queryKeys.analytics.summary(profileId ?? ''),
    queryFn: () => getAnalyticsSummary(profileId!),
    enabled: !!profileId,
  });

  const { data: llmCtx, isLoading: ctxLoading } = useQuery({
    queryKey: queryKeys.analytics.llmContext(profileId ?? ''),
    queryFn: () => getLlmContext(profileId!),
    enabled: !!profileId,
  });

  if (!profileId) {
    return (
      <div className="jobDetails">
        <p className="emptyState">Create a profile to view analytics.</p>
      </div>
    );
  }

  const isLoading = summaryLoading || ctxLoading;

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
        </>
      ) : (
        <p className="emptyState">No analytics data available.</p>
      )}
    </div>
  );
}
