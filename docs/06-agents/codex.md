# Codex Guide

## Goal
Help migrate the repository from a TypeScript monolith to:
- React web
- Rust engine API
- Rust ingestion
- Python ML service

## Before coding
Always read:
1. AGENTS.md
2. docs/01-product/vision.md
3. docs/02-architecture/system-overview.md
4. docs/03-domain/entities.md

## Rules
- make small changes
- do not rewrite unrelated modules
- keep apps/web stable
- treat apps/engine-api as the canonical backend
- prefer clear names and modular code
- document structural changes

## Output
Return:
1. summary of changes
2. files changed
3. any follow-up work needed
