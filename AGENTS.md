# AGENTS.md

## Project
Job search platform with:
- job ingestion
- normalization
- search and ranking
- application tracking
- CV/job matching
- future ML service

## Current architecture
- web = React app
- engine-api = Rust backend
- ingestion = new Rust ingestion service
- ml = new Python ML/LLM service
- contracts = shared schemas and contracts

## Rules
- prefer small, incremental changes
- do not rewrite unrelated modules
- keep web stable
- keep engine-api as the canonical backend
- avoid large refactors unless requested
- update docs when architecture changes

## Read order
1. docs/01-product/vision.md
2. docs/02-architecture/system-overview.md
3. docs/03-domain/entities.md
4. docs/06-agents/codex.md

## Output style for coding agents
- concise
- technical
- list files changed
- explain why changes were made
