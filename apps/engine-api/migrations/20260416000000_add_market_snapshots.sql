CREATE TABLE market_snapshots (
    id TEXT PRIMARY KEY,
    snapshot_date DATE NOT NULL,
    snapshot_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX ON market_snapshots(snapshot_date, snapshot_type);

CREATE INDEX ON jobs USING GIN(to_tsvector('simple', title || ' ' || description_text));
