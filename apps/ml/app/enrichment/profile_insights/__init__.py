from importlib import import_module

__all__ = [
    "LIST_PREFIX_RE",
    "MAX_LIST_ITEMS",
    "WHITESPACE_RE",
    "LlmContextAnalyzedProfile",
    "LlmContextEvidenceEntry",
    "LlmContextFeedbackSummary",
    "LlmContextJobsFeedSummary",
    "LlmContextRequest",
    "MalformedProviderOutputError",
    "ProfileInsightsPrompt",
    "ProfileInsightsProviderError",
    "ProfileInsightsResponse",
    "build_profile_insights_prompt",
    "http_error_from_provider_error",
    "parse_profile_insights_output",
    "profile_insights_schema",
    "sanitize_text",
]

_EXPORTS = {
    "LIST_PREFIX_RE": ("app.enrichment.profile_insights.contract", "LIST_PREFIX_RE"),
    "MAX_LIST_ITEMS": ("app.enrichment.profile_insights.contract", "MAX_LIST_ITEMS"),
    "WHITESPACE_RE": ("app.enrichment.profile_insights.contract", "WHITESPACE_RE"),
    "LlmContextAnalyzedProfile": (
        "app.enrichment.profile_insights.contract",
        "LlmContextAnalyzedProfile",
    ),
    "LlmContextEvidenceEntry": (
        "app.enrichment.profile_insights.contract",
        "LlmContextEvidenceEntry",
    ),
    "LlmContextFeedbackSummary": (
        "app.enrichment.profile_insights.contract",
        "LlmContextFeedbackSummary",
    ),
    "LlmContextJobsFeedSummary": (
        "app.enrichment.profile_insights.contract",
        "LlmContextJobsFeedSummary",
    ),
    "LlmContextRequest": (
        "app.enrichment.profile_insights.contract",
        "LlmContextRequest",
    ),
    "MalformedProviderOutputError": (
        "app.enrichment.profile_insights.contract",
        "MalformedProviderOutputError",
    ),
    "ProfileInsightsPrompt": (
        "app.enrichment.profile_insights.contract",
        "ProfileInsightsPrompt",
    ),
    "ProfileInsightsProviderError": (
        "app.enrichment.profile_insights.contract",
        "ProfileInsightsProviderError",
    ),
    "ProfileInsightsResponse": (
        "app.enrichment.profile_insights.contract",
        "ProfileInsightsResponse",
    ),
    "build_profile_insights_prompt": (
        "app.enrichment.profile_insights.prompt",
        "build_profile_insights_prompt",
    ),
    "http_error_from_provider_error": (
        "app.enrichment.profile_insights.parser",
        "http_error_from_provider_error",
    ),
    "parse_profile_insights_output": (
        "app.enrichment.profile_insights.parser",
        "parse_profile_insights_output",
    ),
    "profile_insights_schema": (
        "app.enrichment.profile_insights.contract",
        "profile_insights_schema",
    ),
    "sanitize_text": ("app.enrichment.profile_insights.contract", "sanitize_text"),
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
