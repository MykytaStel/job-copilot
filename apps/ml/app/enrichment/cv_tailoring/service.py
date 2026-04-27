from __future__ import annotations

import logging
from datetime import datetime, timezone
from typing import Any

from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.cv_tailoring.contract import (
    CvTailoringRequest,
    CvTailoringResponse,
    MalformedCvTailoringOutputError,
)
from app.enrichment.cv_tailoring.parser import parse_cv_tailoring_suggestions
from app.enrichment.cv_tailoring.prompt import build_cv_tailoring_prompt
from app.llm_provider_types import CvTailoringProvider
from app.template_cv_tailoring import build_cv_tailoring

logger = logging.getLogger(__name__)


def _provider_label(provider: Any) -> str:
    name = type(provider).__name__

    if "Template" in name:
        return "template"

    if "OpenAI" in name or "Openai" in name:
        return "openai"

    if "Ollama" in name:
        return "ollama"

    return "unknown"


class CvTailoringService:
    def __init__(self, provider: CvTailoringProvider) -> None:
        self._provider = provider

    async def enrich(self, context: CvTailoringRequest) -> CvTailoringResponse:
        prompt = build_cv_tailoring_prompt(context)
        provider = _provider_label(self._provider)

        try:
            suggestions = await run_enrichment_call(
                flow="cv_tailoring",
                provider=self._provider,
                context=context,
                prompt=prompt,
                provider_call=lambda: self._provider.generate_cv_tailoring(
                    context,
                    prompt,
                ),
                parse_output=parse_cv_tailoring_suggestions,
            )
        except MalformedCvTailoringOutputError:
            logger.warning(
                "cv tailoring provider returned malformed output; falling back to template suggestions",
                extra={
                    "flow": "cv_tailoring",
                    "provider": provider,
                    "profile_id": context.profile_id,
                    "job_id": context.job_id,
                },
            )

            suggestions = parse_cv_tailoring_suggestions(build_cv_tailoring(context))
            provider = f"{provider}_fallback_template"

        return CvTailoringResponse(
            suggestions=suggestions,
            provider=provider,
            generated_at=datetime.now(timezone.utc).isoformat(),
        )