import json
from typing import Any

from pydantic import ValidationError

from app.weekly_guidance_flow.contract import WeeklyGuidanceResponse
from app.weekly_guidance_flow.errors import MalformedWeeklyGuidanceOutputError


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
