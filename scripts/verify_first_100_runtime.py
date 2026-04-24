#!/usr/bin/env python3
from __future__ import annotations

import json
import math
import os
import statistics
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass


@dataclass(frozen=True)
class CheckTarget:
    name: str
    method: str
    url: str
    body: bytes | None = None
    headers: dict[str, str] | None = None


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = min(len(ordered) - 1, max(0, math.ceil(len(ordered) * fraction) - 1))
    return ordered[index]


def run_request(target: CheckTarget, timeout_seconds: float) -> dict[str, object]:
    request = urllib.request.Request(
        target.url,
        data=target.body,
        method=target.method,
        headers=target.headers or {},
    )
    started = time.perf_counter()
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            response.read()
            duration_ms = round((time.perf_counter() - started) * 1000, 2)
            return {
                "ok": 200 <= response.status < 400,
                "status": response.status,
                "duration_ms": duration_ms,
            }
    except urllib.error.HTTPError as exc:
        duration_ms = round((time.perf_counter() - started) * 1000, 2)
        return {"ok": False, "status": exc.code, "duration_ms": duration_ms}
    except Exception:
        duration_ms = round((time.perf_counter() - started) * 1000, 2)
        return {"ok": False, "status": "error", "duration_ms": duration_ms}


def build_targets() -> list[CheckTarget]:
    engine_base = os.getenv("ENGINE_API_BASE_URL", "http://127.0.0.1:8080").rstrip("/")
    ml_base = os.getenv("ML_BASE_URL", "http://127.0.0.1:8000").rstrip("/")
    profile_id = os.getenv("PROFILE_ID", "").strip()
    job_ids = [
        job_id.strip()
        for job_id in os.getenv("JOB_IDS_CSV", "").split(",")
        if job_id.strip()
    ]
    internal_token = os.getenv("ML_INTERNAL_TOKEN", "").strip()

    ml_headers = {}
    if internal_token:
        ml_headers["X-Internal-Token"] = internal_token

    targets = [
        CheckTarget(name="engine_health", method="GET", url=f"{engine_base}/health"),
        CheckTarget(name="ml_health", method="GET", url=f"{ml_base}/health", headers=ml_headers),
        CheckTarget(name="ml_ready", method="GET", url=f"{ml_base}/ready", headers=ml_headers),
    ]

    if profile_id:
        targets.extend(
            [
                CheckTarget(
                    name="analytics_summary",
                    method="GET",
                    url=f"{engine_base}/api/v1/profiles/{urllib.parse.quote(profile_id)}/analytics/summary",
                ),
                CheckTarget(
                    name="reranker_metrics",
                    method="GET",
                    url=f"{engine_base}/api/v1/profiles/{urllib.parse.quote(profile_id)}/reranker/metrics",
                ),
            ]
        )

    if profile_id and job_ids:
        payload = json.dumps({"profile_id": profile_id, "job_ids": job_ids}).encode("utf-8")
        targets.append(
            CheckTarget(
                name="ml_rerank",
                method="POST",
                url=f"{ml_base}/api/v1/rerank",
                body=payload,
                headers={
                    **ml_headers,
                    "Content-Type": "application/json",
                },
            )
        )

    return targets


def main() -> int:
    total_requests = max(1, int(os.getenv("VERIFY_REQUESTS", "100")))
    concurrency = max(1, int(os.getenv("VERIFY_CONCURRENCY", "10")))
    timeout_seconds = max(0.5, float(os.getenv("VERIFY_TIMEOUT_SECONDS", "15")))
    targets = build_targets()

    if not targets:
        print("No verification targets configured.", file=sys.stderr)
        return 1

    print(
        json.dumps(
            {
                "requests_per_target": total_requests,
                "concurrency": concurrency,
                "timeout_seconds": timeout_seconds,
                "targets": [target.name for target in targets],
            },
            indent=2,
        )
    )

    exit_code = 0
    for target in targets:
        results: list[dict[str, object]] = []
        with ThreadPoolExecutor(max_workers=concurrency) as executor:
            futures = [
                executor.submit(run_request, target, timeout_seconds)
                for _ in range(total_requests)
            ]
            for future in as_completed(futures):
                results.append(future.result())

        durations = [float(result["duration_ms"]) for result in results]
        error_count = sum(1 for result in results if not result["ok"])
        status_counts: dict[str, int] = {}
        for result in results:
            key = str(result["status"])
            status_counts[key] = status_counts.get(key, 0) + 1

        summary = {
            "target": target.name,
            "requests": len(results),
            "errors": error_count,
            "success_rate": round((len(results) - error_count) / len(results), 4),
            "mean_ms": round(statistics.fmean(durations), 2),
            "p50_ms": round(percentile(durations, 0.50), 2),
            "p95_ms": round(percentile(durations, 0.95), 2),
            "max_ms": round(max(durations), 2),
            "status_counts": status_counts,
        }
        print(json.dumps(summary, indent=2))
        if error_count > 0:
            exit_code = 1

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
