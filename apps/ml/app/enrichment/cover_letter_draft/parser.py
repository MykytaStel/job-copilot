import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import ValidationError

from app.enrichment.cover_letter_draft.contract import (
    CoverLetterDraftProviderError,
    CoverLetterDraftResponse,
    MalformedCoverLetterDraftOutputError,
)


def parse_cover_letter_draft_output(raw_output: Any) -> CoverLetterDraftResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedCoverLetterDraftOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedCoverLetterDraftOutputError("provider returned a non-object response")

    try:
        return CoverLetterDraftResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedCoverLetterDraftOutputError(
            "provider returned invalid cover letter draft"
        ) from exc


def http_error_from_cover_letter_draft_error(
    error: CoverLetterDraftProviderError,
) -> HTTPException:
    if isinstance(error, MalformedCoverLetterDraftOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Cover letter draft provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Cover letter draft provider failed.",
    )
