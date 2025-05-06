CREATE TABLE IF NOT EXISTS bucket_metadata (
    id TEXT PRIMARY KEY,
    bucket_name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
