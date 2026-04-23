from importlib import import_module

__all__ = [
    "CurrentJobFeedbackState",
    "DeterministicFitContext",
    "FeedbackStateContext",
    "JobFitExplanationPrompt",
    "JobFitExplanationProviderError",
    "JobFitExplanationRequest",
    "JobFitExplanationResponse",
    "MalformedJobFitExplanationOutputError",
    "RankedJobContext",
    "SearchProfileContext",
    "SearchProfileRoleCandidate",
    "build_job_fit_explanation_prompt",
    "http_error_from_job_fit_explanation_error",
    "job_fit_explanation_schema",
    "parse_job_fit_explanation_output",
]

_EXPORTS = {
    "CurrentJobFeedbackState": (
        "app.enrichment.job_fit_explanation.contract",
        "CurrentJobFeedbackState",
    ),
    "DeterministicFitContext": (
        "app.enrichment.job_fit_explanation.contract",
        "DeterministicFitContext",
    ),
    "FeedbackStateContext": (
        "app.enrichment.job_fit_explanation.contract",
        "FeedbackStateContext",
    ),
    "JobFitExplanationPrompt": (
        "app.enrichment.job_fit_explanation.contract",
        "JobFitExplanationPrompt",
    ),
    "JobFitExplanationProviderError": (
        "app.enrichment.job_fit_explanation.contract",
        "JobFitExplanationProviderError",
    ),
    "JobFitExplanationRequest": (
        "app.enrichment.job_fit_explanation.contract",
        "JobFitExplanationRequest",
    ),
    "JobFitExplanationResponse": (
        "app.enrichment.job_fit_explanation.contract",
        "JobFitExplanationResponse",
    ),
    "MalformedJobFitExplanationOutputError": (
        "app.enrichment.job_fit_explanation.contract",
        "MalformedJobFitExplanationOutputError",
    ),
    "RankedJobContext": (
        "app.enrichment.job_fit_explanation.contract",
        "RankedJobContext",
    ),
    "SearchProfileContext": (
        "app.enrichment.job_fit_explanation.contract",
        "SearchProfileContext",
    ),
    "SearchProfileRoleCandidate": (
        "app.enrichment.job_fit_explanation.contract",
        "SearchProfileRoleCandidate",
    ),
    "build_job_fit_explanation_prompt": (
        "app.enrichment.job_fit_explanation.prompt",
        "build_job_fit_explanation_prompt",
    ),
    "http_error_from_job_fit_explanation_error": (
        "app.enrichment.job_fit_explanation.parser",
        "http_error_from_job_fit_explanation_error",
    ),
    "job_fit_explanation_schema": (
        "app.enrichment.job_fit_explanation.contract",
        "job_fit_explanation_schema",
    ),
    "parse_job_fit_explanation_output": (
        "app.enrichment.job_fit_explanation.parser",
        "parse_job_fit_explanation_output",
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
