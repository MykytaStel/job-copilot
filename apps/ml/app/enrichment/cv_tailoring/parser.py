import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import ValidationError

from app.enrichment.cv_tailoring.contract import (
    CvTailoringProviderError,
    CvTailoringSuggestions,
    MalformedCvTailoringOutputError,
)


def parse_cv_tailoring_suggestions(raw_output: Any) -> CvTailoringSuggestions:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedCvTailoringOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedCvTailoringOutputError("provider returned a non-object response")

    try:
        return CvTailoringSuggestions.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedCvTailoringOutputError("provider returned invalid cv tailoring output") from exc


def http_error_from_cv_tailoring_error(error: CvTailoringProviderError) -> HTTPException:
    if isinstance(error, MalformedCvTailoringOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="CV tailoring provider returned malformed output.",
        )
    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "CV tailoring provider failed.",
    )
