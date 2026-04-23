import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import ValidationError

from app.enrichment.job_fit_explanation.contract import (
    JobFitExplanationProviderError,
    JobFitExplanationResponse,
    MalformedJobFitExplanationOutputError,
)


def parse_job_fit_explanation_output(raw_output: Any) -> JobFitExplanationResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedJobFitExplanationOutputError(
                "provider returned non-JSON output"
            ) from exc

    if not isinstance(payload, dict):
        raise MalformedJobFitExplanationOutputError("provider returned a non-object response")

    try:
        return JobFitExplanationResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedJobFitExplanationOutputError(
            "provider returned invalid job fit explanation"
        ) from exc


def http_error_from_job_fit_explanation_error(
    error: JobFitExplanationProviderError,
) -> HTTPException:
    if isinstance(error, MalformedJobFitExplanationOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Job fit explanation provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Job fit explanation provider failed.",
    )
