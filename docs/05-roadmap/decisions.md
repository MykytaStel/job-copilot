# Decisions

## Decision 001
Use a gradual migration strategy instead of a full rewrite.

Reason:
- preserve working flows
- reduce risk
- keep momentum

## Decision 002
Keep React frontend.

Reason:
- already exists
- migration target is backend/core, not UI framework

## Decision 003
Use Rust for the core engine.

Reason:
- long-term maintainability
- explicit domain modeling
- strong fit for engine-like architecture
- independent core logic

## Decision 004
Use Python for ML/LLM sidecar only.

Reason:
- best flexibility for model work
- easier experimentation
- avoids forcing ML concerns into the Rust core

## Decision 005
Move from SQLite to Postgres for new architecture.

Reason:
- better growth path
- better concurrency and service separation
- stronger fit for multi-service system

## Decision 006
Keep core ranking and domain logic independent from any specific LLM.

Reason:
- preserve control
- reduce vendor/model lock-in
- keep engine explainable and stable