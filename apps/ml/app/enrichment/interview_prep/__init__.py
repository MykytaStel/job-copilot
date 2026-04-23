from importlib import import_module

__all__ = [
    "MAX_LIST_TEXT_LENGTH",
    "MAX_SUMMARY_LENGTH",
    "InterviewPrepPrompt",
    "InterviewPrepProviderError",
    "InterviewPrepRequest",
    "InterviewPrepResponse",
    "MalformedInterviewPrepOutputError",
    "build_interview_prep_prompt",
    "http_error_from_interview_prep_error",
    "interview_prep_schema",
    "parse_interview_prep_output",
]

_EXPORTS = {
    "MAX_LIST_TEXT_LENGTH": (
        "app.enrichment.interview_prep.contract",
        "MAX_LIST_TEXT_LENGTH",
    ),
    "MAX_SUMMARY_LENGTH": (
        "app.enrichment.interview_prep.contract",
        "MAX_SUMMARY_LENGTH",
    ),
    "InterviewPrepPrompt": (
        "app.enrichment.interview_prep.contract",
        "InterviewPrepPrompt",
    ),
    "InterviewPrepProviderError": (
        "app.enrichment.interview_prep.contract",
        "InterviewPrepProviderError",
    ),
    "InterviewPrepRequest": (
        "app.enrichment.interview_prep.contract",
        "InterviewPrepRequest",
    ),
    "InterviewPrepResponse": (
        "app.enrichment.interview_prep.contract",
        "InterviewPrepResponse",
    ),
    "MalformedInterviewPrepOutputError": (
        "app.enrichment.interview_prep.contract",
        "MalformedInterviewPrepOutputError",
    ),
    "build_interview_prep_prompt": (
        "app.enrichment.interview_prep.prompt",
        "build_interview_prep_prompt",
    ),
    "http_error_from_interview_prep_error": (
        "app.enrichment.interview_prep.parser",
        "http_error_from_interview_prep_error",
    ),
    "interview_prep_schema": (
        "app.enrichment.interview_prep.contract",
        "interview_prep_schema",
    ),
    "parse_interview_prep_output": (
        "app.enrichment.interview_prep.parser",
        "parse_interview_prep_output",
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
