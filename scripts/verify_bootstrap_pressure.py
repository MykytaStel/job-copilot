#!/usr/bin/env python3
from __future__ import annotations

import concurrent.futures
import json
import math
import os
import statistics
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone


@dataclass(frozen=True)
class SubmittedTask:
    profile_id: str
    task_id: str
    submitted_at: datetime


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def parse_iso8601(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = min(len(ordered) - 1, max(0, math.ceil(len(ordered) * fraction) - 1))
    return ordered[index]


def non_negative_duration_ms(started_at: datetime, finished_at: datetime) -> float:
    return round(max(0.0, (finished_at - started_at).total_seconds() * 1000), 2)


def parse_bootstrap_runtime_detail(detail: str | None) -> dict[str, int]:
    counts = {
        "active_jobs": 0,
        "max_concurrent_jobs": 0,
        "available_slots": 0,
        "queued": 0,
        "accepted": 0,
        "running": 0,
        "completed": 0,
        "failed": 0,
    }
    if not detail:
        return counts

    for part in detail.split(","):
        key, _, raw_value = part.strip().partition("=")
        if not raw_value:
            continue
        key = key.strip()
        raw_value = raw_value.strip()
        if key == "active":
            active_raw, _, max_raw = raw_value.partition("/")
            try:
                counts["active_jobs"] = int(active_raw)
                counts["max_concurrent_jobs"] = int(max_raw)
            except ValueError:
                continue
            continue
        if key in counts:
            try:
                counts[key] = int(raw_value)
            except ValueError:
                continue

    return counts


def normalize_runtime_sample(
    sample: dict[str, int],
    baseline: dict[str, int],
) -> dict[str, int]:
    normalized = dict(sample)
    for key in ("accepted", "running", "completed", "failed"):
        normalized[key] = max(0, sample.get(key, 0) - baseline.get(key, 0))
    return normalized


def summarize_task_results(
    task_results: list[dict[str, object]],
    runtime_samples: list[dict[str, int]],
) -> dict[str, object]:
    terminal_status_counts: dict[str, int] = {}
    promotion_decisions: dict[str, int] = {}
    result_reason_counts: dict[str, int] = {}
    total_durations_ms: list[float] = []
    queue_delays_ms: list[float] = []
    retrained_count = 0
    completed_without_retrain_count = 0
    timeout_count = 0

    for result in task_results:
        status = str(result.get("status", "unknown"))
        terminal_status_counts[status] = terminal_status_counts.get(status, 0) + 1

        if status == "timeout":
            timeout_count += 1

        duration_ms = result.get("duration_ms")
        if isinstance(duration_ms, (int, float)):
            total_durations_ms.append(float(duration_ms))

        queue_delay_ms = result.get("queue_delay_ms")
        if isinstance(queue_delay_ms, (int, float)):
            queue_delays_ms.append(float(queue_delay_ms))

        retrained = result.get("retrained")
        if retrained is True:
            retrained_count += 1
        elif status == "completed":
            completed_without_retrain_count += 1

        promotion_decision = result.get("promotion_decision")
        if isinstance(promotion_decision, str) and promotion_decision.strip():
            promotion_decisions[promotion_decision] = (
                promotion_decisions.get(promotion_decision, 0) + 1
            )

        reason = result.get("reason")
        if isinstance(reason, str) and reason.strip():
            result_reason_counts[reason] = result_reason_counts.get(reason, 0) + 1

    max_active_jobs = max((sample["active_jobs"] for sample in runtime_samples), default=0)
    max_accepted_queue = max((sample["accepted"] for sample in runtime_samples), default=0)
    max_queued_jobs = max((sample["queued"] for sample in runtime_samples), default=0)
    max_running_tasks = max((sample["running"] for sample in runtime_samples), default=0)
    max_completed_tasks = max((sample["completed"] for sample in runtime_samples), default=0)
    max_failed_tasks = max((sample["failed"] for sample in runtime_samples), default=0)
    saturation_samples = sum(
        1
        for sample in runtime_samples
        if sample["active_jobs"] >= sample["max_concurrent_jobs"] > 0 and sample["accepted"] > 0
    )

    summary: dict[str, object] = {
        "task_count": len(task_results),
        "terminal_status_counts": terminal_status_counts,
        "retrained_count": retrained_count,
        "completed_without_retrain_count": completed_without_retrain_count,
        "timeout_count": timeout_count,
        "promotion_decisions": promotion_decisions,
        "reasons": result_reason_counts,
        "runtime_observations": {
            "sample_count": len(runtime_samples),
            "saturation_samples": saturation_samples,
            "max_active_jobs": max_active_jobs,
            "max_queued_jobs": max_queued_jobs,
            "max_accepted_queue": max_accepted_queue,
            "max_running_tasks": max_running_tasks,
            "max_completed_tasks": max_completed_tasks,
            "max_failed_tasks": max_failed_tasks,
        },
    }

    if total_durations_ms:
        summary["duration_ms"] = {
            "mean": round(statistics.fmean(total_durations_ms), 2),
            "p50": round(percentile(total_durations_ms, 0.50), 2),
            "p95": round(percentile(total_durations_ms, 0.95), 2),
            "max": round(max(total_durations_ms), 2),
        }

    if queue_delays_ms:
        summary["queue_delay_ms"] = {
            "mean": round(statistics.fmean(queue_delays_ms), 2),
            "p50": round(percentile(queue_delays_ms, 0.50), 2),
            "p95": round(percentile(queue_delays_ms, 0.95), 2),
            "max": round(max(queue_delays_ms), 2),
        }

    return summary


def build_profile_sequence(profile_ids: list[str], total_requests: int) -> list[str]:
    if not profile_ids:
        return []
    return [profile_ids[index % len(profile_ids)] for index in range(max(1, total_requests))]


def request_json(
    *,
    method: str,
    url: str,
    timeout_seconds: float,
    body: bytes | None = None,
    headers: dict[str, str] | None = None,
) -> tuple[int, object]:
    request = urllib.request.Request(
        url,
        data=body,
        method=method,
        headers=headers or {},
    )
    with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
        payload = response.read()
        return response.status, json.loads(payload.decode("utf-8"))


def post_bootstrap_request(
    *,
    ml_base_url: str,
    profile_id: str,
    min_examples: int,
    timeout_seconds: float,
    headers: dict[str, str],
) -> dict[str, object]:
    payload = json.dumps({"profile_id": profile_id, "min_examples": min_examples}).encode("utf-8")
    status_code, response = request_json(
        method="POST",
        url=f"{ml_base_url}/api/v1/reranker/bootstrap",
        timeout_seconds=timeout_seconds,
        body=payload,
        headers={"Content-Type": "application/json", **headers},
    )
    return {
        "profile_id": profile_id,
        "status_code": status_code,
        "task_id": response.get("task_id") if isinstance(response, dict) else None,
        "response": response,
    }


def fetch_bootstrap_status(
    *,
    ml_base_url: str,
    task_id: str,
    timeout_seconds: float,
    headers: dict[str, str],
) -> dict[str, object]:
    _, response = request_json(
        method="GET",
        url=f"{ml_base_url}/api/v1/reranker/bootstrap/{urllib.parse.quote(task_id)}",
        timeout_seconds=timeout_seconds,
        headers=headers,
    )
    if not isinstance(response, dict):
        raise ValueError("bootstrap status payload must be an object")
    return response


def fetch_ready_runtime(
    *,
    ml_base_url: str,
    timeout_seconds: float,
    headers: dict[str, str],
) -> dict[str, int]:
    try:
        _, response = request_json(
            method="GET",
            url=f"{ml_base_url}/ready",
            timeout_seconds=timeout_seconds,
            headers=headers,
        )
    except Exception:
        return {
            "active_jobs": 0,
            "max_concurrent_jobs": 0,
            "available_slots": 0,
            "accepted": 0,
            "running": 0,
            "completed": 0,
            "failed": 0,
        }

    if not isinstance(response, dict):
        return {
            "active_jobs": 0,
            "max_concurrent_jobs": 0,
            "available_slots": 0,
            "accepted": 0,
            "running": 0,
            "completed": 0,
            "failed": 0,
        }

    checks = response.get("checks")
    if not isinstance(checks, list):
        return {
            "active_jobs": 0,
            "max_concurrent_jobs": 0,
            "available_slots": 0,
            "accepted": 0,
            "running": 0,
            "completed": 0,
            "failed": 0,
        }

    for check in checks:
        if not isinstance(check, dict):
            continue
        if check.get("name") == "bootstrap_runtime":
            return parse_bootstrap_runtime_detail(check.get("detail"))

    return {
        "active_jobs": 0,
        "max_concurrent_jobs": 0,
        "available_slots": 0,
        "accepted": 0,
        "running": 0,
        "completed": 0,
        "failed": 0,
    }


def main() -> int:
    ml_base_url = os.getenv("ML_BASE_URL", "http://127.0.0.1:8000").rstrip("/")
    profile_ids = [
        profile_id.strip()
        for profile_id in os.getenv("PROFILE_IDS_CSV", os.getenv("PROFILE_ID", "")).split(",")
        if profile_id.strip()
    ]
    total_requests = int(os.getenv("VERIFY_BOOTSTRAP_REQUESTS", str(len(profile_ids) or 1)))
    concurrency = max(1, int(os.getenv("VERIFY_BOOTSTRAP_CONCURRENCY", "4")))
    min_examples = max(1, int(os.getenv("VERIFY_BOOTSTRAP_MIN_EXAMPLES", "15")))
    timeout_seconds = max(1.0, float(os.getenv("VERIFY_BOOTSTRAP_TIMEOUT_SECONDS", "60")))
    poll_interval_seconds = max(
        0.1,
        float(os.getenv("VERIFY_BOOTSTRAP_POLL_INTERVAL_SECONDS", "0.5")),
    )
    internal_token = os.getenv("ML_INTERNAL_TOKEN", "").strip()
    headers = {"X-Internal-Token": internal_token} if internal_token else {}

    if not profile_ids:
        print("Set PROFILE_IDS_CSV or PROFILE_ID before running bootstrap pressure verification.", file=sys.stderr)
        return 1

    request_profiles = build_profile_sequence(profile_ids, total_requests)
    print(
        json.dumps(
            {
                "ml_base_url": ml_base_url,
                "request_count": len(request_profiles),
                "concurrency": concurrency,
                "min_examples": min_examples,
                "profile_ids": request_profiles,
                "timeout_seconds": timeout_seconds,
                "poll_interval_seconds": poll_interval_seconds,
            },
            indent=2,
        )
    )

    accepted_tasks: list[SubmittedTask] = []
    submission_errors: list[dict[str, object]] = []

    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [
            executor.submit(
                post_bootstrap_request,
                ml_base_url=ml_base_url,
                profile_id=profile_id,
                min_examples=min_examples,
                timeout_seconds=timeout_seconds,
                headers=headers,
            )
            for profile_id in request_profiles
        ]
        for future in concurrent.futures.as_completed(futures):
            try:
                result = future.result()
            except urllib.error.HTTPError as exc:
                submission_errors.append(
                    {
                        "status_code": exc.code,
                        "detail": exc.read().decode("utf-8", errors="replace"),
                    }
                )
                continue
            except Exception as exc:
                submission_errors.append({"status_code": "error", "detail": str(exc)})
                continue

            task_id = result.get("task_id")
            if result.get("status_code") == 202 and isinstance(task_id, str) and task_id:
                accepted_tasks.append(
                    SubmittedTask(
                        profile_id=str(result["profile_id"]),
                        task_id=task_id,
                        submitted_at=utc_now(),
                    )
                )
            else:
                submission_errors.append(result)

    baseline_runtime = fetch_ready_runtime(
        ml_base_url=ml_base_url,
        timeout_seconds=timeout_seconds,
        headers=headers,
    )
    runtime_samples = [normalize_runtime_sample(baseline_runtime, baseline_runtime)]

    unfinished = {task.task_id: task for task in accepted_tasks}
    task_results: list[dict[str, object]] = []
    deadline = time.monotonic() + timeout_seconds

    while unfinished and time.monotonic() < deadline:
        runtime_samples.append(
            normalize_runtime_sample(
                fetch_ready_runtime(
                    ml_base_url=ml_base_url,
                    timeout_seconds=timeout_seconds,
                    headers=headers,
                ),
                baseline_runtime,
            )
        )
        for task_id, task in list(unfinished.items()):
            try:
                payload = fetch_bootstrap_status(
                    ml_base_url=ml_base_url,
                    task_id=task_id,
                    timeout_seconds=timeout_seconds,
                    headers=headers,
                )
            except Exception:
                continue

            status = str(payload.get("status", "")).strip().lower()
            if status not in {"completed", "failed"}:
                continue

            started_at = parse_iso8601(payload.get("started_at") if isinstance(payload.get("started_at"), str) else None)
            finished_at = parse_iso8601(payload.get("finished_at") if isinstance(payload.get("finished_at"), str) else None)
            result_payload = payload.get("result")
            retrained = None
            reason = payload.get("error")
            if isinstance(result_payload, dict):
                retrained = result_payload.get("retrained")
                reason = result_payload.get("reason")

            task_result: dict[str, object] = {
                "task_id": task.task_id,
                "profile_id": task.profile_id,
                "status": status,
                "retrained": retrained,
                "reason": reason,
                "promotion_decision": payload.get("promotion_decision"),
            }
            if started_at is not None:
                task_result["queue_delay_ms"] = non_negative_duration_ms(
                    task.submitted_at,
                    started_at,
                )
            if finished_at is not None:
                task_result["duration_ms"] = non_negative_duration_ms(
                    task.submitted_at,
                    finished_at,
                )
            task_results.append(task_result)
            unfinished.pop(task_id, None)

        if unfinished:
            time.sleep(poll_interval_seconds)

    if unfinished:
        for task in unfinished.values():
            task_results.append(
                {
                    "task_id": task.task_id,
                    "profile_id": task.profile_id,
                    "status": "timeout",
                    "reason": "task did not reach a terminal state before timeout",
                }
            )

    runtime_samples.append(
        normalize_runtime_sample(
            fetch_ready_runtime(
                ml_base_url=ml_base_url,
                timeout_seconds=timeout_seconds,
                headers=headers,
            ),
            baseline_runtime,
        )
    )

    summary = summarize_task_results(task_results, runtime_samples)
    summary["accepted_task_count"] = len(accepted_tasks)
    summary["submission_error_count"] = len(submission_errors)
    summary["submission_errors"] = submission_errors

    print(json.dumps(summary, indent=2))

    has_failures = bool(summary["submission_error_count"]) or bool(
        summary["terminal_status_counts"].get("failed", 0)
    )
    has_timeouts = bool(summary["terminal_status_counts"].get("timeout", 0))
    return 1 if has_failures or has_timeouts else 0


if __name__ == "__main__":
    raise SystemExit(main())
