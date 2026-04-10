import { useQuery } from '@tanstack/react-query';
import type { MarketInsights } from '../api';
import { getMarketInsights } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonPage } from '../components/Skeleton';

export default function Market() {
  const { data, isLoading, error } = useQuery<MarketInsights>({
    queryKey: queryKeys.market.insights(),
    queryFn: getMarketInsights,
  });

  if (isLoading) return <SkeletonPage />;
  if (error) return <p className="error">{error instanceof Error ? error.message : 'Error'}</p>;
  if (!data) return null;

  if (data.totalJobs === 0) {
    return (
      <div>
        <h1>Market Pulse</h1>
        <p className="muted">Add some jobs first — Pulse analyzes patterns across all your saved postings.</p>
      </div>
    );
  }

  const maxCount = data.topSkills[0]?.count ?? 1;

  return (
    <div>
      <h1>Market Pulse</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Based on <strong>{data.totalJobs}</strong> saved job{data.totalJobs !== 1 ? 's' : ''}
      </p>

      {/* Coverage score */}
      <div style={{ display: 'flex', gap: 16, marginBottom: 24, flexWrap: 'wrap' }}>
        <div className="card" style={{ minWidth: 160, textAlign: 'center', padding: '20px 24px' }}>
          <div style={{ fontSize: 48, fontWeight: 700, color: scoreColor(data.coverageScore) }}>
            {data.coverageScore}%
          </div>
          <div className="muted" style={{ fontSize: 13 }}>Top-10 skill coverage</div>
        </div>

        {data.hotGaps.length > 0 && (
          <div className="card" style={{ flex: 1, minWidth: 220 }}>
            <p className="eyebrow" style={{ marginBottom: 8 }}>Hot gaps — in 30%+ of your jobs</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
              {data.hotGaps.map((s) => (
                <span key={s} className="pill pill-missing">{s}</span>
              ))}
            </div>
          </div>
        )}

        {data.salaryMentions.length > 0 && (
          <div className="card" style={{ flex: 1, minWidth: 220 }}>
            <p className="eyebrow" style={{ marginBottom: 8 }}>Salary mentions</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
              {data.salaryMentions.map((s) => (
                <span key={s} style={{ fontSize: 13, padding: '2px 8px', background: 'var(--surface)', borderRadius: 4 }}>{s}</span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Skills bar chart */}
      <div className="card">
        <p className="eyebrow" style={{ marginBottom: 16 }}>Most demanded skills in your jobs</p>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          {data.topSkills.map((s) => (
            <div key={s.skill} style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <div style={{ width: 110, fontSize: 13, textAlign: 'right', flexShrink: 0 }}>
                {s.skill}
              </div>
              <div style={{ flex: 1, background: 'var(--surface)', borderRadius: 4, height: 20, position: 'relative', overflow: 'hidden' }}>
                <div style={{
                  width: `${(s.count / maxCount) * 100}%`,
                  height: '100%',
                  background: s.inResume ? 'var(--accent, #38a169)' : 'var(--danger, #e53e3e)',
                  borderRadius: 4,
                  transition: 'width 0.4s ease',
                }} />
              </div>
              <div style={{ width: 90, fontSize: 12, color: 'var(--muted)' }}>
                {s.count}/{data.totalJobs} jobs · {s.pct}%
              </div>
              <div style={{ width: 16, fontSize: 12 }}>
                {s.inResume ? '✓' : '✗'}
              </div>
            </div>
          ))}
        </div>
        <p className="muted" style={{ fontSize: 12, marginTop: 16 }}>
          <span style={{ color: 'var(--accent, #38a169)' }}>■</span> in your resume &nbsp;
          <span style={{ color: 'var(--danger, #e53e3e)' }}>■</span> missing
        </p>
      </div>
    </div>
  );
}

function scoreColor(score: number): string {
  if (score >= 70) return 'var(--accent, #38a169)';
  if (score >= 40) return '#d97706';
  return 'var(--danger, #e53e3e)';
}
