# Backlog (відкладено, але не забуто)

## Технічний борг
- PostgreSQL FTS5 index для global search (`GIN(to_tsvector(...))`)
- Ingestion stats endpoint (`GET /api/v1/admin/ingestion-stats`)
- `SCRAPE_INTERVAL_PEAK_MINUTES` — частіший scraping в пікові години
- Збільшити `SCRAPE_PAGES` до 5 для Djinni
- Profile completion % indicator
- Ingestion stats widget у Analytics ("Last updated X min ago")

## Scheduled Cleanup

### Remove legacy profile role deserialization by 2026-07-01

Goal: remove the temporary compatibility parser for old profile role keys after all
stored `profiles.primary_role` values are canonical `RoleId` keys.

Assumptions:
- Legacy values may still exist until a migration/backfill proves otherwise.
- The Rust engine remains the canonical owner of role identity and validation.

Files likely touched:
- `apps/engine-api/migrations/` — add/verify a backfill for legacy `profiles.primary_role` values
- `apps/engine-api/src/domain/role/role_id.rs` — delete `RoleId::parse_compat_key`
- `apps/engine-api/src/db/repositories/profiles.rs` — switch `ProfileRow` deserialization to `RoleId::parse_canonical_key`

Acceptance criteria:
- All known legacy keys (`react_native_developer`, `frontend_developer`, `backend_developer`,
  `fullstack_developer`, `ui_ux_designer`, `data_analyst`, `marketing_specialist`,
  `sales_manager`, `customer_support_specialist`, `recruiter`) are backfilled or explicitly
  rejected before the compatibility parser is removed.
- Profile deserialization accepts canonical role keys only.
- Legacy compatibility tests are removed or replaced with migration/backfill coverage.
- The TODO in `role_id.rs` is removed.

Verification commands:
- `cargo test --manifest-path apps/engine-api/Cargo.toml role_id`
- `cargo test --manifest-path apps/engine-api/Cargo.toml profiles`
- `cargo check --manifest-path apps/engine-api/Cargo.toml`

Risks / tradeoffs:
- Removing the parser before production-like data is backfilled will make affected profiles fail
  to deserialize.
- Keeping the parser past 2026-07-01 weakens the canonical role catalog invariant by preserving
  old role keys in the read path.

## Домен
- Company resolution v2 (нормалізація назв компаній)
- Compensation normalization (USD/EUR/UAH уніфікація)
- Richer geographic model (city-level, не тільки country)
- Source trust score (якість source впливає на ranking)
- User feedback weighting (чим більше сигналів — тим сильніший вплив)
- Company notes (коментарі до компанії від юзера)

## Фічі
- Alerting / watchlists (email коли нова вакансія за критеріями)
- Timeline analytics (активність по тижнях)
- Export: PDF резюме + CSV application list
- Semantic embeddings + pgvector (після достатнього training set)
- Telegram bot integration (повернення фічі зі старого проекту)
- Job sharing / referral links

## ML
- Sentence-transformers (`all-MiniLM-L6-v2`) для semantic match
- A/B testing scoring variants
- Outcome tracking (чи отримали оффер після apply)
- Reranker v3 з більш складною моделлю (після 200+ labeled examples)
