-- Extensions installed at cluster init time.
-- pg_stat_statements requires shared_preload_libraries (set via postgres command arg).
-- pg_trgm and unaccent are contrib modules bundled in the official postgres:16 image.

CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS unaccent;
