import logging
import os
from dataclasses import dataclass
from functools import lru_cache


DEFAULT_CORS_ALLOWED_ORIGINS = (
    "http://localhost:3000",
    "http://127.0.0.1:3000",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
)


def _split_csv(value: str) -> tuple[str, ...]:
    return tuple(item for item in (part.strip() for part in value.split(",")) if item)


def _cors_allowed_origins(raw_value: str | None) -> tuple[str, ...]:
    if raw_value is None:
        return DEFAULT_CORS_ALLOWED_ORIGINS

    cleaned = raw_value.strip()
    if not cleaned:
        return DEFAULT_CORS_ALLOWED_ORIGINS
    if cleaned == "*":
        return ("*",)
    return _split_csv(cleaned)


@dataclass(frozen=True)
class RuntimeSettings:
    log_level: str
    cors_allowed_origins: tuple[str, ...]


@lru_cache(maxsize=1)
def get_runtime_settings() -> RuntimeSettings:
    return RuntimeSettings(
        log_level=os.getenv("ML_LOG_LEVEL", "INFO").strip().upper() or "INFO",
        cors_allowed_origins=_cors_allowed_origins(
            os.getenv("ML_CORS_ALLOWED_ORIGINS")
        ),
    )


def configure_logging() -> None:
    settings = get_runtime_settings()
    level = getattr(logging, settings.log_level, logging.INFO)
    logging.basicConfig(
        level=level,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )
