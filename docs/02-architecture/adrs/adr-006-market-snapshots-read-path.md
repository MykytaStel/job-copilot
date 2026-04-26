# ADR-006: Market intelligence should move toward snapshot-backed reads

## Status

Partially Implemented

## Context

Market intelligence endpoints (overview, companies, salary trends, role demand) answer
questions about the aggregate job supply. These questions are read-heavy and do not
require real-time accuracy: a market overview that is a few hours stale is acceptable.

Running aggregate queries directly against the live `jobs` table on every market request
creates two problems:

1. **Query load** — aggregate queries on a large `jobs` table are expensive and compete
   with the job feed queries that need low latency.
2. **Consistency** — aggregate results can shift mid-ingestion as new rows land, which
   produces inconsistent snapshots within a single page load.

A separate `market_snapshots` table lets ingestion write a stable aggregate at the end
of each successful run, and market route handlers read from that stable snapshot instead
of querying live job data.

## Decision

Market intelligence reads should be backed by `market_snapshots` rather than direct
queries against the live `jobs` table.

- **Ingestion writes snapshots.** After a successful ingestion run completes its upserts,
  it refreshes `market_snapshots` with the current aggregate view.
- **Market route handlers read from snapshots.** Engine-api market endpoints serve
  from `market_snapshots`, not from live `jobs` queries.
- **Staleness is acceptable.** Snapshot data reflects the state at the last successful
  ingestion run. This is consistent with the ingestion cadence (60-minute intervals)
  and user expectations for market overview data.
- **Ingestion remains the sole snapshot writer.** Engine-api market routes are read-only
  against the snapshot table. They do not trigger snapshot refreshes directly.

## Consequences

**Easier:**
- Market queries are cheap and consistent within a snapshot window.
- Aggregate reads do not compete with live feed queries for database resources.
- Market data is stable during an ingestion run; users see a consistent view.

**Harder:**
- Market data lags the live `jobs` table by up to one ingestion interval.
- If ingestion fails, snapshots become stale; staleness must be visible to users
  eventually (analytics freshness widget).
- The snapshot schema must be maintained alongside the live `jobs` schema as the
  canonical job shape evolves.

**Constraints created:**
- Do not add real-time market queries against `jobs` for features that can tolerate
  snapshot staleness.
- Ingestion is responsible for snapshot refresh; do not trigger refreshes from engine-api
  or web.
- An analytics freshness widget should expose snapshot recency so users can see when
  market data was last refreshed.

## Current State

**Implemented:**
- `market_snapshots` table exists and is refreshed by ingestion after successful upserts.
- Market endpoints exist in engine-api: overview, companies, salary trends, role demand.

**Partial / gaps:**
- Current market route handlers still query the live `jobs` table directly.
  The snapshot write path exists; the read-side decoupling is not yet complete.
- The analytics freshness widget that would surface snapshot recency in the web UI
  is not yet implemented.

The read-side migration is a planned slice, not a completed one.
See [current-state.md](../current-state.md) for the current known issues table.

## Related Docs

- [data-flow.md](../data-flow.md)
- [current-state.md](../current-state.md)
- [ADR-001: Rust domain authority](adr-001-rust-domain-authority.md)
