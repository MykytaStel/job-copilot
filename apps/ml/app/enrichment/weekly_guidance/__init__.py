from importlib import import_module

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
        "app.enrichment.weekly_guidance.errors",
        "MalformedWeeklyGuidanceOutputError",
    ),
    "WeeklyGuidanceAnalyticsSummary": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceAnalyticsSummary",
    ),
    "WeeklyGuidanceBehaviorSignalCount": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceBehaviorSignalCount",
    ),
    "WeeklyGuidanceBehaviorSummary": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceBehaviorSummary",
    ),
    "WeeklyGuidanceFunnelConversionRates": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceFunnelConversionRates",
    ),
    "WeeklyGuidanceFunnelSourceCount": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceFunnelSourceCount",
    ),
    "WeeklyGuidanceFunnelSummary": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceFunnelSummary",
    ),
    "WeeklyGuidanceJobsByLifecycle": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceJobsByLifecycle",
    ),
    "WeeklyGuidanceJobsBySourceEntry": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceJobsBySourceEntry",
    ),
    "WeeklyGuidanceLlmContext": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceLlmContext",
    ),
    "WeeklyGuidancePrompt": (
        "app.enrichment.weekly_guidance.prompt",
        "WeeklyGuidancePrompt",
    ),
    "WeeklyGuidanceProviderError": (
        "app.enrichment.weekly_guidance.errors",
        "WeeklyGuidanceProviderError",
    ),
    "WeeklyGuidanceRecentFeedbackSummary": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceRecentFeedbackSummary",
    ),
    "WeeklyGuidanceRecentSearchSummary": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceRecentSearchSummary",
    ),
    "WeeklyGuidanceRequest": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceRequest",
    ),
    "WeeklyGuidanceResponse": (
        "app.enrichment.weekly_guidance.contract",
        "WeeklyGuidanceResponse",
    ),
    "build_weekly_guidance_prompt": (
        "app.enrichment.weekly_guidance.prompt",
        "build_weekly_guidance_prompt",
    ),
    "http_error_from_weekly_guidance_error": (
        "app.enrichment.weekly_guidance.errors",
        "http_error_from_weekly_guidance_error",
    ),
    "parse_weekly_guidance_output": (
        "app.enrichment.weekly_guidance.parser",
        "parse_weekly_guidance_output",
    ),
    "weekly_guidance_schema": (
        "app.enrichment.weekly_guidance.prompt",
        "weekly_guidance_schema",
    ),
}


def __getattr__(name: str):
    if name not in _EXPORTS:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
    module_name, attribute_name = _EXPORTS[name]
    value = getattr(import_module(module_name), attribute_name)
    globals()[name] = value
    return value


def __dir__() -> list[str]:
    return sorted(list(globals()) + __all__)
