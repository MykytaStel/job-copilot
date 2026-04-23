import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import ValidationError

from app.enrichment.interview_prep.contract import (
    InterviewPrepProviderError,
    InterviewPrepResponse,
    MalformedInterviewPrepOutputError,
)


def parse_interview_prep_output(raw_output: Any) -> InterviewPrepResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedInterviewPrepOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedInterviewPrepOutputError("provider returned a non-object response")

    try:
        return InterviewPrepResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedInterviewPrepOutputError(
            "provider returned invalid interview preparation"
        ) from exc


def http_error_from_interview_prep_error(error: InterviewPrepProviderError) -> HTTPException:
    if isinstance(error, MalformedInterviewPrepOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Interview prep provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Interview prep provider failed.",
    )
