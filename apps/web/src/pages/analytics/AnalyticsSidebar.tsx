import { AlertTriangle, FileWarning, Sparkles, XCircle, TrendingDown } from 'lucide-react';
import type { AnalyticsSummary } from '../../api/analytics';
import type { AIInsight } from '../../components/ui/AIInsightPanel';

import { AIInsightPanel } from '../../components/ui/AIInsightPanel';
import { StatCard } from '../../components/ui/StatCard';

import { PillCloud, Section } from './AnalyticsHelpers';

export function AnalyticsSidebar({
  summary,
  aiInsights,
}: {
  summary: AnalyticsSummary;
  aiInsights: AIInsight[];
}) {
  return (
    <>
      <AIInsightPanel insights={aiInsights} title="AI Guidance" />
      <Section
        title="Match Surface"
        description="Top deterministic dimensions currently shaping ranked jobs."
        icon={Sparkles}
        eyebrow="Current signal"
      >
        <div className="space-y-4">
          <PillCloud
            title="Matched roles"
            items={summary.topMatchedRoles}
            emptyMessage="No matched roles yet."
            tone="primary"
          />
          <PillCloud
            title="Matched skills"
            items={summary.topMatchedSkills}
            emptyMessage="No matched skills yet."
            tone="success"
          />
          <PillCloud
            title="Matched keywords"
            items={summary.topMatchedKeywords}
            emptyMessage="No matched keywords yet."
            tone="warning"
          />
        </div>
      </Section>
      <Section
        title="Search Quality"
        description="Signals that explain why current matching can feel weak or noisy."
        icon={AlertTriangle}
        eyebrow="Diagnostics"
      >
        <div className="grid grid-cols-2 gap-3">
          <StatCard title="Low evidence" value={summary.searchQuality.lowEvidenceJobs} icon={AlertTriangle} />
          <StatCard title="Weak descriptions" value={summary.searchQuality.weakDescriptionJobs} icon={FileWarning} />
          <StatCard title="Role mismatch" value={summary.searchQuality.roleMismatchJobs} icon={XCircle} />
          <StatCard
            title="Seniority mismatch"
            value={summary.searchQuality.seniorityMismatchJobs}
            icon={TrendingDown}
          />
        </div>

        <div className="mt-4">
          <PillCloud
            title="Top missing signals"
            items={summary.searchQuality.topMissingSignals}
            emptyMessage="No repeated missing signals yet."
            tone="warning"
          />
        </div>
      </Section>
    </>
  );
}
