CREATE TABLE IF NOT EXISTS bucket_metadata (
    id TEXT PRIMARY KEY NOT NULL,
    bucket_name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);
