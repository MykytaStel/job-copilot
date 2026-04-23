import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import ValidationError

from app.enrichment.profile_insights.contract import (
    MalformedProviderOutputError,
    ProfileInsightsProviderError,
    ProfileInsightsResponse,
)


def parse_profile_insights_output(raw_output: Any) -> ProfileInsightsResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedProviderOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedProviderOutputError("provider returned a non-object response")

    try:
        return ProfileInsightsResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedProviderOutputError("provider returned invalid profile insights") from exc


def http_error_from_provider_error(error: ProfileInsightsProviderError) -> HTTPException:
    if isinstance(error, MalformedProviderOutputError):
        detail = "Profile insights provider returned malformed output."
        status_code = status.HTTP_502_BAD_GATEWAY
    else:
        detail = str(error) or "Profile insights provider failed."
        status_code = status.HTTP_503_SERVICE_UNAVAILABLE

    return HTTPException(status_code=status_code, detail=detail)
