import logging
from functools import lru_cache
from pathlib import Path
from typing import Annotated, Any

from pydantic import Field, field_validator
from pydantic_settings import BaseSettings, NoDecode, SettingsConfigDict


DEFAULT_CORS_ALLOWED_ORIGINS = (
    "http://localhost:3000",
    "http://127.0.0.1:3000",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
)
DEFAULT_ENGINE_API_BASE_URL = "http://localhost:8080"
DEFAULT_ENGINE_API_TIMEOUT_SECONDS = 10.0
DEFAULT_LLM_PROVIDER = "template"
DEFAULT_OPENAI_MODEL = "gpt-4o-mini"
DEFAULT_OLLAMA_BASE_URL = "http://localhost:11434"
DEFAULT_OLLAMA_MODEL = "llama3.1:8b"
DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS = 90.0
DEFAULT_TASK_TTL_HOURS = 24
DEFAULT_READY_TIMEOUT_SECONDS = 2.0
DEFAULT_BOOTSTRAP_MAX_CONCURRENT_JOBS = 2


def _ml_root_dir() -> Path:
    return Path(__file__).resolve().parent.parent


DEFAULT_ARTIFACTS_DIR = _ml_root_dir() / "models" / "profiles"
DEFAULT_BOOTSTRAP_TASKS_DIR = _ml_root_dir() / ".runtime" / "bootstrap-tasks"


def _split_csv(value: str) -> tuple[str, ...]:
    return tuple(item for item in (part.strip() for part in value.split(",")) if item)


class RuntimeSettings(BaseSettings):
    model_config = SettingsConfigDict(extra="ignore")

    log_level: str = Field(default="INFO", validation_alias="ML_LOG_LEVEL")
    log_format: str = Field(default="json", validation_alias="ML_LOG_FORMAT")
    cors_allowed_origins: Annotated[tuple[str, ...], NoDecode] = Field(
        default=DEFAULT_CORS_ALLOWED_ORIGINS,
        validation_alias="ML_CORS_ALLOWED_ORIGINS",
    )
    engine_api_base_url: str = Field(
        default=DEFAULT_ENGINE_API_BASE_URL,
        validation_alias="ENGINE_API_BASE_URL",
    )
    engine_api_timeout_seconds: float = Field(
        default=DEFAULT_ENGINE_API_TIMEOUT_SECONDS,
        validation_alias="ENGINE_API_TIMEOUT_SECONDS",
    )
    llm_provider: str = Field(
        default=DEFAULT_LLM_PROVIDER,
        validation_alias="ML_LLM_PROVIDER",
    )
    openai_api_key: str | None = Field(default=None, validation_alias="OPENAI_API_KEY")
    openai_model: str = Field(default=DEFAULT_OPENAI_MODEL, validation_alias="OPENAI_MODEL")
    openai_base_url: str | None = Field(default=None, validation_alias="OPENAI_BASE_URL")
    ollama_base_url: str = Field(
        default=DEFAULT_OLLAMA_BASE_URL,
        validation_alias="OLLAMA_BASE_URL",
    )
    ollama_model: str = Field(default=DEFAULT_OLLAMA_MODEL, validation_alias="OLLAMA_MODEL")
    internal_token: str | None = Field(default=None, validation_alias="ML_INTERNAL_TOKEN")
    engine_api_internal_token: str | None = Field(
        default=None,
        validation_alias="ENGINE_API_INTERNAL_TOKEN",
    )
    model_path: str | None = Field(default=None, validation_alias="ML_MODEL_PATH")
    artifacts_dir: str | None = Field(default=None, validation_alias="ML_ARTIFACTS_DIR")
    bootstrap_tasks_dir: str | None = Field(
        default=None,
        validation_alias="ML_BOOTSTRAP_TASKS_DIR",
    )
    task_ttl_hours: int = Field(default=DEFAULT_TASK_TTL_HOURS, validation_alias="ML_TASK_TTL_HOURS")
    bootstrap_max_concurrent_jobs: int = Field(
        default=DEFAULT_BOOTSTRAP_MAX_CONCURRENT_JOBS,
        validation_alias="ML_BOOTSTRAP_MAX_CONCURRENT_JOBS",
    )
    ready_timeout_seconds: float = Field(
        default=DEFAULT_READY_TIMEOUT_SECONDS,
        validation_alias="ML_READY_TIMEOUT_SECONDS",
    )
    llm_request_timeout_seconds: float = Field(
        default=DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS,
        validation_alias="ML_LLM_REQUEST_TIMEOUT_SECONDS",
    )

    @field_validator("log_level", mode="before")
    @classmethod
    def normalize_log_level(cls, value: Any) -> str:
        if not isinstance(value, str):
            return "INFO"
        return value.strip().upper() or "INFO"

    @field_validator("log_format", mode="before")
    @classmethod
    def normalize_log_format(cls, value: Any) -> str:
        if not isinstance(value, str):
            return "json"
        cleaned = value.strip().lower()
        return cleaned if cleaned in {"plain", "json"} else "json"

    @field_validator("cors_allowed_origins", mode="before")
    @classmethod
    def normalize_cors_allowed_origins(cls, value: Any) -> tuple[str, ...]:
        if value is None:
            return DEFAULT_CORS_ALLOWED_ORIGINS
        if isinstance(value, tuple):
            return tuple(item for item in value if item)
        if isinstance(value, list):
            return tuple(str(item).strip() for item in value if str(item).strip())
        if not isinstance(value, str):
            return DEFAULT_CORS_ALLOWED_ORIGINS

        cleaned = value.strip()
        if not cleaned:
            return DEFAULT_CORS_ALLOWED_ORIGINS
        if cleaned == "*":
            return ("*",)
        return _split_csv(cleaned)

    @field_validator("engine_api_base_url", "ollama_base_url", mode="before")
    @classmethod
    def normalize_base_url(cls, value: Any) -> str:
        if not isinstance(value, str):
            return ""
        return value.strip().rstrip("/")

    @field_validator("engine_api_timeout_seconds", mode="before")
    @classmethod
    def normalize_engine_api_timeout(cls, value: Any) -> float:
        try:
            return max(1.0, float(value))
        except (TypeError, ValueError):
            return DEFAULT_ENGINE_API_TIMEOUT_SECONDS

    @field_validator("ready_timeout_seconds", mode="before")
    @classmethod
    def normalize_ready_timeout(cls, value: Any) -> float:
        try:
            return max(0.2, float(value))
        except (TypeError, ValueError):
            return DEFAULT_READY_TIMEOUT_SECONDS

    @field_validator("task_ttl_hours", mode="before")
    @classmethod
    def normalize_task_ttl_hours(cls, value: Any) -> int:
        try:
            return max(1, int(value))
        except (TypeError, ValueError):
            return DEFAULT_TASK_TTL_HOURS

    @field_validator("bootstrap_max_concurrent_jobs", mode="before")
    @classmethod
    def normalize_bootstrap_max_concurrent_jobs(cls, value: Any) -> int:
        try:
            return max(1, int(value))
        except (TypeError, ValueError):
            return DEFAULT_BOOTSTRAP_MAX_CONCURRENT_JOBS

    @field_validator("llm_provider", mode="before")
    @classmethod
    def normalize_llm_provider(cls, value: Any) -> str:
        if not isinstance(value, str):
            return DEFAULT_LLM_PROVIDER
        return value.strip().lower() or DEFAULT_LLM_PROVIDER

    @field_validator(
        "openai_api_key",
        "openai_base_url",
        "internal_token",
        "engine_api_internal_token",
        "model_path",
        "artifacts_dir",
        "bootstrap_tasks_dir",
        mode="before",
    )
    @classmethod
    def normalize_optional_string(cls, value: Any) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str):
            return None
        cleaned = value.strip()
        return cleaned or None

    @field_validator("openai_model", "ollama_model", mode="before")
    @classmethod
    def normalize_required_string(cls, value: Any, info) -> str:
        default = DEFAULT_OPENAI_MODEL if info.field_name == "openai_model" else DEFAULT_OLLAMA_MODEL
        if not isinstance(value, str):
            return default
        cleaned = value.strip()
        return cleaned or default


@lru_cache(maxsize=1)
def get_runtime_settings() -> RuntimeSettings:
    return RuntimeSettings()


def configure_logging() -> None:
    settings = get_runtime_settings()
    level = getattr(logging, settings.log_level, logging.INFO)
    handlers: dict[str, Any] = {"level": level}
    if settings.log_format == "json":
        try:
            from pythonjsonlogger.json import JsonFormatter

            handler = logging.StreamHandler()
            handler.setFormatter(
                JsonFormatter("%(asctime)s %(levelname)s %(name)s %(message)s")
            )
            handlers["handlers"] = [handler]
        except ImportError:
            handlers["format"] = "%(asctime)s %(levelname)s %(name)s %(message)s"
    else:
        handlers["format"] = "%(asctime)s %(levelname)s %(name)s %(message)s"
    logging.basicConfig(**handlers)
