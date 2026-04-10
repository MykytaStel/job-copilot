# engine-api

New Rust backend for:
- jobs API
- applications API
- ranking
- search
- gradual replacement of api-legacy

## Runtime

`engine-api` now includes optional Postgres bootstrapping for the migration off SQLite.

Environment variables:
- `PORT` default `8080`
- `DATABASE_URL` optional Postgres connection string
- `DATABASE_MAX_CONNECTIONS` default `5`
- `RUN_DB_MIGRATIONS` default `true`

Behavior:
- if `DATABASE_URL` is not set, `engine-api` still starts and reports database status as `disabled`
- if `DATABASE_URL` is set, `engine-api` opens a Postgres pool on startup
- if `RUN_DB_MIGRATIONS=true`, embedded SQL migrations from `migrations/` are applied on startup
