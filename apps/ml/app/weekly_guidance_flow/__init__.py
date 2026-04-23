from importlib import import_module
from typing import TYPE_CHECKING, Any

__all__ = [
    "MalformedWeeklyGuidanceOutputError",
    "WeeklyGuidanceAnalyticsSummary",
    "WeeklyGuidanceBehaviorSignalCount",
    "WeeklyGuidanceBehaviorSummary",
    "WeeklyGuidanceFunnelConversionRates",
    "WeeklyGuidanceFunnelSourceCount",
    "WeeklyGuidanceFunnelSummary",
    "WeeklyGuidanceJobsByLifecycle",
    "WeeklyGuidanceJobsBySourceEntry",
    "WeeklyGuidanceLlmContext",
    "WeeklyGuidancePrompt",
    "WeeklyGuidanceProviderError",
    "WeeklyGuidanceRecentFeedbackSummary",
    "WeeklyGuidanceRecentSearchSummary",
    "WeeklyGuidanceRequest",
    "WeeklyGuidanceResponse",
    "build_weekly_guidance_prompt",
    "http_error_from_weekly_guidance_error",
    "parse_weekly_guidance_output",
    "weekly_guidance_schema",
]

_EXPORTS = {
    "MalformedWeeklyGuidanceOutputError": (
        "app.weekly_guidance_flow.errors",
        "MalformedWeeklyGuidanceOutputError",
    ),
    "WeeklyGuidanceAnalyticsSummary": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceAnalyticsSummary",
    ),
    "WeeklyGuidanceBehaviorSignalCount": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceBehaviorSignalCount",
    ),
    "WeeklyGuidanceBehaviorSummary": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceBehaviorSummary",
    ),
    "WeeklyGuidanceFunnelConversionRates": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceFunnelConversionRates",
    ),
    "WeeklyGuidanceFunnelSourceCount": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceFunnelSourceCount",
    ),
    "WeeklyGuidanceFunnelSummary": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceFunnelSummary",
    ),
    "WeeklyGuidanceJobsByLifecycle": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceJobsByLifecycle",
    ),
    "WeeklyGuidanceJobsBySourceEntry": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceJobsBySourceEntry",
    ),
    "WeeklyGuidanceLlmContext": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceLlmContext",
    ),
    "WeeklyGuidancePrompt": (
        "app.weekly_guidance_flow.prompt",
        "WeeklyGuidancePrompt",
    ),
    "WeeklyGuidanceProviderError": (
        "app.weekly_guidance_flow.errors",
        "WeeklyGuidanceProviderError",
    ),
    "WeeklyGuidanceRecentFeedbackSummary": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceRecentFeedbackSummary",
    ),
    "WeeklyGuidanceRecentSearchSummary": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceRecentSearchSummary",
    ),
    "WeeklyGuidanceRequest": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceRequest",
    ),
    "WeeklyGuidanceResponse": (
        "app.weekly_guidance_flow.contract",
        "WeeklyGuidanceResponse",
    ),
    "build_weekly_guidance_prompt": (
        "app.weekly_guidance_flow.prompt",
        "build_weekly_guidance_prompt",
    ),
    "http_error_from_weekly_guidance_error": (
        "app.weekly_guidance_flow.errors",
        "http_error_from_weekly_guidance_error",
    ),
    "parse_weekly_guidance_output": (
        "app.weekly_guidance_flow.parser",
        "parse_weekly_guidance_output",
    ),
    "weekly_guidance_schema": (
        "app.weekly_guidance_flow.prompt",
        "weekly_guidance_schema",
    ),
}

if TYPE_CHECKING:
    from app.weekly_guidance_flow.contract import (
        WeeklyGuidanceAnalyticsSummary,
        WeeklyGuidanceBehaviorSignalCount,
        WeeklyGuidanceBehaviorSummary,
        WeeklyGuidanceFunnelConversionRates,
        WeeklyGuidanceFunnelSourceCount,
        WeeklyGuidanceFunnelSummary,
        WeeklyGuidanceJobsByLifecycle,
        WeeklyGuidanceJobsBySourceEntry,
        WeeklyGuidanceLlmContext,
        WeeklyGuidanceRecentFeedbackSummary,
        WeeklyGuidanceRecentSearchSummary,
        WeeklyGuidanceRequest,
        WeeklyGuidanceResponse,
    )
    from app.weekly_guidance_flow.errors import (
        MalformedWeeklyGuidanceOutputError,
        WeeklyGuidanceProviderError,
        http_error_from_weekly_guidance_error,
    )
    from app.weekly_guidance_flow.parser import parse_weekly_guidance_output
    from app.weekly_guidance_flow.prompt import (
        WeeklyGuidancePrompt,
        build_weekly_guidance_prompt,
        weekly_guidance_schema,
    )


def __getattr__(name: str) -> Any:
    export = _EXPORTS.get(name)
    if export is None:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

    module_name, attribute_name = export
    value = getattr(import_module(module_name), attribute_name)
    globals()[name] = value
    return value


def __dir__() -> list[str]:
    return sorted(set(globals()) | set(__all__))
