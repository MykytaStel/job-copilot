import type { ApplicationCoach, CoverLetterDraft, InterviewPrep, JobFitExplanation } from '../../api/enrichment';

export const insightPanelBaseClass = 'flex flex-col gap-3 rounded-[14px] border px-4 py-3.5';

export function FitExplanationList({
  label,
  items,
  emptyLabel,
}: {
  label: string;
  items: string[];
  emptyLabel: string;
}) {
  return (
    <div>
      <span className="detailLabel">{label}</span>
      {items.length > 0 ? (
        <ul className="textList fitReasonsList" style={{ marginTop: 8 }}>
          {items.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      ) : (
        <p className="m-0 text-sm leading-6 text-muted-foreground">{emptyLabel}</p>
      )}
    </div>
  );
}

export function FitExplanationPanel({ explanation }: { explanation: JobFitExplanation }) {
  return (
    <div className={`${insightPanelBaseClass} border-edge-accent bg-surface-accent`}>
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
        <div>
          <span className="detailLabel">Recommended next step</span>
          <p className="sectionText" style={{ marginBottom: 0 }}>
            {explanation.recommendedNextStep || 'No next step returned.'}
          </p>
        </div>
        <div>
          <span className="detailLabel">Application angle</span>
          <p className="sectionText" style={{ marginBottom: 0 }}>
            {explanation.applicationAngle || 'No application angle returned.'}
          </p>
        </div>
      </div>
    </div>
  );
}

export function ApplicationCoachPanel({ coaching }: { coaching: ApplicationCoach }) {
  return (
    <div className={`${insightPanelBaseClass} border-edge-accent bg-surface-accent`}>
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
    </div>
  );
}

export function CoverLetterDraftPanel({ draft }: { draft: CoverLetterDraft }) {
  return (
    <div className={`${insightPanelBaseClass} border-edge-accent bg-surface-warning`}>
      <div>
        <span className="detailLabel">Draft summary</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {draft.draftSummary || 'No draft summary returned.'}
        </p>
      </div>

      <div>
        <span className="detailLabel">Opening paragraph</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {draft.openingParagraph || 'No opening paragraph returned.'}
        </p>
      </div>

      <div>
        <span className="detailLabel">Body paragraphs</span>
        {draft.bodyParagraphs.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10, marginTop: 8 }}>
            {draft.bodyParagraphs.map((paragraph, index) => (
              <p key={`${index}-${paragraph}`} className="sectionText" style={{ marginBottom: 0 }}>
                {paragraph}
              </p>
            ))}
          </div>
        ) : (
          <p className="m-0 text-sm leading-6 text-muted-foreground">
            No body paragraphs returned.
          </p>
        )}
      </div>

      <div>
        <span className="detailLabel">Closing paragraph</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {draft.closingParagraph || 'No closing paragraph returned.'}
        </p>
      </div>

      <FitExplanationList
        label="Key claims used"
        items={draft.keyClaimsUsed}
        emptyLabel="No grounded claims returned."
      />
      <FitExplanationList
        label="Evidence gaps"
        items={draft.evidenceGaps}
        emptyLabel="No evidence gaps returned."
      />
      <FitExplanationList
        label="Tone notes"
        items={draft.toneNotes}
        emptyLabel="No tone notes returned."
      />
    </div>
  );
}

export function InterviewPrepPanel({ prep }: { prep: InterviewPrep }) {
  return (
    <div className={`${insightPanelBaseClass} border-edge-accent bg-surface-success`}>
      <div>
        <span className="detailLabel">Prep summary</span>
        <p className="sectionText" style={{ marginBottom: 0 }}>
          {prep.prepSummary || 'No prep summary returned.'}
        </p>
      </div>

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
    </div>
  );
}
