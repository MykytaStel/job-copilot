import json
from typing import Any

from pydantic import ValidationError

from app.enrichment.weekly_guidance.contract import WeeklyGuidanceResponse
from app.enrichment.weekly_guidance.errors import MalformedWeeklyGuidanceOutputError


def parse_weekly_guidance_output(raw_output: Any) -> WeeklyGuidanceResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedWeeklyGuidanceOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedWeeklyGuidanceOutputError("provider returned a non-object response")

    try:
        return WeeklyGuidanceResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedWeeklyGuidanceOutputError("provider returned invalid weekly guidance") from exc
