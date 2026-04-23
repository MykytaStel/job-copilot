from importlib import import_module

__all__ = [
    "ApplicationCoachPrompt",
    "ApplicationCoachProviderError",
    "ApplicationCoachRequest",
    "ApplicationCoachResponse",
    "MalformedApplicationCoachOutputError",
    "application_coach_schema",
    "build_application_coach_prompt",
    "http_error_from_application_coach_error",
    "parse_application_coach_output",
]

_EXPORTS = {
    "ApplicationCoachPrompt": (
        "app.enrichment.application_coach.contract",
        "ApplicationCoachPrompt",
    ),
    "ApplicationCoachProviderError": (
        "app.enrichment.application_coach.contract",
        "ApplicationCoachProviderError",
    ),
    "ApplicationCoachRequest": (
        "app.enrichment.application_coach.contract",
        "ApplicationCoachRequest",
    ),
    "ApplicationCoachResponse": (
        "app.enrichment.application_coach.contract",
        "ApplicationCoachResponse",
    ),
    "MalformedApplicationCoachOutputError": (
        "app.enrichment.application_coach.contract",
        "MalformedApplicationCoachOutputError",
    ),
    "application_coach_schema": (
        "app.enrichment.application_coach.contract",
        "application_coach_schema",
    ),
    "build_application_coach_prompt": (
        "app.enrichment.application_coach.prompt",
        "build_application_coach_prompt",
    ),
    "http_error_from_application_coach_error": (
        "app.enrichment.application_coach.parser",
        "http_error_from_application_coach_error",
    ),
    "parse_application_coach_output": (
        "app.enrichment.application_coach.parser",
        "parse_application_coach_output",
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
