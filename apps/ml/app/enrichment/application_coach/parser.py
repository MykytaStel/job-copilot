import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import ValidationError

from app.enrichment.application_coach.contract import (
    ApplicationCoachProviderError,
    ApplicationCoachResponse,
    MalformedApplicationCoachOutputError,
)


def parse_application_coach_output(raw_output: Any) -> ApplicationCoachResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedApplicationCoachOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedApplicationCoachOutputError("provider returned a non-object response")

    try:
        return ApplicationCoachResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedApplicationCoachOutputError(
            "provider returned invalid application coaching"
        ) from exc


def http_error_from_application_coach_error(
    error: ApplicationCoachProviderError,
) -> HTTPException:
    if isinstance(error, MalformedApplicationCoachOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Application coach provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Application coach provider failed.",
    )
