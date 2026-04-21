import type { CoverLetterDraft } from '../../api/enrichment';

import {
  FitExplanationList,
  InsightPanel,
  PanelTextBlock,
  ParagraphStack,
} from './fitPanels.shared';

export function CoverLetterDraftPanel({ draft }: { draft: CoverLetterDraft }) {
  return (
    <InsightPanel toneClassName="bg-surface-warning">
      <PanelTextBlock
        label="Draft summary"
        text={draft.draftSummary}
        emptyLabel="No draft summary returned."
      />

      <PanelTextBlock
        label="Opening paragraph"
        text={draft.openingParagraph}
        emptyLabel="No opening paragraph returned."
      />

      <div>
        <span className="detailLabel">Body paragraphs</span>
        <ParagraphStack items={draft.bodyParagraphs} emptyLabel="No body paragraphs returned." />
      </div>

      <PanelTextBlock
        label="Closing paragraph"
        text={draft.closingParagraph}
        emptyLabel="No closing paragraph returned."
      />

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
    </InsightPanel>
  );
}
