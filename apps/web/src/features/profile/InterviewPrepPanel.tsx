import type { InterviewPrep } from '../../api/enrichment';

import { FitExplanationList, InsightPanel, PanelTextBlock } from './fitPanels.shared';

export function InterviewPrepPanel({ prep }: { prep: InterviewPrep }) {
  return (
    <InsightPanel toneClassName="bg-surface-success">
      <PanelTextBlock
        label="Prep summary"
        text={prep.prepSummary}
        emptyLabel="No prep summary returned."
      />

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
    </InsightPanel>
  );
}
