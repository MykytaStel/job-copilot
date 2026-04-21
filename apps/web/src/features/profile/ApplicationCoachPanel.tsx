import type { ApplicationCoach } from '../../api/enrichment';

import { FitExplanationList, InsightPanel } from './fitPanels.shared';

export function ApplicationCoachPanel({ coaching }: { coaching: ApplicationCoach }) {
  return (
    <InsightPanel toneClassName="bg-surface-accent">
      <p className="m-0 text-sm leading-relaxed text-content">
        {coaching.applicationSummary || 'No application summary returned.'}
      </p>

      <FitExplanationList
        label="Resume focus points"
        items={coaching.resumeFocusPoints}
        emptyLabel="No resume focus points returned."
      />
      <FitExplanationList
        label="Suggested bullets"
        items={coaching.suggestedBullets}
        emptyLabel="No suggested bullets returned."
      />
      <FitExplanationList
        label="Cover letter angles"
        items={coaching.coverLetterAngles}
        emptyLabel="No cover letter angles returned."
      />
      <FitExplanationList
        label="Interview focus"
        items={coaching.interviewFocus}
        emptyLabel="No interview focus returned."
      />
      <FitExplanationList
        label="Gaps to address"
        items={coaching.gapsToAddress}
        emptyLabel="No explicit gaps returned."
      />
      <FitExplanationList
        label="Red flags"
        items={coaching.redFlags}
        emptyLabel="No explicit red flags returned."
      />
    </InsightPanel>
  );
}
