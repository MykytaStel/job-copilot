from app.cover_letter_draft import (
    CoverLetterDraftRequest,
    CoverLetterDraftResponse,
    build_cover_letter_draft_prompt,
    parse_cover_letter_draft_output,
)
from app.llm_provider import CoverLetterDraftProvider


class CoverLetterDraftService:
    def __init__(self, provider: CoverLetterDraftProvider):
        self._provider = provider

    async def enrich(self, context: CoverLetterDraftRequest) -> CoverLetterDraftResponse:
        prompt = build_cover_letter_draft_prompt(context)
        raw_output = await self._provider.generate_cover_letter_draft(context, prompt)
        return parse_cover_letter_draft_output(raw_output)
