from importlib import import_module

__all__ = [
    "CvTailoringGapItem",
    "CvTailoringSuggestions",
    "CvTailoringRequest",
    "CvTailoringResponse",
    "CvTailoringPrompt",
    "CvTailoringProviderError",
    "MalformedCvTailoringOutputError",
    "build_cv_tailoring_prompt",
    "cv_tailoring_suggestions_schema",
    "http_error_from_cv_tailoring_error",
    "parse_cv_tailoring_suggestions",
]

_EXPORTS = {
    "CvTailoringGapItem": ("app.enrichment.cv_tailoring.contract", "CvTailoringGapItem"),
    "CvTailoringSuggestions": ("app.enrichment.cv_tailoring.contract", "CvTailoringSuggestions"),
    "CvTailoringRequest": ("app.enrichment.cv_tailoring.contract", "CvTailoringRequest"),
    "CvTailoringResponse": ("app.enrichment.cv_tailoring.contract", "CvTailoringResponse"),
    "CvTailoringPrompt": ("app.enrichment.cv_tailoring.contract", "CvTailoringPrompt"),
    "CvTailoringProviderError": ("app.enrichment.cv_tailoring.contract", "CvTailoringProviderError"),
    "MalformedCvTailoringOutputError": ("app.enrichment.cv_tailoring.contract", "MalformedCvTailoringOutputError"),
    "build_cv_tailoring_prompt": ("app.enrichment.cv_tailoring.prompt", "build_cv_tailoring_prompt"),
    "cv_tailoring_suggestions_schema": ("app.enrichment.cv_tailoring.contract", "cv_tailoring_suggestions_schema"),
    "http_error_from_cv_tailoring_error": ("app.enrichment.cv_tailoring.parser", "http_error_from_cv_tailoring_error"),
    "parse_cv_tailoring_suggestions": ("app.enrichment.cv_tailoring.parser", "parse_cv_tailoring_suggestions"),
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
