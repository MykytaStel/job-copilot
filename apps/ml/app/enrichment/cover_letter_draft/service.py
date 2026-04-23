from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.cover_letter_draft.contract import (
    CoverLetterDraftRequest,
    CoverLetterDraftResponse,
)
from app.enrichment.cover_letter_draft.parser import parse_cover_letter_draft_output
from app.enrichment.cover_letter_draft.prompt import build_cover_letter_draft_prompt
from app.llm_provider_types import CoverLetterDraftProvider


class CoverLetterDraftService:
    def __init__(self, provider: CoverLetterDraftProvider):
        self._provider = provider

    async def enrich(self, context: CoverLetterDraftRequest) -> CoverLetterDraftResponse:
        prompt = build_cover_letter_draft_prompt(context)
        return await run_enrichment_call(
            flow="cover_letter_draft",
            provider=self._provider,
            context=context,
            prompt=prompt,
            provider_call=lambda: self._provider.generate_cover_letter_draft(context, prompt),
            parse_output=parse_cover_letter_draft_output,
        )
