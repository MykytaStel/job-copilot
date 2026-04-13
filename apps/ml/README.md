# ml

Python ML/LLM service for:
- job extraction
- fit analysis
- reranking
- future adapter-based model integration

## Current slice

This service now exposes a read-only Phase 9 integration layer:
- fetch canonical profile data from `engine-api`
- fetch a dedicated lifecycle-aware job payload from `engine-api`
- compute heuristic fit analysis without writing to Postgres
- rerank a provided list of jobs for a persisted profile

## Runtime

Environment variables:
- `PORT` default `8000`
- `ENGINE_API_BASE_URL` default `http://localhost:8080`
- `ENGINE_API_TIMEOUT_SECONDS` default `10`

Install dependencies:

```bash
cd apps/ml
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Run the service:

```bash
cd apps/ml
PORT=8000 ENGINE_API_BASE_URL=http://localhost:8080 \
  uvicorn app.main:app --host 0.0.0.0 --port ${PORT:-8000}
```

## Endpoints

Health:

```bash
curl http://localhost:8000/health
```

Fit analysis for persisted canonical entities:

```bash
curl \
  -X POST http://localhost:8000/api/v1/fit/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "job_id": "job_backend_rust_001"
  }'
```

Rerank a list of jobs for a persisted profile:

```bash
curl \
  -X POST http://localhost:8000/api/v1/rerank \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "job_ids": [
      "job_backend_rust_001",
      "job_frontend_react_001"
    ]
  }'
```

## Rules

- `ml` does not write canonical job, profile, or application data
- `engine-api` remains the only write authority
- this service consumes `engine-api` over HTTP as a sidecar
- `app/engine_api_client.py` is the only place that knows the ML read-only engine-api surface
