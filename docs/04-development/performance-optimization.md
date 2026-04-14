# Performance / Optimization Track

## Backend
- cache expensive ranking inputs
- avoid repeated normalization work
- batch DB reads for list and fit state
- index by source / lifecycle / company / role / recency

## Web
- fix stale page transitions
- avoid duplicate fetches
- normalize query keys
- use list virtualization if job lists grow
- memoize expensive derived UI blocks

## ML
- cache enrichment results by content hash
- separate online vs offline jobs
- keep analytics generation asynchronous where possible
