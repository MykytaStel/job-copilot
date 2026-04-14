# System Overview

## Services
- `engine-api`: Rust, canonical backend
- `ingestion`: Rust, source ingestion
- `ml`: Python, enrichment/analytics
- `web`: React/Vite, UI

## Data flow
1. ingest jobs from selected sources
2. normalize and dedupe
3. store canonical jobs and variants
4. analyze candidate profile
5. build search profile
6. search and rank jobs
7. explain fit and track actions
8. enrich with ML/LLM
