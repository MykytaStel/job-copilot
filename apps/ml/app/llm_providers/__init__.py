from importlib import import_module
from typing import TYPE_CHECKING, Any

__all__ = ["OpenAIEnrichmentProvider", "OllamaEnrichmentProvider"]

_REMOTE_PROVIDER_MODULES = {
    "OpenAIEnrichmentProvider": "app.llm_providers.openai_provider",
    "OllamaEnrichmentProvider": "app.llm_providers.ollama_provider",
}

if TYPE_CHECKING:
    from app.llm_providers.ollama_provider import OllamaEnrichmentProvider
    from app.llm_providers.openai_provider import OpenAIEnrichmentProvider


def __getattr__(name: str) -> Any:
    module_name = _REMOTE_PROVIDER_MODULES.get(name)
    if module_name is None:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

    exported = getattr(import_module(module_name), name)
    globals()[name] = exported
    return exported


def __dir__() -> list[str]:
    return sorted(set(globals()) | set(__all__))
