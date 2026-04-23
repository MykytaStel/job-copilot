from __future__ import annotations

import json
import logging
from time import perf_counter
from typing import Any, Awaitable, Callable, TypeVar


logger = logging.getLogger(__name__)

TResult = TypeVar("TResult")


async def run_enrichment_call(
    *,
    flow: str,
    provider: Any,
    context: Any,
    prompt: Any,
    provider_call: Callable[[], Awaitable[Any]],
    parse_output: Callable[[Any], TResult],
) -> TResult:
    started_at = perf_counter()
    provider_name = provider.__class__.__name__
    profile_id = getattr(context, "profile_id", None)
    context_bytes = _safe_len(getattr(prompt, "context_payload", None))
    schema_bytes = _safe_len(getattr(prompt, "output_schema_expectations", None))

    try:
        raw_output = await provider_call()
        result = parse_output(raw_output)
    except Exception as exc:
        logger.warning(
            "enrichment call failed",
            extra={
                "flow": flow,
                "provider": provider_name,
                "profile_id": profile_id,
                "duration_ms": round((perf_counter() - started_at) * 1000, 2),
                "success": False,
                "error_type": type(exc).__name__,
                "context_bytes": context_bytes,
                "schema_bytes": schema_bytes,
            },
        )
        raise

    logger.info(
        "enrichment call completed",
        extra={
            "flow": flow,
            "provider": provider_name,
            "profile_id": profile_id,
            "duration_ms": round((perf_counter() - started_at) * 1000, 2),
            "success": True,
            "context_bytes": context_bytes,
            "schema_bytes": schema_bytes,
            "output_bytes": _payload_size(raw_output),
        },
    )
    return result


def _safe_len(value: Any) -> int:
    if value is None:
        return 0
    if isinstance(value, str):
        return len(value.encode("utf-8"))
    return _payload_size(value)


def _payload_size(value: Any) -> int:
    if value is None:
        return 0
    if isinstance(value, str):
        return len(value.encode("utf-8"))
    try:
        return len(json.dumps(value, ensure_ascii=True, sort_keys=True).encode("utf-8"))
    except TypeError:
        return len(str(value).encode("utf-8"))
