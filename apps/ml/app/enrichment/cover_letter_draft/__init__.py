from importlib import import_module

__all__ = [
    "MAX_BODY_PARAGRAPHS",
    "MAX_LIST_TEXT_LENGTH",
    "MAX_PARAGRAPH_LENGTH",
    "MAX_SUMMARY_LENGTH",
    "CoverLetterDraftPrompt",
    "CoverLetterDraftProviderError",
    "CoverLetterDraftRequest",
    "CoverLetterDraftResponse",
    "MalformedCoverLetterDraftOutputError",
    "build_cover_letter_draft_prompt",
    "cover_letter_draft_schema",
    "http_error_from_cover_letter_draft_error",
    "parse_cover_letter_draft_output",
]

_EXPORTS = {
    "MAX_BODY_PARAGRAPHS": (
        "app.enrichment.cover_letter_draft.contract",
        "MAX_BODY_PARAGRAPHS",
    ),
    "MAX_LIST_TEXT_LENGTH": (
        "app.enrichment.cover_letter_draft.contract",
        "MAX_LIST_TEXT_LENGTH",
    ),
    "MAX_PARAGRAPH_LENGTH": (
        "app.enrichment.cover_letter_draft.contract",
        "MAX_PARAGRAPH_LENGTH",
    ),
    "MAX_SUMMARY_LENGTH": (
        "app.enrichment.cover_letter_draft.contract",
        "MAX_SUMMARY_LENGTH",
    ),
    "CoverLetterDraftPrompt": (
        "app.enrichment.cover_letter_draft.contract",
        "CoverLetterDraftPrompt",
    ),
    "CoverLetterDraftProviderError": (
        "app.enrichment.cover_letter_draft.contract",
        "CoverLetterDraftProviderError",
    ),
    "CoverLetterDraftRequest": (
        "app.enrichment.cover_letter_draft.contract",
        "CoverLetterDraftRequest",
    ),
    "CoverLetterDraftResponse": (
        "app.enrichment.cover_letter_draft.contract",
        "CoverLetterDraftResponse",
    ),
    "MalformedCoverLetterDraftOutputError": (
        "app.enrichment.cover_letter_draft.contract",
        "MalformedCoverLetterDraftOutputError",
    ),
    "build_cover_letter_draft_prompt": (
        "app.enrichment.cover_letter_draft.prompt",
        "build_cover_letter_draft_prompt",
    ),
    "cover_letter_draft_schema": (
        "app.enrichment.cover_letter_draft.contract",
        "cover_letter_draft_schema",
    ),
    "http_error_from_cover_letter_draft_error": (
        "app.enrichment.cover_letter_draft.parser",
        "http_error_from_cover_letter_draft_error",
    ),
    "parse_cover_letter_draft_output": (
        "app.enrichment.cover_letter_draft.parser",
        "parse_cover_letter_draft_output",
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
