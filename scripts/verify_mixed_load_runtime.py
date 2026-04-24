#!/usr/bin/env python3
from __future__ import annotations

import concurrent.futures
import json
import math
import os
from pathlib import Path
import re
import subprocess
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


_PROFILE_SEGMENT_RE = re.compile(r"[^A-Za-z0-9._-]+")


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = min(len(ordered) - 1, max(0, math.ceil(len(ordered) * fraction) - 1))
    return ordered[index]


def parse_iso8601(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


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


def sanitize_profile_segment(profile_id: str) -> str:
    cleaned = _PROFILE_SEGMENT_RE.sub("_", profile_id.strip())
    return cleaned or "unknown-profile"


def local_runtime_cleanup_paths(
    *,
    repo_root: Path,
    profile_ids: list[str],
    task_ids: list[str],
) -> dict[str, object]:
    ml_root = repo_root / "apps" / "ml"
    task_paths = [
        ml_root / ".runtime" / "bootstrap-tasks" / "tasks" / f"{task_id}.json"
        for task_id in task_ids
    ]
    profile_dirs = [
        ml_root / "models" / "profiles" / sanitize_profile_segment(profile_id)
        for profile_id in profile_ids
    ]
    return {
        "task_paths": task_paths,
        "profile_dirs": profile_dirs,
        "runtime_artifact_path": ml_root / "models" / "trained-reranker-v3.json",
        "profile_root_dir": ml_root / "models" / "profiles",
    }


def cleanup_local_runtime_outputs(
    *,
    repo_root: Path,
    profile_ids: list[str],
    task_ids: list[str],
) -> dict[str, object]:
    paths = local_runtime_cleanup_paths(
        repo_root=repo_root,
        profile_ids=profile_ids,
        task_ids=task_ids,
    )
    deleted_task_files = 0
    deleted_profile_dirs = 0
    runtime_artifact_deleted = False
    errors: list[str] = []

    task_root = (
        repo_root / "apps" / "ml" / ".runtime" / "bootstrap-tasks" / "tasks"
    )
    if task_root.exists():
        for task_path in task_root.glob("*.json"):
            try:
                task_path.unlink()
                deleted_task_files += 1
            except OSError as exc:
                errors.append(f"{task_path}: {exc}")

    profile_root_dir = paths["profile_root_dir"]
    if isinstance(profile_root_dir, Path) and profile_root_dir.exists():
        for profile_dir in profile_root_dir.iterdir():
            if not profile_dir.is_dir():
                continue
            try:
                for child in sorted(profile_dir.rglob("*"), reverse=True):
                    if child.is_file() or child.is_symlink():
                        child.unlink()
                    elif child.is_dir():
                        child.rmdir()
                profile_dir.rmdir()
                deleted_profile_dirs += 1
            except OSError as exc:
                errors.append(f"{profile_dir}: {exc}")

    runtime_artifact_path = paths["runtime_artifact_path"]
    if isinstance(runtime_artifact_path, Path):
        try:
            if runtime_artifact_path.exists():
                runtime_artifact_path.unlink()
                runtime_artifact_deleted = True
        except OSError as exc:
            errors.append(f"{runtime_artifact_path}: {exc}")

    profile_root_dir = paths["profile_root_dir"]
    if isinstance(profile_root_dir, Path):
        try:
            if profile_root_dir.exists() and not any(profile_root_dir.iterdir()):
                profile_root_dir.rmdir()
        except OSError:
            pass

    return {
        "deleted_task_files": deleted_task_files,
        "deleted_profile_dirs": deleted_profile_dirs,
        "runtime_artifact_deleted": runtime_artifact_deleted,
        "errors": errors,
    }


def cleanup_docker_runtime_outputs(
    *,
    container_name: str,
    profile_ids: list[str],
    task_ids: list[str],
    timeout_seconds: float,
) -> dict[str, object]:
    if not container_name:
        return {
            "container": None,
            "deleted_task_files": 0,
            "deleted_profile_dirs": 0,
            "runtime_artifact_deleted": False,
            "errors": [],
        }

    commands = []
    commands.append("find /app/.runtime/bootstrap-tasks/tasks -maxdepth 1 -type f -name '*.json' -delete 2>/dev/null || true")
    commands.append("find /app/models/profiles -mindepth 1 -maxdepth 1 -type d -exec rm -rf {} + 2>/dev/null || true")
    commands.append("rm -f /app/models/trained-reranker-v3.json")
    commands.append("rmdir /app/models/profiles >/dev/null 2>&1 || true")

    try:
        completed = subprocess.run(
            ["docker", "exec", container_name, "sh", "-lc", " && ".join(commands)],
            capture_output=True,
            text=True,
            timeout=max(1.0, timeout_seconds),
            check=False,
        )
    except Exception as exc:
        return {
            "container": container_name,
            "deleted_task_files": 0,
            "deleted_profile_dirs": 0,
            "runtime_artifact_deleted": False,
            "errors": [str(exc)],
        }

    errors: list[str] = []
    if completed.returncode != 0:
        errors.append((completed.stderr or completed.stdout or "docker exec cleanup failed").strip())

    return {
        "container": container_name,
        "deleted_task_files": "all",
        "deleted_profile_dirs": "all",
        "runtime_artifact_deleted": True,
        "errors": errors,
    }


def parse_percentage(value: str | None) -> float | None:
    if not value:
        return None
    cleaned = value.strip().rstrip("%").strip()
    if not cleaned:
        return None
    try:
        return float(cleaned)
    except ValueError:
        return None


def parse_bytes(value: str | None) -> int | None:
    if not value:
        return None
    cleaned = value.strip()
    if not cleaned:
        return None

    units = {
        "b": 1,
        "kib": 1024,
        "kb": 1000,
        "mib": 1024**2,
        "mb": 1000**2,
        "gib": 1024**3,
        "gb": 1000**3,
        "tib": 1024**4,
        "tb": 1000**4,
    }
    number = ""
    unit = ""
    for char in cleaned:
        if char.isdigit() or char in {".", "-"}:
            number += char
        elif not char.isspace():
            unit += char
    if not number:
        return None
    try:
        numeric = float(number)
    except ValueError:
        return None
    multiplier = units.get(unit.lower(), 1)
    return int(numeric * multiplier)


def mib_from_bytes(value: int | None) -> float | None:
    if value is None:
        return None
    return round(value / (1024**2), 2)


def parse_docker_stats_payload(payload: dict[str, object]) -> dict[str, object]:
    memory_usage = str(payload.get("MemUsage") or "")
    usage_raw, _, limit_raw = memory_usage.partition("/")
    usage_bytes = parse_bytes(usage_raw)
    limit_bytes = parse_bytes(limit_raw)

    return {
        "container": str(payload.get("Name") or payload.get("Container") or "unknown"),
        "cpu_percent": parse_percentage(str(payload.get("CPUPerc") or "")),
        "memory_percent": parse_percentage(str(payload.get("MemPerc") or "")),
        "memory_usage_mib": mib_from_bytes(usage_bytes),
        "memory_limit_mib": mib_from_bytes(limit_bytes),
    }


def sample_container_stats(
    *,
    container_names: list[str],
    timeout_seconds: float,
) -> list[dict[str, object]]:
    if not container_names:
        return []

    command = [
        "docker",
        "stats",
        "--no-stream",
        "--format",
        "{{ json . }}",
        *container_names,
    ]
    try:
        completed = subprocess.run(
            command,
            capture_output=True,
            text=True,
            timeout=max(1.0, timeout_seconds),
            check=False,
        )
    except Exception as exc:
        return [
            {
                "container": container_name,
                "error": str(exc),
            }
            for container_name in container_names
        ]

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "docker stats failed").strip()
        return [
            {
                "container": container_name,
                "error": detail,
            }
            for container_name in container_names
        ]

    parsed_by_name: dict[str, dict[str, object]] = {}
    for line in completed.stdout.splitlines():
        cleaned = line.strip()
        if not cleaned:
            continue
        try:
            payload = json.loads(cleaned)
        except json.JSONDecodeError:
            continue
        if not isinstance(payload, dict):
            continue
        parsed = parse_docker_stats_payload(payload)
        parsed_by_name[str(parsed["container"])] = parsed

    samples: list[dict[str, object]] = []
    for container_name in container_names:
        sample = parsed_by_name.get(container_name)
        if sample is None:
            samples.append({"container": container_name, "error": "container stats missing"})
        else:
            samples.append(sample)
    return samples


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


def run_rerank_request(
    *,
    ml_base_url: str,
    profile_id: str,
    job_ids: list[str],
    timeout_seconds: float,
    headers: dict[str, str],
) -> dict[str, object]:
    payload = json.dumps({"profile_id": profile_id, "job_ids": job_ids}).encode("utf-8")
    started = time.perf_counter()
    try:
        status_code, response = request_json(
            method="POST",
            url=f"{ml_base_url}/api/v1/rerank",
            timeout_seconds=timeout_seconds,
            body=payload,
            headers={"Content-Type": "application/json", **headers},
        )
        duration_ms = round((time.perf_counter() - started) * 1000, 2)
        jobs = response.get("jobs") if isinstance(response, dict) else None
        returned_jobs = len(jobs) if isinstance(jobs, list) else None
        return {
            "ok": 200 <= status_code < 400,
            "status": status_code,
            "duration_ms": duration_ms,
            "returned_jobs": returned_jobs,
        }
    except urllib.error.HTTPError as exc:
        duration_ms = round((time.perf_counter() - started) * 1000, 2)
        return {"ok": False, "status": exc.code, "duration_ms": duration_ms}
    except Exception:
        duration_ms = round((time.perf_counter() - started) * 1000, 2)
        return {"ok": False, "status": "error", "duration_ms": duration_ms}


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


def fetch_ready_payload(
    *,
    ml_base_url: str,
    timeout_seconds: float,
    headers: dict[str, str],
) -> dict[str, object] | None:
    try:
        _, response = request_json(
            method="GET",
            url=f"{ml_base_url}/ready",
            timeout_seconds=timeout_seconds,
            headers=headers,
        )
    except Exception:
        return None
    return response if isinstance(response, dict) else None


def extract_bootstrap_runtime_detail(ready_payload: dict[str, object] | None) -> tuple[object | None, str | None]:
    if not isinstance(ready_payload, dict):
        return None, None

    ready_status = ready_payload.get("status")
    checks = ready_payload.get("checks")
    if not isinstance(checks, list):
        return ready_status, None

    for check in checks:
        if isinstance(check, dict) and check.get("name") == "bootstrap_runtime":
            detail = check.get("detail")
            return ready_status, detail if isinstance(detail, str) else None
    return ready_status, None


def summarize_request_results(results: list[dict[str, object]]) -> dict[str, object]:
    durations = [float(result["duration_ms"]) for result in results]
    status_counts: dict[str, int] = {}
    error_count = 0
    total_returned_jobs = 0
    returned_jobs_samples = 0

    for result in results:
        status_key = str(result.get("status", "unknown"))
        status_counts[status_key] = status_counts.get(status_key, 0) + 1
        if not result.get("ok"):
            error_count += 1
        returned_jobs = result.get("returned_jobs")
        if isinstance(returned_jobs, int):
            total_returned_jobs += returned_jobs
            returned_jobs_samples += 1

    summary: dict[str, object] = {
        "requests": len(results),
        "errors": error_count,
        "success_rate": round((len(results) - error_count) / len(results), 4) if results else 0.0,
        "mean_ms": round(statistics.fmean(durations), 2) if durations else 0.0,
        "p50_ms": round(percentile(durations, 0.50), 2) if durations else 0.0,
        "p95_ms": round(percentile(durations, 0.95), 2) if durations else 0.0,
        "max_ms": round(max(durations), 2) if durations else 0.0,
        "status_counts": status_counts,
    }
    if returned_jobs_samples > 0:
        summary["avg_returned_jobs"] = round(total_returned_jobs / returned_jobs_samples, 2)
    return summary


def build_profile_sequence(profile_ids: list[str], total_requests: int) -> list[str]:
    if not profile_ids:
        return []
    return [profile_ids[index % len(profile_ids)] for index in range(max(1, total_requests))]


def summarize_runtime_samples(samples: list[dict[str, object]]) -> dict[str, object]:
    parsed_samples: list[dict[str, int]] = []
    degraded_samples = 0

    for sample in samples:
        if sample.get("status") == "degraded":
            degraded_samples += 1
        normalized_runtime = sample.get("normalized_runtime")
        if isinstance(normalized_runtime, dict):
            parsed_samples.append(
                {
                    "active_jobs": int(normalized_runtime.get("active_jobs", 0)),
                    "max_concurrent_jobs": int(normalized_runtime.get("max_concurrent_jobs", 0)),
                    "available_slots": int(normalized_runtime.get("available_slots", 0)),
                    "queued": int(normalized_runtime.get("queued", 0)),
                    "accepted": int(normalized_runtime.get("accepted", 0)),
                    "running": int(normalized_runtime.get("running", 0)),
                    "completed": int(normalized_runtime.get("completed", 0)),
                    "failed": int(normalized_runtime.get("failed", 0)),
                }
            )
            continue
        detail = sample.get("bootstrap_runtime_detail")
        parsed_samples.append(parse_bootstrap_runtime_detail(detail if isinstance(detail, str) else None))

    return {
        "sample_count": len(samples),
        "degraded_samples": degraded_samples,
        "max_active_jobs": max((sample["active_jobs"] for sample in parsed_samples), default=0),
        "max_queued_jobs": max((sample["queued"] for sample in parsed_samples), default=0),
        "max_accepted_tasks": max((sample["accepted"] for sample in parsed_samples), default=0),
        "max_running_tasks": max((sample["running"] for sample in parsed_samples), default=0),
        "max_completed_tasks": max((sample["completed"] for sample in parsed_samples), default=0),
        "max_failed_tasks": max((sample["failed"] for sample in parsed_samples), default=0),
    }


def summarize_resource_samples(samples: list[dict[str, object]]) -> dict[str, object]:
    by_container: dict[str, list[dict[str, object]]] = {}
    for sample in samples:
        container = str(sample.get("container") or "unknown")
        by_container.setdefault(container, []).append(sample)

    summary: dict[str, object] = {}
    for container, container_samples in by_container.items():
        successful = [sample for sample in container_samples if sample.get("error") is None]
        cpu_values = [
            float(sample["cpu_percent"])
            for sample in successful
            if isinstance(sample.get("cpu_percent"), (int, float))
        ]
        mem_percent_values = [
            float(sample["memory_percent"])
            for sample in successful
            if isinstance(sample.get("memory_percent"), (int, float))
        ]
        mem_mib_values = [
            float(sample["memory_usage_mib"])
            for sample in successful
            if isinstance(sample.get("memory_usage_mib"), (int, float))
        ]

        container_summary: dict[str, object] = {
            "sample_count": len(container_samples),
            "successful_samples": len(successful),
            "failed_samples": len(container_samples) - len(successful),
        }
        if cpu_values:
            container_summary["cpu_percent"] = {
                "mean": round(statistics.fmean(cpu_values), 2),
                "p95": round(percentile(cpu_values, 0.95), 2),
                "max": round(max(cpu_values), 2),
            }
        if mem_percent_values:
            container_summary["memory_percent"] = {
                "mean": round(statistics.fmean(mem_percent_values), 2),
                "p95": round(percentile(mem_percent_values, 0.95), 2),
                "max": round(max(mem_percent_values), 2),
            }
        if mem_mib_values:
            container_summary["memory_usage_mib"] = {
                "mean": round(statistics.fmean(mem_mib_values), 2),
                "p95": round(percentile(mem_mib_values, 0.95), 2),
                "max": round(max(mem_mib_values), 2),
            }
        if not successful:
            first_error = next(
                (
                    str(sample.get("error"))
                    for sample in container_samples
                    if isinstance(sample.get("error"), str) and sample.get("error")
                ),
                "resource sampling unavailable",
            )
            container_summary["error"] = first_error
        summary[container] = container_summary

    return summary


def slowdown_ratio(baseline_ms: float, observed_ms: float) -> float:
    if baseline_ms <= 0:
        return 0.0
    return round(observed_ms / baseline_ms, 2)


def summarize_bootstrap_results(task_results: list[dict[str, object]]) -> dict[str, object]:
    terminal_status_counts: dict[str, int] = {}
    promotion_decisions: dict[str, int] = {}
    reasons: dict[str, int] = {}
    total_durations_ms: list[float] = []
    queue_delays_ms: list[float] = []
    retrained_count = 0

    for result in task_results:
        status = str(result.get("status", "unknown"))
        terminal_status_counts[status] = terminal_status_counts.get(status, 0) + 1
        if result.get("retrained") is True:
            retrained_count += 1
        duration_ms = result.get("duration_ms")
        if isinstance(duration_ms, (int, float)):
            total_durations_ms.append(float(duration_ms))
        queue_delay_ms = result.get("queue_delay_ms")
        if isinstance(queue_delay_ms, (int, float)):
            queue_delays_ms.append(float(queue_delay_ms))
        promotion_decision = result.get("promotion_decision")
        if isinstance(promotion_decision, str) and promotion_decision.strip():
            promotion_decisions[promotion_decision] = promotion_decisions.get(promotion_decision, 0) + 1
        reason = result.get("reason")
        if isinstance(reason, str) and reason.strip():
            reasons[reason] = reasons.get(reason, 0) + 1

    summary: dict[str, object] = {
        "task_count": len(task_results),
        "terminal_status_counts": terminal_status_counts,
        "retrained_count": retrained_count,
        "promotion_decisions": promotion_decisions,
        "reasons": reasons,
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


def main() -> int:
    ml_base_url = os.getenv("ML_BASE_URL", "http://127.0.0.1:8000").rstrip("/")
    profile_id = os.getenv("PROFILE_ID", "").strip()
    bootstrap_profile_ids = [
        profile.strip()
        for profile in os.getenv("PROFILE_IDS_CSV", "").split(",")
        if profile.strip()
    ]
    job_ids = [job_id.strip() for job_id in os.getenv("JOB_IDS_CSV", "").split(",") if job_id.strip()]
    timeout_seconds = max(0.5, float(os.getenv("VERIFY_MIXED_TIMEOUT_SECONDS", "60")))
    baseline_requests = max(1, int(os.getenv("VERIFY_MIXED_BASELINE_REQUESTS", "20")))
    rerank_requests = max(1, int(os.getenv("VERIFY_MIXED_RERANK_REQUESTS", "60")))
    rerank_concurrency = max(1, int(os.getenv("VERIFY_MIXED_RERANK_CONCURRENCY", "10")))
    bootstrap_requests = max(
        1,
        int(os.getenv("VERIFY_MIXED_BOOTSTRAP_REQUESTS", str(len(bootstrap_profile_ids) or 1))),
    )
    bootstrap_concurrency = max(
        1, int(os.getenv("VERIFY_MIXED_BOOTSTRAP_CONCURRENCY", "4"))
    )
    bootstrap_min_examples = max(
        1, int(os.getenv("VERIFY_MIXED_BOOTSTRAP_MIN_EXAMPLES", "15"))
    )
    poll_interval_seconds = max(
        0.1, float(os.getenv("VERIFY_MIXED_POLL_INTERVAL_SECONDS", "0.25"))
    )
    max_rerank_p95_slowdown = max(
        1.0, float(os.getenv("VERIFY_MIXED_MAX_RERANK_P95_SLOWDOWN", "3.0"))
    )
    max_rerank_error_rate = min(
        1.0, max(0.0, float(os.getenv("VERIFY_MIXED_MAX_RERANK_ERROR_RATE", "0.0")))
    )
    cleanup_runtime = os.getenv("VERIFY_MIXED_CLEANUP_RUNTIME", "true").strip().lower() not in {
        "0",
        "false",
        "no",
    }
    cleanup_docker_container = os.getenv(
        "VERIFY_MIXED_CLEANUP_DOCKER_CONTAINER",
        "job-copilot-ml",
    ).strip()
    internal_token = os.getenv("ML_INTERNAL_TOKEN", "").strip()
    resource_containers = [
        container.strip()
        for container in os.getenv(
            "VERIFY_MIXED_RESOURCE_CONTAINERS_CSV",
            "job-copilot-ml,job-copilot-engine-api",
        ).split(",")
        if container.strip()
    ]

    if not profile_id or not job_ids or not bootstrap_profile_ids:
        print(
            "PROFILE_ID, JOB_IDS_CSV, and PROFILE_IDS_CSV must be set for mixed-load verification.",
            file=sys.stderr,
        )
        return 1

    headers = {"X-Internal-Token": internal_token} if internal_token else {}
    profile_sequence = build_profile_sequence(bootstrap_profile_ids, bootstrap_requests)

    print(
        json.dumps(
            {
                "ml_base_url": ml_base_url,
                "rerank_profile_id": profile_id,
                "job_ids": job_ids,
                "bootstrap_profile_ids": bootstrap_profile_ids,
                "baseline_requests": baseline_requests,
                "rerank_requests": rerank_requests,
                "rerank_concurrency": rerank_concurrency,
                "bootstrap_requests": bootstrap_requests,
                "bootstrap_concurrency": bootstrap_concurrency,
                "bootstrap_min_examples": bootstrap_min_examples,
                "timeout_seconds": timeout_seconds,
                "poll_interval_seconds": poll_interval_seconds,
                "max_rerank_p95_slowdown": max_rerank_p95_slowdown,
                "max_rerank_error_rate": max_rerank_error_rate,
                "cleanup_runtime": cleanup_runtime,
                "cleanup_docker_container": cleanup_docker_container or None,
                "resource_containers": resource_containers,
            },
            indent=2,
        )
    )

    baseline_results: list[dict[str, object]] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=rerank_concurrency) as executor:
        futures = [
            executor.submit(
                run_rerank_request,
                ml_base_url=ml_base_url,
                profile_id=profile_id,
                job_ids=job_ids,
                timeout_seconds=timeout_seconds,
                headers=headers,
            )
            for _ in range(baseline_requests)
        ]
        for future in concurrent.futures.as_completed(futures):
            baseline_results.append(future.result())
    baseline_summary = summarize_request_results(baseline_results)
    baseline_runtime_counts = parse_bootstrap_runtime_detail(None)
    baseline_ready_payload = fetch_ready_payload(
        ml_base_url=ml_base_url,
        timeout_seconds=timeout_seconds,
        headers=headers,
    )
    if isinstance(baseline_ready_payload, dict):
        checks = baseline_ready_payload.get("checks")
        if isinstance(checks, list):
            for check in checks:
                if isinstance(check, dict) and check.get("name") == "bootstrap_runtime":
                    baseline_runtime_counts = parse_bootstrap_runtime_detail(check.get("detail"))
                    break

    submission_errors: list[dict[str, object]] = []
    submitted_tasks: list[SubmittedTask] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=bootstrap_concurrency) as executor:
        futures = [
            executor.submit(
                post_bootstrap_request,
                ml_base_url=ml_base_url,
                profile_id=bootstrap_profile_id,
                min_examples=bootstrap_min_examples,
                timeout_seconds=timeout_seconds,
                headers=headers,
            )
            for bootstrap_profile_id in profile_sequence
        ]
        for future in concurrent.futures.as_completed(futures):
            try:
                response = future.result()
            except Exception as exc:
                submission_errors.append({"status": "error", "detail": str(exc)})
                continue
            task_id = response.get("task_id")
            status_code = response.get("status_code")
            if status_code != 202 or not isinstance(task_id, str) or not task_id.strip():
                submission_errors.append(response)
                continue
            submitted_tasks.append(
                SubmittedTask(
                    profile_id=str(response["profile_id"]),
                    task_id=task_id,
                    submitted_at=utc_now(),
                )
            )

    mixed_rerank_results: list[dict[str, object]] = []
    runtime_samples: list[dict[str, object]] = []
    resource_samples: list[dict[str, object]] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=rerank_concurrency) as executor:
        rerank_futures = [
            executor.submit(
                run_rerank_request,
                ml_base_url=ml_base_url,
                profile_id=profile_id,
                job_ids=job_ids,
                timeout_seconds=timeout_seconds,
                headers=headers,
            )
            for _ in range(rerank_requests)
        ]

        terminal_statuses = {"completed", "failed"}
        pending_task_ids = {task.task_id for task in submitted_tasks}
        task_results: dict[str, dict[str, object]] = {}

        while pending_task_ids or any(not future.done() for future in rerank_futures):
            ready_payload = fetch_ready_payload(
                ml_base_url=ml_base_url,
                timeout_seconds=timeout_seconds,
                headers=headers,
            )
            ready_status, bootstrap_detail = extract_bootstrap_runtime_detail(ready_payload)
            normalized_runtime = normalize_runtime_sample(
                parse_bootstrap_runtime_detail(bootstrap_detail if isinstance(bootstrap_detail, str) else None),
                baseline_runtime_counts,
            )
            resource_samples.extend(
                sample_container_stats(
                    container_names=resource_containers,
                    timeout_seconds=min(timeout_seconds, 5.0),
                )
            )
            runtime_samples.append(
                {
                    "status": ready_status,
                    "bootstrap_runtime_detail": bootstrap_detail,
                    "normalized_runtime": normalized_runtime,
                }
            )

            for task in list(submitted_tasks):
                if task.task_id not in pending_task_ids:
                    continue
                try:
                    status_payload = fetch_bootstrap_status(
                        ml_base_url=ml_base_url,
                        task_id=task.task_id,
                        timeout_seconds=timeout_seconds,
                        headers=headers,
                    )
                except Exception:
                    continue
                status = str(status_payload.get("status", "unknown"))
                if status not in terminal_statuses:
                    continue

                result_payload = status_payload.get("result")
                if not isinstance(result_payload, dict):
                    result_payload = {}
                started_at = parse_iso8601(status_payload.get("started_at"))
                finished_at = parse_iso8601(status_payload.get("finished_at"))
                queue_delay_ms = None
                duration_ms = None
                if started_at is not None:
                    queue_delay_ms = non_negative_duration_ms(task.submitted_at, started_at)
                if started_at is not None and finished_at is not None:
                    duration_ms = non_negative_duration_ms(started_at, finished_at)

                task_results[task.task_id] = {
                    "task_id": task.task_id,
                    "profile_id": task.profile_id,
                    "status": status,
                    "retrained": result_payload.get("retrained"),
                    "reason": result_payload.get("reason") or status_payload.get("error"),
                    "promotion_decision": result_payload.get("promotion_decision"),
                    "duration_ms": duration_ms,
                    "queue_delay_ms": queue_delay_ms,
                }
                pending_task_ids.remove(task.task_id)

            if pending_task_ids or any(not future.done() for future in rerank_futures):
                time.sleep(poll_interval_seconds)

        for future in concurrent.futures.as_completed(rerank_futures):
            mixed_rerank_results.append(future.result())

    mixed_rerank_summary = summarize_request_results(mixed_rerank_results)
    runtime_summary = summarize_runtime_samples(runtime_samples)
    resource_summary = summarize_resource_samples(resource_samples)
    bootstrap_summary = summarize_bootstrap_results(list(task_results.values()))

    slowdown = {
        "mean_ratio": slowdown_ratio(
            float(baseline_summary["mean_ms"]),
            float(mixed_rerank_summary["mean_ms"]),
        ),
        "p95_ratio": slowdown_ratio(
            float(baseline_summary["p95_ms"]),
            float(mixed_rerank_summary["p95_ms"]),
        ),
    }

    ready_before_cleanup = fetch_ready_payload(
        ml_base_url=ml_base_url,
        timeout_seconds=timeout_seconds,
        headers=headers,
    )
    cleanup_summary: dict[str, object] = {
        "enabled": cleanup_runtime,
        "local": None,
        "docker": None,
    }
    if cleanup_runtime:
        repo_root = Path(__file__).resolve().parent.parent
        cleanup_summary["local"] = cleanup_local_runtime_outputs(
            repo_root=repo_root,
            profile_ids=bootstrap_profile_ids,
            task_ids=[task.task_id for task in submitted_tasks],
        )
        if cleanup_docker_container:
            cleanup_summary["docker"] = cleanup_docker_runtime_outputs(
                container_name=cleanup_docker_container,
                profile_ids=bootstrap_profile_ids,
                task_ids=[task.task_id for task in submitted_tasks],
                timeout_seconds=min(timeout_seconds, 10.0),
            )

    ready_after_cleanup = fetch_ready_payload(
        ml_base_url=ml_base_url,
        timeout_seconds=timeout_seconds,
        headers=headers,
    )
    summary = {
        "baseline_rerank": baseline_summary,
        "mixed_rerank": mixed_rerank_summary,
        "rerank_slowdown": slowdown,
        "bootstrap": bootstrap_summary,
        "runtime_observations": runtime_summary,
        "resource_observations": resource_summary,
        "bootstrap_submission_error_count": len(submission_errors),
        "bootstrap_submission_errors": submission_errors,
        "cleanup": cleanup_summary,
        "ready_status_before_cleanup": (
            ready_before_cleanup.get("status")
            if isinstance(ready_before_cleanup, dict)
            else None
        ),
        "ready_status_after_cleanup": (
            ready_after_cleanup.get("status")
            if isinstance(ready_after_cleanup, dict)
            else None
        ),
    }
    print(json.dumps(summary, indent=2))

    rerank_error_rate = (
        float(mixed_rerank_summary["errors"]) / float(mixed_rerank_summary["requests"])
        if mixed_rerank_summary["requests"]
        else 1.0
    )

    exit_code = 0
    if submission_errors:
        exit_code = 1
    if bootstrap_summary["task_count"] != len(submitted_tasks):
        exit_code = 1
    if bootstrap_summary["terminal_status_counts"].get("completed", 0) != len(submitted_tasks):
        exit_code = 1
    if rerank_error_rate > max_rerank_error_rate:
        exit_code = 1
    if float(slowdown["p95_ratio"]) > max_rerank_p95_slowdown:
        exit_code = 1
    if cleanup_runtime:
        local_cleanup = cleanup_summary.get("local")
        docker_cleanup = cleanup_summary.get("docker")
        if isinstance(local_cleanup, dict) and local_cleanup.get("errors"):
            exit_code = 1
        if isinstance(docker_cleanup, dict) and docker_cleanup.get("errors"):
            exit_code = 1

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
