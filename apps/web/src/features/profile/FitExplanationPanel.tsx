import type { JobFitExplanation } from '../../api/enrichment';

import { FitExplanationList, InsightPanel, PanelTextBlock } from './fitPanels.shared';

export function FitExplanationPanel({ explanation }: { explanation: JobFitExplanation }) {
  return (
    <InsightPanel toneClassName="bg-surface-accent">
      <p className="m-0 text-sm leading-relaxed text-content">
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
        <PanelTextBlock
          label="Recommended next step"
          text={explanation.recommendedNextStep}
          emptyLabel="No next step returned."
        />
        <PanelTextBlock
          label="Application angle"
          text={explanation.applicationAngle}
          emptyLabel="No application angle returned."
        />
      </div>
    </InsightPanel>
  );
}
